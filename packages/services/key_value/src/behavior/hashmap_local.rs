use crate::{
    msg::{HashmapLocalEvent, HashmapRemoteEvent},
    KeyId, KeySource, KeyVersion, ReqId, SubKeyId, ValueType,
};
use bluesea_identity::NodeId;
use bluesea_router::RouteRule;
/// This hashmap local storage is used for storing and act with remote storage
/// Main idea is we using sdk to act with local storage, and local storage will sync that data to remote
/// Local storage allow us to set/get/del/subscribe/unsubscribe
///
/// With Set, we will send Set event to remote storage, and wait for ack. If acked, we will set acked flag to true
/// With Del, we will send Del event to remote storage, and wait for ack. If acked, we will set acked flag to true
///
/// If we not received ack in time, we will resend event to remote storage in tick
///
/// With acked data we also sync data to remote storage in tick each sync_each_ms
/// Same with subscribe/unsubscribe
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use utils::awaker::Awaker;

struct KeySlotData {
    value: Option<Vec<u8>>,
    ex: Option<u64>,
    version: KeyVersion,
    last_sync: u64,
    acked: bool,
}

struct KeySlotSubscribe {
    ex: Option<u64>,
    last_sync: u64,
    sub: bool,
    acked: bool,
    handler: Box<dyn FnMut(KeyId, SubKeyId, Option<Vec<u8>>, KeyVersion, KeySource) + Send + Sync>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HashmapKeyValueGetError {
    NetworkError,
    Timeout,
}

struct KeySlotGetCallback {
    timeout_after_ts: u64,
    callback: Box<dyn FnOnce(Result<Option<Vec<(SubKeyId, ValueType, KeyVersion, KeySource)>>, HashmapKeyValueGetError>) + Send + Sync>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct LocalStorageAction(pub(crate) HashmapRemoteEvent, pub(crate) RouteRule);

pub struct HashmapLocalStorage {
    req_id_seed: AtomicU64,
    version_seed: u16,
    sync_each_ms: u64,
    data: HashMap<(KeyId, SubKeyId), KeySlotData>,
    subscribe: HashMap<KeyId, KeySlotSubscribe>,
    output_events: VecDeque<LocalStorageAction>,
    get_queue: HashMap<ReqId, KeySlotGetCallback>,
    awake_notify: Arc<dyn Awaker>,
}

impl HashmapLocalStorage {
    /// create new local storage with provided timer and sync_each_ms. Sync_each_ms is used for sync data to remote storage incase of acked
    pub fn new(awake_notify: Arc<dyn Awaker>, sync_each_ms: u64) -> Self {
        Self {
            req_id_seed: AtomicU64::new(0),
            version_seed: 0,
            sync_each_ms,
            data: HashMap::new(),
            subscribe: HashMap::new(),
            output_events: VecDeque::new(),
            get_queue: HashMap::new(),
            awake_notify,
        }
    }

    pub fn change_awake_notify(&mut self, awake_notify: Arc<dyn Awaker>) {
        self.awake_notify = awake_notify;
    }

    fn gen_req_id(&self) -> u64 {
        return self.req_id_seed.fetch_add(1, Ordering::SeqCst);
    }

    fn gen_version(&mut self, now_ms: u64) -> u64 {
        let res = (now_ms << 16 | self.version_seed as u64) as u64;
        self.version_seed = self.version_seed.wrapping_add(1);
        return res;
    }

    /// Resend key releated event if not acked
    pub fn tick(&mut self, now: u64) {
        for ((key, sub_key), slot) in self.data.iter() {
            // we resend event each tick if not acked. If has data => Set, no data => Del
            if !slot.acked {
                let req_id = self.gen_req_id();
                if let Some(value) = &slot.value {
                    log::debug!("[HashmapLocal] resend set key {} with version {}", key, slot.version);
                    self.output_events.push_back(LocalStorageAction(
                        HashmapRemoteEvent::Set(req_id, *key, *sub_key, value.clone(), slot.version, slot.ex.clone()),
                        RouteRule::ToKey(*key as u32),
                    ));
                } else {
                    log::debug!("[HashmapLocal] resend del key {} with version {}", key, slot.version);
                    self.output_events
                        .push_back(LocalStorageAction(HashmapRemoteEvent::Del(req_id, *key, *sub_key, slot.version), RouteRule::ToKey(*key as u32)));
                }
            }
        }

        for (key, slot) in self.subscribe.iter() {
            // we resend event each tick if not acked, corresponse with sub/unsub
            if !slot.acked {
                let req_id = self.gen_req_id();
                if slot.sub {
                    log::debug!("[HashmapLocal] resend sub key {}", key);
                    self.output_events
                        .push_back(LocalStorageAction(HashmapRemoteEvent::Sub(req_id, *key, slot.ex.clone()), RouteRule::ToKey(*key as u32)));
                } else {
                    log::debug!("[HashmapLocal] resend unsub key {}", key);
                    self.output_events.push_back(LocalStorageAction(HashmapRemoteEvent::Unsub(req_id, *key), RouteRule::ToKey(*key as u32)));
                }
            }
        }

        // we sync data each sync_each_ms with each data which acked
        let mut removed_keys = Vec::new();
        for ((key, sub_key), slot) in self.data.iter() {
            if slot.acked && now - slot.last_sync >= self.sync_each_ms {
                let req_id = self.gen_req_id();
                if let Some(value) = &slot.value {
                    log::debug!("[HashmapLocal] sync set key {} with version {}", key, slot.version);
                    self.output_events.push_back(LocalStorageAction(
                        HashmapRemoteEvent::Set(req_id, *key, *sub_key, value.clone(), slot.version, slot.ex.clone()),
                        RouteRule::ToKey(*key as u32),
                    ));
                } else {
                    log::debug!("[HashmapLocal] del key {} with version {} after acked", key, slot.version);
                    // Just removed if acked and no data
                    removed_keys.push((*key, *sub_key));
                }
            }
        }

        // we set last_sync here for avoid borrowed mutable Self twice
        for (_key, slot) in self.data.iter_mut() {
            if slot.acked && now - slot.last_sync >= self.sync_each_ms {
                slot.last_sync = now;
            }
        }

        let mut unsub_keys = Vec::new();
        // we sync subscribe each sync_each_ms with each subscribe which acked
        for (key, slot) in self.subscribe.iter() {
            if slot.acked && now - slot.last_sync >= self.sync_each_ms {
                let req_id = self.gen_req_id();
                if slot.sub {
                    log::debug!("[HashmapLocal] sync sub key {}", key);
                    self.output_events
                        .push_back(LocalStorageAction(HashmapRemoteEvent::Sub(req_id, *key, slot.ex.clone()), RouteRule::ToKey(*key as u32)));
                } else {
                    log::debug!("[HashmapLocal] remove sub key {} after acked", key);
                    // Just remove if acked and unsub
                    unsub_keys.push(*key);
                }
            }
        }

        // we set last_sync here for avoid borrowed mutable Self twice
        for (_key, slot) in self.subscribe.iter_mut() {
            if slot.acked && now - slot.last_sync >= self.sync_each_ms {
                slot.last_sync = now;
            }
        }

        // we get timeout getter
        let mut timeout_gets = Vec::new();
        for (req_id, slot) in self.get_queue.iter() {
            if now >= slot.timeout_after_ts {
                timeout_gets.push(*req_id);
            }
        }

        // we clear timeout getter
        for req_id in timeout_gets {
            if let Some(slot) = self.get_queue.remove(&req_id) {
                log::debug!("[HashmapLocal] get key {} timeout", req_id);
                (slot.callback)(Err(HashmapKeyValueGetError::Timeout));
            }
        }

        for key in removed_keys {
            self.data.remove(&key);
        }

        for key in unsub_keys {
            self.subscribe.remove(&key);
        }
    }

    pub fn on_event(&mut self, from: NodeId, event: HashmapLocalEvent) {
        log::debug!("[HashmapLocal] on_event from {} {:?}", from, event);

        match event {
            HashmapLocalEvent::SetAck(_req_id, key, sub_key, version, success) => {
                if success {
                    if let Some(slot) = self.data.get_mut(&(key, sub_key)) {
                        // we acked if version match
                        if slot.version == version {
                            slot.acked = true;
                        }
                    }
                } else {
                    // TODO: we should avoid race condition here, when multiple node set with same key
                    // let new_version = self.gen_version();
                    // if let Some(slot) = self.data.get_mut(&key) {
                    //     // we regenete if version match, because of remote reject that version
                    //     if slot.version < version {
                    //         slot.version = new_version;
                    //     }
                    // }
                }
            }
            HashmapLocalEvent::GetAck(req_id, _key, value) => {
                if let Some(slot) = self.get_queue.remove(&req_id) {
                    (slot.callback)(Ok(value))
                } else {
                }
            }
            HashmapLocalEvent::DelAck(_req_id, key, sub_key, version) => {
                if let Some(slot) = self.data.get_mut(&(key, sub_key)) {
                    if let Some(deleted_version) = version {
                        // we acked if deleted version older than current version
                        if slot.version >= deleted_version {
                            slot.acked = true;
                        }
                    } else {
                        // incase of NoneKeyVersion, we just acked
                        slot.acked = true;
                    }
                }
            }
            HashmapLocalEvent::SubAck(_req_id, key_id) => {
                if let Some(slot) = self.subscribe.get_mut(&key_id) {
                    if slot.sub {
                        slot.acked = true;
                    }
                }
            }
            HashmapLocalEvent::UnsubAck(_req_id, key_id, success) => {
                if success {
                    if let Some(slot) = self.subscribe.get_mut(&key_id) {
                        if slot.sub == false {
                            slot.acked = true;
                        }
                    }
                }
            }
            HashmapLocalEvent::OnKeySet(req_id, key, sub_key, value, version, source) => {
                self.output_events.push_back(LocalStorageAction(HashmapRemoteEvent::OnKeySetAck(req_id), RouteRule::ToNode(from)));
                if let Some(slot) = self.subscribe.get_mut(&key) {
                    if slot.sub {
                        (slot.handler)(key, sub_key, Some(value), version, source);
                    }
                }
            }
            HashmapLocalEvent::OnKeyDel(req_id, key, sub_key, version, source) => {
                self.output_events.push_back(LocalStorageAction(HashmapRemoteEvent::OnKeyDelAck(req_id), RouteRule::ToNode(from)));
                if let Some(slot) = self.subscribe.get_mut(&key) {
                    if slot.sub {
                        (slot.handler)(key, sub_key, None, version, source);
                    }
                }
            }
        }
    }

    pub fn pop_action(&mut self) -> Option<LocalStorageAction> {
        self.output_events.pop_front()
    }

    pub fn set(&mut self, now_ms: u64, key: KeyId, sub_key: SubKeyId, value: ValueType, ex: Option<u64>) {
        let req_id = self.gen_req_id();
        let version = self.gen_version(now_ms);
        log::debug!("[HashmapLocal] set key {} with version {}", key, version);
        self.data.insert(
            (key, sub_key),
            KeySlotData {
                value: Some(value.clone()),
                ex,
                version,
                last_sync: 0,
                acked: false,
            },
        );

        self.output_events
            .push_back(LocalStorageAction(HashmapRemoteEvent::Set(req_id, key, sub_key, value, version, ex), RouteRule::ToKey(key as u32)));
        self.awake_notify.notify();
    }

    pub fn get(
        &mut self,
        now_ms: u64,
        key: KeyId,
        callback: Box<dyn FnOnce(Result<Option<Vec<(SubKeyId, ValueType, KeyVersion, KeySource)>>, HashmapKeyValueGetError>) + Send + Sync>,
        timeout_ms: u64,
    ) {
        let req_id = self.gen_req_id();
        log::debug!("[HashmapLocal] get key {} with req_id {}", key, req_id);
        self.get_queue.insert(
            req_id,
            KeySlotGetCallback {
                timeout_after_ts: now_ms + timeout_ms,
                callback,
            },
        );
        self.output_events.push_back(LocalStorageAction(HashmapRemoteEvent::Get(req_id, key), RouteRule::ToKey(key as u32)));
        self.awake_notify.notify();
    }

    pub fn del(&mut self, key: KeyId, sub_key: SubKeyId) {
        let req_id = self.gen_req_id();
        log::debug!("[HashmapLocal] del key {} with req_id {}", key, req_id);
        if let Some(slot) = self.data.get_mut(&(key, sub_key)) {
            slot.value = None;
            slot.last_sync = 0;
            slot.acked = false;

            self.output_events
                .push_back(LocalStorageAction(HashmapRemoteEvent::Del(req_id, key, sub_key, slot.version), RouteRule::ToKey(key as u32)));
            self.awake_notify.notify();
        }
    }

    pub fn subscribe(&mut self, key: KeyId, ex: Option<u64>, handler: Box<dyn FnMut(KeyId, SubKeyId, Option<Vec<u8>>, KeyVersion, KeySource) + Send + Sync>) {
        if self.subscribe.contains_key(&key) {
            log::warn!("[HashmapLocal] subscribe key {} but already subscribed", key);
            return;
        }

        let req_id = self.gen_req_id();
        log::debug!("[HashmapLocal] subscribe key {} with req_id {}", key, req_id);
        self.subscribe.insert(
            key,
            KeySlotSubscribe {
                ex,
                last_sync: 0,
                sub: true,
                acked: false,
                handler,
            },
        );
        self.output_events.push_back(LocalStorageAction(HashmapRemoteEvent::Sub(req_id, key, ex), RouteRule::ToKey(key as u32)));
        self.awake_notify.notify();
    }

    pub fn unsubscribe(&mut self, key: KeyId) {
        let req_id = self.gen_req_id();
        if let Some(slot) = self.subscribe.get_mut(&key) {
            slot.sub = false;
            slot.last_sync = 0;
            slot.acked = false;

            log::debug!("[HashmapLocal] unsubscribe key {} with req_id {}", key, req_id);

            self.output_events.push_back(LocalStorageAction(HashmapRemoteEvent::Unsub(req_id, key), RouteRule::ToKey(key as u32)));
            self.awake_notify.notify();
        } else {
            log::warn!("[HashmapLocal] unsubscribe key {} but not subscribed", key);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bluesea_router::RouteRule;
    use parking_lot::Mutex;
    use utils::awaker::{Awaker, MockAwaker};

    use crate::{
        behavior::hashmap_local::LocalStorageAction,
        msg::{HashmapLocalEvent, HashmapRemoteEvent},
    };

    use super::HashmapLocalStorage;

    #[test]
    fn set_should_mark_after_ack() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify.clone(), 10000);

        storage.set(0, 1, 2, vec![1], None);
        assert_eq!(awake_notify.pop_awake_count(), 1);

        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Set(0, 1, 2, vec![1], 0, None), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        storage.on_event(2, HashmapLocalEvent::SetAck(0, 1, 2, 0, true));

        //after received ack should not resend event
        storage.tick(0);
        assert_eq!(storage.pop_action(), None);
    }

    // #[test]
    // fn should_renegerate_set_event_if_ack_failed() {
    //     let timer = Arc::new(utils::MockTimer::default());
    //     let awake_notify = Arc::new(MockAwaker::default());
    //     let mut storage = LocalStorage::new(awake_notify, 10000);

    //     storage.set(1, vec![1], None);
    //     assert_eq!(storage.pop_action(), Some(LocalStorageAction(RemoteEvent::Set(0, 1, vec![1], 0, None), RouteRule::ToKey(1))));
    //     assert_eq!(storage.pop_action(), None);

    //     storage.on_event(2, LocalEvent::SetAck(0, 1, 0, false));

    //     //after received ack with failed => should regenerate new version
    //     storage.tick();
    //     assert_eq!(storage.pop_action(), Some(LocalStorageAction(RemoteEvent::Set(1, 1, vec![1], 1, None), RouteRule::ToKey(1))));
    //     assert_eq!(storage.pop_action(), None);

    //     storage.on_event(2, LocalEvent::SetAck(1, 1, 1, true));
    //     storage.tick();
    //     assert_eq!(storage.pop_action(), None);
    // }

    #[test]
    fn set_should_generate_new_version() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify, 10000);

        storage.set(0, 1, 2, vec![1], None);
        assert!(storage.pop_action().is_some());
        assert!(storage.pop_action().is_none());

        storage.set(1000, 1, 2, vec![2], None);
        assert_eq!(
            storage.pop_action(),
            Some(LocalStorageAction(HashmapRemoteEvent::Set(1, 1, 2, vec![2], 65536001, None), RouteRule::ToKey(1)))
        );
        assert_eq!(storage.pop_action(), None);

        storage.on_event(2, HashmapLocalEvent::SetAck(1, 1, 2, 65536001, true));

        //after received ack should not resend event
        storage.tick(1000);
        assert_eq!(storage.pop_action(), None);
    }

    #[test]
    fn set_should_retry_after_tick_and_not_received_ack() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify, 10000);

        storage.set(0, 1, 2, vec![1], None);
        assert!(storage.pop_action().is_some());
        assert!(storage.pop_action().is_none());

        //because dont received ack, should resend event
        storage.tick(0);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Set(1, 1, 2, vec![1], 0, None), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);
    }

    #[test]
    fn set_acked_should_resend_each_sync_each_ms() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify, 10000);

        storage.set(0, 1, 2, vec![1], None);
        assert!(storage.pop_action().is_some());
        assert!(storage.pop_action().is_none());

        storage.on_event(2, HashmapLocalEvent::SetAck(0, 1, 2, 0, true));

        //after received ack should not resend event
        storage.tick(0);
        assert_eq!(storage.pop_action(), None);

        //should resend if timer greater than sync_each_ms
        storage.tick(10001);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Set(1, 1, 2, vec![1], 0, None), RouteRule::ToKey(1))));
    }

    #[test]
    fn del_should_mark_after_ack() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify.clone(), 10000);

        storage.set(0, 1, 2, vec![1], None);
        assert!(storage.pop_action().is_some());
        assert!(storage.pop_action().is_none());
        storage.on_event(2, HashmapLocalEvent::SetAck(0, 1, 2, 0, true));

        storage.del(1, 2);
        assert_eq!(awake_notify.pop_awake_count(), 2);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Del(1, 1, 2, 0), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        //after received ack should not resend event
        storage.on_event(2, HashmapLocalEvent::DelAck(0, 1, 2, Some(0)));
        storage.tick(0);
        assert_eq!(storage.pop_action(), None);
    }

    #[test]
    fn del_should_mark_after_ack_older() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify.clone(), 10000);

        storage.set(0, 1, 2, vec![1], None);
        assert!(storage.pop_action().is_some());
        assert!(storage.pop_action().is_none());
        storage.on_event(2, HashmapLocalEvent::SetAck(0, 1, 2, 0, true));

        storage.set(1000, 1, 2, vec![2], None);
        assert!(storage.pop_action().is_some());
        assert!(storage.pop_action().is_none());
        storage.on_event(2, HashmapLocalEvent::SetAck(0, 1, 2, 0, true));

        storage.del(1, 2);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Del(2, 1, 2, 65536001), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        //after received ack should not resend event
        storage.on_event(2, HashmapLocalEvent::DelAck(2, 1, 2, Some(65536001)));
        storage.tick(1000);
        assert_eq!(storage.pop_action(), None);
    }

    #[test]
    fn del_should_retry_after_tick_and_not_received_ack() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify.clone(), 10000);

        storage.set(0, 1, 2, vec![1], None);
        assert!(storage.pop_action().is_some());
        assert!(storage.pop_action().is_none());
        storage.on_event(2, HashmapLocalEvent::SetAck(0, 1, 2, 0, true));

        storage.del(1, 2);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Del(1, 1, 2, 0), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        storage.tick(0);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Del(2, 1, 2, 0), RouteRule::ToKey(1))));
    }

    #[test]
    fn sub_should_mark_after_ack() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify.clone(), 10000);

        storage.subscribe(1, None, Box::new(|_, _, _, _, _| {}));
        assert_eq!(awake_notify.pop_awake_count(), 1);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Sub(0, 1, None), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        storage.on_event(2, HashmapLocalEvent::SubAck(0, 1));

        storage.tick(0);
        assert_eq!(storage.pop_action(), None);
    }

    #[test]
    fn sub_event_test() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify, 10000);
        let received_events = Arc::new(Mutex::new(Vec::new()));

        let received_events_clone = received_events.clone();
        storage.subscribe(
            1,
            None,
            Box::new(move |key, sub_key, value, version, source| {
                received_events_clone.lock().push((key, sub_key, value, version, source));
            }),
        );
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Sub(0, 1, None), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        storage.on_event(2, HashmapLocalEvent::SubAck(0, 1));

        storage.tick(0);
        assert_eq!(storage.pop_action(), None);

        // fake incoming event
        storage.on_event(2, HashmapLocalEvent::OnKeySet(0, 1, 2, vec![1], 0, 1000));
        storage.on_event(2, HashmapLocalEvent::OnKeyDel(0, 1, 2, 0, 1000));

        assert_eq!(*received_events.lock(), vec![(1, 2, Some(vec![1]), 0, 1000), (1, 2, None, 0, 1000),]);
    }

    #[test]
    fn sub_should_retry_after_tick_and_not_received_ack() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify, 10000);

        storage.subscribe(1, None, Box::new(|_, _, _, _, _| {}));
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Sub(0, 1, None), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        storage.tick(0);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Sub(1, 1, None), RouteRule::ToKey(1))));
    }

    #[test]
    fn sub_acked_should_resend_each_sync_each_ms() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify, 10000);

        storage.subscribe(1, None, Box::new(|_, _, _, _, _| {}));
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Sub(0, 1, None), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        storage.on_event(2, HashmapLocalEvent::SubAck(0, 1));

        storage.tick(0);
        assert_eq!(storage.pop_action(), None);

        //should resend if timer greater than sync_each_ms
        storage.tick(10001);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Sub(1, 1, None), RouteRule::ToKey(1))));
    }

    #[test]
    fn unsub_should_mark_after_ack() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify.clone(), 10000);

        storage.subscribe(1, None, Box::new(|_, _, _, _, _| {}));
        assert!(storage.pop_action().is_some());
        assert!(storage.pop_action().is_none());

        storage.on_event(2, HashmapLocalEvent::SubAck(0, 1));

        //sending unsub
        storage.unsubscribe(1);
        assert_eq!(awake_notify.pop_awake_count(), 2);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Unsub(1, 1), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        //after received ack should not resend event
        storage.on_event(2, HashmapLocalEvent::UnsubAck(1, 1, true));
        storage.tick(0);
        assert_eq!(storage.pop_action(), None);
    }

    #[test]
    fn unsub_should_retry_after_tick_if_not_received_ack() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify, 10000);

        storage.subscribe(1, None, Box::new(|_, _, _, _, _| {}));
        assert!(storage.pop_action().is_some());
        assert!(storage.pop_action().is_none());

        storage.on_event(2, HashmapLocalEvent::SubAck(0, 1));

        //sending unsub
        storage.unsubscribe(1);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Unsub(1, 1), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        //if not received ack should resend event each tick
        storage.tick(0);
        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Unsub(2, 1), RouteRule::ToKey(1))));
    }

    #[test]
    fn get_should_callback_correct_value() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify, 10000);

        let got_value = Arc::new(Mutex::new(None));
        let got_value_clone = got_value.clone();
        storage.get(
            0,
            1,
            Box::new(move |result| {
                *got_value_clone.lock() = Some(result);
            }),
            1000,
        );

        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Get(0, 1), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        //fake received result
        storage.on_event(2, HashmapLocalEvent::GetAck(0, 1, Some(vec![(2, vec![1], 0, 1000)])));
        assert_eq!(*got_value.lock(), Some(Ok(Some(vec![(2, vec![1], 0, 1000)]))));
    }

    #[test]
    fn get_should_timeout_after_no_ack() {
        let awake_notify = Arc::new(MockAwaker::default());
        let mut storage = HashmapLocalStorage::new(awake_notify, 10000);

        let got_value = Arc::new(Mutex::new(None));
        let got_value_clone = got_value.clone();
        storage.get(
            0,
            1,
            Box::new(move |result| {
                *got_value_clone.lock() = Some(result);
            }),
            1000,
        );

        assert_eq!(storage.pop_action(), Some(LocalStorageAction(HashmapRemoteEvent::Get(0, 1), RouteRule::ToKey(1))));
        assert_eq!(storage.pop_action(), None);

        //after timeout should callback error
        storage.tick(1001);
        assert_eq!(*got_value.lock(), Some(Err(super::HashmapKeyValueGetError::Timeout)));
    }
}
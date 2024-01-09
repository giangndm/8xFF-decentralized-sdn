//! Local relay is a relay for local pubsub only
//! This will store all local consumer id, local consumer service id, local publisher id, local publisher service id
//! And it will awake logic when there is a change in consumer or publisher
//! It will help to fire event to all consumers or publishers

use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    sync::Arc,
};

use async_std::channel::Sender;
use atm0s_sdn_identity::NodeId;
use atm0s_sdn_utils::{awaker::Awaker, error_handle::ErrorUtils};
use bytes::Bytes;

use crate::{msg::PubsubSdkEvent, ChannelIdentify, LocalPubId};

use super::{feedback::Feedback, ChannelUuid, LocalSubId};

#[derive(Debug, PartialEq, Eq)]
pub enum LocalRelayAction {
    Subscribe(ChannelUuid),
    Unsubscribe(ChannelUuid),
    Publish(ChannelUuid),
    Unpublish(ChannelUuid),
    ToService(u8, PubsubSdkEvent),
}

struct ChannelContainer {
    sdk_subs: Vec<(LocalSubId, Sender<(LocalSubId, NodeId, ChannelUuid, Bytes)>)>,
    sdk_pubs: Vec<(LocalPubId, Sender<Feedback>)>,
    service_subs: Vec<u8>,
    service_pubs: Vec<u8>,
}

pub struct LocalRelay {
    channels: HashMap<ChannelUuid, ChannelContainer>,
    actions: VecDeque<LocalRelayAction>,
    awaker: Arc<dyn Awaker>,
}

impl LocalRelay {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            actions: VecDeque::new(),
            awaker: Arc::new(atm0s_sdn_utils::awaker::MockAwaker::default()),
        }
    }

    pub fn set_awaker(&mut self, awaker: Arc<dyn Awaker>) {
        self.awaker = awaker;
    }

    /// add a consumer to a channel
    /// if channel wasn't subscribe the channel, subscribe it then awake behavior for action
    pub fn on_sdk_sub(&mut self, channel: ChannelUuid, uuid: LocalSubId, sender: Sender<(LocalSubId, NodeId, ChannelUuid, Bytes)>) {
        match self.channels.entry(channel) {
            Entry::Occupied(mut entry) => {
                //only push if not exist
                if !entry.get().sdk_subs.iter().any(|(id, _)| *id == uuid) {
                    if entry.get_mut().sdk_subs.is_empty() && entry.get_mut().service_subs.is_empty() {
                        self.actions.push_back(LocalRelayAction::Subscribe(channel));
                        self.awaker.notify();
                    }
                    entry.get_mut().sdk_subs.push((uuid, sender));
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(ChannelContainer {
                    sdk_subs: vec![(uuid, sender)],
                    sdk_pubs: Vec::new(),
                    service_subs: Vec::new(),
                    service_pubs: Vec::new(),
                });
                self.actions.push_back(LocalRelayAction::Subscribe(channel));
                self.awaker.notify();
            }
        }
    }

    /// remove a consumer from a channel
    /// if channel has no consumer after removed, unsubscribe it then awake behavior for action
    pub fn on_sdk_unsub(&mut self, channel: ChannelUuid, uuid: LocalSubId) {
        if let Some(entry) = self.channels.get_mut(&channel) {
            entry.sdk_subs.retain(|(id, _)| *id != uuid);
            if entry.sdk_subs.is_empty() && entry.service_subs.is_empty() {
                self.actions.push_back(LocalRelayAction::Unsubscribe(channel));
                self.awaker.notify();

                if entry.sdk_pubs.is_empty() && entry.service_pubs.is_empty() {
                    self.channels.remove(&channel);
                }
            }
        }
    }

    /// add a service as consumer of a channel
    /// if channel wasn't subscribe the channel, subscribe it then awake behavior for action
    /// note that a service will only subscribe a channel once, so if a service subscribe a channel twice, it will only be counted as one
    pub fn on_service_sub(&mut self, channel: ChannelUuid, service: u8) {
        match self.channels.entry(channel) {
            Entry::Occupied(mut entry) => {
                //only push if not exist
                if !entry.get().service_subs.contains(&service) {
                    if entry.get_mut().sdk_subs.is_empty() && entry.get_mut().service_subs.is_empty() {
                        self.actions.push_back(LocalRelayAction::Subscribe(channel));
                        self.awaker.notify();
                    }
                    entry.get_mut().service_subs.push(service);
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(ChannelContainer {
                    sdk_subs: Vec::new(),
                    sdk_pubs: Vec::new(),
                    service_subs: vec![service],
                    service_pubs: Vec::new(),
                });
                self.actions.push_back(LocalRelayAction::Subscribe(channel));
                self.awaker.notify();
            }
        }
    }

    /// remove a service from a channel consumer
    /// if channel has no consumer after removed, unsubscribe it then awake behavior for action
    /// note that we use flag for storing service subsribe state therefore it will unsub for service immediately even if the service subscribe the channel twice
    pub fn on_service_unsub(&mut self, channel: ChannelUuid, service: u8) {
        if let Some(entry) = self.channels.get_mut(&channel) {
            entry.service_subs.retain(|id| *id != service);
            if entry.sdk_subs.is_empty() && entry.service_subs.is_empty() {
                self.actions.push_back(LocalRelayAction::Unsubscribe(channel));
                self.awaker.notify();

                if entry.sdk_pubs.is_empty() && entry.service_pubs.is_empty() {
                    self.channels.remove(&channel);
                }
            }
        }
    }

    /// add a publisher to a channel
    /// if channel wasn't publish the channel, publish it then awake behavior for action
    pub fn on_sdk_pub(&mut self, channel: ChannelUuid, uuid: LocalPubId, fb_sender: Sender<Feedback>) {
        match self.channels.entry(channel) {
            Entry::Occupied(mut entry) => {
                //only push if not exist
                if !entry.get().sdk_pubs.iter().any(|(id, _)| *id == uuid) {
                    if entry.get_mut().sdk_pubs.is_empty() && entry.get_mut().service_pubs.is_empty() {
                        self.actions.push_back(LocalRelayAction::Publish(channel));
                        self.awaker.notify();
                    }
                    entry.get_mut().sdk_pubs.push((uuid, fb_sender));
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(ChannelContainer {
                    sdk_subs: Vec::new(),
                    sdk_pubs: vec![(uuid, fb_sender)],
                    service_subs: Vec::new(),
                    service_pubs: Vec::new(),
                });
                self.actions.push_back(LocalRelayAction::Subscribe(channel));
                self.awaker.notify();
            }
        }
    }

    /// remove a publisher from a channel
    /// if channel has no publisher after removed, unpublish it then awake behavior for action
    pub fn on_sdk_unpub(&mut self, channel: ChannelUuid, uuid: LocalPubId) {
        if let Some(entry) = self.channels.get_mut(&channel) {
            entry.sdk_pubs.retain(|(id, _)| *id != uuid);
            if entry.sdk_pubs.is_empty() && entry.service_pubs.is_empty() {
                self.actions.push_back(LocalRelayAction::Unpublish(channel));
                self.awaker.notify();

                if entry.sdk_subs.is_empty() && entry.service_subs.is_empty() {
                    self.channels.remove(&channel);
                }
            }
        }
    }

    /// add a service as publisher of a channel
    /// if channel wasn't publish the channel, publish it then awake behavior for action
    /// note that a service will only publish a channel once, so if a service publish a channel twice, it will only be counted as one
    pub fn on_service_pub(&mut self, channel: ChannelUuid, service: u8) {
        match self.channels.entry(channel) {
            Entry::Occupied(mut entry) => {
                //only push if not exist
                if !entry.get().service_pubs.contains(&service) {
                    if entry.get_mut().sdk_pubs.is_empty() && entry.get_mut().service_pubs.is_empty() {
                        self.actions.push_back(LocalRelayAction::Publish(channel));
                        self.awaker.notify();
                    }
                    entry.get_mut().service_pubs.push(service);
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(ChannelContainer {
                    sdk_subs: Vec::new(),
                    sdk_pubs: Vec::new(),
                    service_subs: Vec::new(),
                    service_pubs: vec![service]
                });
                self.actions.push_back(LocalRelayAction::Publish(channel));
                self.awaker.notify();
            }
        }
    }

    /// remove a service from a channel publisher
    /// if channel has no publisher after removed, unpublish it then awake behavior for action
    /// note that we use flag for storing service publish state therefore it will unpublish for service immediately even if the service publish the channel twice
    pub fn on_service_unpub(&mut self, channel: ChannelUuid, service: u8) {
        if let Some(entry) = self.channels.get_mut(&channel) {
            entry.service_pubs.retain(|id| *id != service);
            if entry.sdk_pubs.is_empty() && entry.service_pubs.is_empty() {
                self.actions.push_back(LocalRelayAction::Unpublish(channel));
                self.awaker.notify();

                if entry.sdk_subs.is_empty() && entry.service_subs.is_empty() {
                    self.channels.remove(&channel);
                }
            }
        }
    }

    /// sending feedback to all publishers and services
    /// if channel has no publisher and service, return None
    /// otherwise return Some(())
    pub fn feedback(&self, uuid: ChannelUuid, fb: Feedback) -> Option<()> {
        let channel = self.channels.get(&uuid)?;
        for (_, sender) in &channel.sdk_pubs {
            sender.try_send(fb.clone()).print_error("Should send feedback");
        }
        for service in &channel.service_pubs {
            self.actions.push_back(LocalRelayAction::ToService(*service, PubsubSdkEvent::PubOnFeedback(uuid, fb.clone())));
        }

        if channel.sdk_pubs.is_empty() && channel.service_pubs.is_empty() {
            None
        } else {
            Some(())
        }
    }

    pub fn relay(&self, source: NodeId, uuid: ChannelUuid, data: Bytes) -> Option<()> {
        let channel = self.channels.get(&uuid)?;
        for (local_pub_uuid, sender) in &channel.sdk_subs {
            sender.try_send((*local_pub_uuid, uuid, source, data.clone())).print_error("Should send data");
        }
        for service in &channel.service_subs {
            self.actions.push_back(LocalRelayAction::ToService(*service, PubsubSdkEvent::SubOnData(ChannelIdentify::new(uuid, source), data.clone())));
        }

        if channel.sdk_subs.is_empty() && channel.service_subs.is_empty() {
            None
        } else {
            Some(())
        }
    }

    pub fn pop_action(&mut self) -> Option<LocalRelayAction> {
        self.actions.pop_front()
    }
}

#[cfg(test)]
mod tests {
    // use std::sync::Arc;

    // use atm0s_sdn_utils::awaker::Awaker;
    // use bytes::Bytes;

    // use crate::{
    //     relay::{feedback::FeedbackType, local::LocalRelayAction},
    //     ChannelIdentify,
    // };

    // #[test]
    // fn first_pub_should_awake_and_output_action() {
    //     let awake = Arc::new(atm0s_sdn_utils::awaker::MockAwaker::default());
    //     let mut relay = super::LocalRelay::new();
    //     relay.set_awaker(awake.clone());

    //     let (tx, _rx) = async_std::channel::bounded(1);
    //     relay.on_local_pub(1, 10, tx.clone());

    //     assert_eq!(awake.pop_awake_count(), 1);
    //     assert_eq!(relay.pop_action(), Some(LocalRelayAction::Publish(1)));
    //     assert_eq!(relay.pop_action(), None);

    //     relay.on_local_pub(1, 11, tx);
    //     assert_eq!(awake.pop_awake_count(), 0);
    //     assert_eq!(relay.pop_action(), None);

    //     relay.on_local_unpub(1, 10);
    //     assert_eq!(awake.pop_awake_count(), 0);
    //     assert_eq!(relay.pop_action(), None);

    //     relay.on_local_unpub(1, 11);
    //     assert_eq!(awake.pop_awake_count(), 1);
    //     assert_eq!(relay.pop_action(), Some(LocalRelayAction::Unpublish(1)));
    // }

    // #[test]
    // fn should_relay_to_all_consumers() {
    //     let mut relay = super::LocalRelay::new();

    //     let (tx1, rx1) = async_std::channel::bounded(1);
    //     let (tx2, rx2) = async_std::channel::bounded(1);

    //     relay.on_local_sub(10, tx1);
    //     relay.on_local_sub(11, tx2);

    //     let data1 = Bytes::from("hello1");
    //     relay.relay(1, 1000, &[10, 11], data1.clone());
    //     assert_eq!(rx1.try_recv(), Ok((10, 1, 1000, data1.clone())));
    //     assert_eq!(rx2.try_recv(), Ok((11, 1, 1000, data1.clone())));

    //     let data2 = Bytes::from("hello2");
    //     let data3 = Bytes::from("hello2");
    //     relay.relay(1, 1000, &[10], data2.clone());
    //     relay.relay(1, 1000, &[11], data3.clone());
    //     assert_eq!(rx1.try_recv(), Ok((10, 1, 1000, data2.clone())));
    //     assert_eq!(rx2.try_recv(), Ok((11, 1, 1000, data3.clone())));
    // }

    // #[test]
    // fn should_feedback_to_all_publishers() {
    //     let mut relay = super::LocalRelay::new();

    //     let (tx1, rx1) = async_std::channel::bounded(1);
    //     let (tx2, rx2) = async_std::channel::bounded(1);

    //     relay.on_local_pub(1, 10, tx1);
    //     relay.on_local_pub(1, 11, tx2);

    //     let channel = ChannelIdentify::new(1, 1);
    //     let fb = super::Feedback {
    //         channel,
    //         id: 1,
    //         feedback_type: FeedbackType::Passthrough(vec![1]),
    //     };

    //     relay.feedback(1, fb.clone());
    //     assert_eq!(rx1.try_recv(), Ok(fb.clone()));
    //     assert_eq!(rx2.try_recv(), Ok(fb.clone()));
    // }
}

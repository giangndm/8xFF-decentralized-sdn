use std::sync::Arc;

use bluesea_identity::NodeId;
use bytes::Bytes;
use parking_lot::RwLock;
use utils::Timer;

use crate::relay::{
    feedback::{Feedback, FeedbackConsumerId, FeedbackType},
    local::LocalRelay,
    logic::PubsubRelayLogic,
    ChannelIdentify, ChannelUuid, LocalSubId,
};

pub struct ConsumerSingle {
    uuid: LocalSubId,
    channel: ChannelIdentify,
    logic: Arc<RwLock<PubsubRelayLogic>>,
    local: Arc<RwLock<LocalRelay>>,
    rx: async_std::channel::Receiver<(LocalSubId, NodeId, ChannelUuid, Bytes)>,
    timer: Arc<dyn Timer>,
}

impl ConsumerSingle {
    pub fn new(uuid: LocalSubId, channel: ChannelIdentify, logic: Arc<RwLock<PubsubRelayLogic>>, local: Arc<RwLock<LocalRelay>>, max_queue_size: usize, timer: Arc<dyn Timer>) -> Self {
        let (tx, rx) = async_std::channel::bounded(max_queue_size);
        logic.write().on_local_sub(channel, uuid);
        local.write().on_local_sub(uuid, tx);

        Self {
            uuid,
            channel,
            logic,
            local,
            rx,
            timer,
        }
    }

    pub fn uuid(&self) -> LocalSubId {
        self.uuid
    }

    pub fn feedback(&self, id: u8, feedback_type: FeedbackType) {
        let fb = Feedback {
            channel: self.channel,
            id,
            feedback_type,
        };
        if let Some(local_fb) = self.logic.write().on_feedback(self.timer.now_ms(), self.channel, FeedbackConsumerId::Local(self.uuid), fb) {
            self.local.read().feedback(self.channel.uuid(), local_fb);
        }
    }

    pub async fn recv(&self) -> Option<(LocalSubId, NodeId, ChannelUuid, Bytes)> {
        self.rx.recv().await.ok()
    }
}

impl Drop for ConsumerSingle {
    fn drop(&mut self) {
        self.logic.write().on_local_unsub(self.channel, self.uuid);
        self.local.write().on_local_unsub(self.uuid);
    }
}
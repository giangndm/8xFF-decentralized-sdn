use crate::{relay::ChannelIdentify, ChannelUuid, Feedback};
use bytes::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq)]
pub enum PubsubServiceBehaviourEvent {}

#[derive(Debug, PartialEq, Eq)]
pub enum PubsubServiceHandlerEvent {}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PubsubRemoteEvent {
    Sub(ChannelIdentify),
    Unsub(ChannelIdentify),
    SubAck(ChannelIdentify, bool),   //did it added, incase of false, it means it already subscribed
    UnsubAck(ChannelIdentify, bool), //did it removed, incase of false, it means it already unsubscribed
}

#[derive(Debug, PartialEq, Eq)]
pub enum PubsubSdkEvent {
    Sub(ChannelUuid),
    Feedback(ChannelUuid, Feedback),
    Unsub(ChannelUuid),
    SubOnData(ChannelIdentify, Bytes),
    Pub(ChannelUuid),
    Unpub(ChannelUuid),
    PubOnFeedback(ChannelUuid, Feedback),
    PubSendData(ChannelUuid, Bytes),
}

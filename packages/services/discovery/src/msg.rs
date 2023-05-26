use bluesea_identity::{NodeAddr, NodeId};

pub enum DiscoveryBehaviorEvent {
    OnNetworkMessage(DiscoveryMsg),
}

pub enum DiscoveryHandlerEvent {}

#[derive(PartialEq, Debug)]
pub enum DiscoveryMsg {
    FindKey(u32, NodeId),
    FindKeyRes(u32, Vec<(NodeId, NodeAddr)>),
}

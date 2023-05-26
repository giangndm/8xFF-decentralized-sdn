use bluesea_identity::NodeId;
use std::collections::HashMap;

#[derive(Default)]
pub struct ConnectionGrouping {
    nodes: HashMap<NodeId, HashMap<u32, bool>>,
}

impl ConnectionGrouping {
    pub fn add(&mut self, node: NodeId, conn_id: u32) -> bool {
        let entry = self.nodes.entry(node).or_insert_with(Default::default);
        let new = entry.is_empty();
        entry.insert(conn_id, true);
        new
    }

    pub fn remove(&mut self, node: NodeId, conn_id: u32) -> bool {
        if let Some(conns) = self.nodes.get_mut(&node) {
            conns.remove(&conn_id);
            if conns.is_empty() {
                self.nodes.remove(&node);
                return true;
            }
        }
        false
    }
}

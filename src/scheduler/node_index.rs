use crate::my_imports::*;


pub struct NodeRTree (RTree<Node>);


impl RTreeObject for Node {
    type Envelope = AABB<(i64, i64, i64)>;

    fn envelope(&self) -> Self::Envelope {
        assert!(self.spec.available_cpu <= (i64::MAX as u64));
        assert!(self.spec.available_memory <= (i64::MAX as u64));
        AABB::from_point((self.spec.available_cpu as i64, self.spec.available_memory as i64, self.metadata.uid as i64))
    }
}


impl NodeRTree {
    pub fn new() -> Self {
        Self { 0: RTree::new() }
    }

    pub fn find_suitable_nodes(&self, target_cpu: u64, target_memory: u64, result: &mut Vec<Node>) {
        let query_box = AABB::from_corners((target_cpu as i64, target_memory as i64, 0), (i64::MAX, i64::MAX, i64::MAX));

        result.clear();
        for node in self.0.locate_in_envelope(&query_box) {
            result.push(node.clone());
        }
    }

    pub fn insert(&mut self, node: Node) {
        self.0.insert(node);
    }

    pub fn remove(&mut self, node: &Node) -> Node {
        return self.0.remove(node).unwrap();
    }
}

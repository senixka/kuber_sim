use crate::my_imports::*;

pub struct NodeRTree(RTree<Node>);

impl RTreeObject for Node {
    type Envelope = AABB<(i64, i64, i64)>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point((
            self.spec.available_cpu,
            self.spec.available_memory,
            self.metadata.uid as i64,
        ))
    }
}

impl NodeRTree {
    pub fn new() -> Self {
        Self { 0: RTree::new() }
    }

    #[inline]
    pub fn find_suitable_nodes(&self, target_cpu: i64, target_memory: i64, result: &mut Vec<Node>) {
        let query_box = AABB::from_corners((target_cpu, target_memory, i64::MIN), (i64::MAX, i64::MAX, i64::MAX));

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

///////////////////////////////////////////// Test /////////////////////////////////////////////////

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_rtree_many_times() {
        for _ in 0..10 {
            test_node_rtree();
        }
    }

    #[test]
    pub fn test_node_rtree() {
        let mut index = NodeRTree::new();

        let mut node1 = Node::default();
        let mut node2 = Node::default();
        let mut node3 = Node::default();
        let mut node4 = Node::default();

        node1.spec.available_cpu = 0;
        node1.spec.available_memory = 0;
        node1.metadata.uid = 1;
        node2.spec.available_cpu = 10;
        node2.spec.available_memory = 0;
        node2.metadata.uid = 2;
        node3.spec.available_cpu = 0;
        node3.spec.available_memory = 10;
        node3.metadata.uid = 3;
        node4.spec.available_cpu = 10;
        node4.spec.available_memory = 10;
        node4.metadata.uid = 4;

        index.insert(node1);
        index.insert(node2);
        index.insert(node3);
        index.insert(node4);

        let mut result = Vec::<Node>::new();

        index.find_suitable_nodes(0, 0, &mut result);
        assert_eq!(result.len(), 4);

        index.find_suitable_nodes(10, 0, &mut result);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].is_both_consumable(10, 0), true);
        assert_eq!(result[1].is_both_consumable(10, 0), true);

        index.find_suitable_nodes(0, 10, &mut result);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].is_both_consumable(0, 10), true);
        assert_eq!(result[1].is_both_consumable(0, 10), true);

        index.find_suitable_nodes(10, 10, &mut result);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].is_both_consumable(10, 10), true);

        index.find_suitable_nodes(11, 0, &mut result);
        assert_eq!(result.len(), 0);
    }
}

use crate::my_imports::*;
use std::collections::BinaryHeap;
use rstar::{AABB, RTree};
use crate::scheduler::active_queue::{ActiveQCmpMinUid, TraitActiveQCmp};
use crate::scheduler::backoff_queue::{BackOffQExponential, TraitBackOffQ};
use crate::scheduler::node_index::NodeRTree;

pub struct Test ();

impl Test {
    pub fn test_all() {
        Test::test_backoff_queue();
        Test::test_active_queue_cmp_min_uid();
        Test::test_node_rtree();
    }

    pub fn test_backoff_queue() {
        let mut q = BackOffQExponential::new(1.0, 10.0);
        q.push(1, 0, 0.0);

        assert_eq!(q.try_pop(0.95), None);
        assert_eq!(q.try_pop(1.05), Some(1));
        assert_eq!(q.try_pop(1.05), None);

        q.push(1, 1, 1.0);

        assert_eq!(q.try_pop(2.95), None);
        assert_eq!(q.try_pop(3.05), Some(1));
        assert_eq!(q.try_pop(3.05), None);

        q.push(1, 4, 1.0);

        assert_eq!(q.try_pop(10.95), None);
        assert_eq!(q.try_pop(11.05), Some(1));
        assert_eq!(q.try_pop(11.05), None);

        q.push(1, 3, 1.0);
        q.push(2, 1, 1.0);
        q.push(3, 2, 1.0);

        assert_eq!(q.try_pop(9.05), Some(2));
        assert_eq!(q.try_pop(9.05), Some(3));
        assert_eq!(q.try_pop(9.05), Some(1));
        assert_eq!(q.try_pop(9.05), None);
    }

    pub fn test_active_queue_cmp_min_uid() {
        let mut q = BinaryHeap::<ActiveQCmpMinUid>::new();

        let mut p1 = Pod::default();
        let mut p2 = Pod::default();
        let mut p22 = Pod::default();
        let mut p3 = Pod::default();

        p1.metadata.uid = 1;
        p2.metadata.uid = 2;
        p22.metadata.uid = 2;
        p3.metadata.uid = 3;

        q.push(ActiveQCmpMinUid::wrap(p2.clone()));
        q.push(ActiveQCmpMinUid::wrap(p1.clone()));
        q.push(ActiveQCmpMinUid::wrap(p22.clone()));
        q.push(ActiveQCmpMinUid::wrap(p3.clone()));

        assert_eq!(q.pop().unwrap().0, p1);
        assert_eq!(q.pop().unwrap().0, p2);
        assert_eq!(q.pop().unwrap().0, p22);
        assert_eq!(q.pop().unwrap().0, p3);
        assert_eq!(q.pop(), None);
    }

    pub fn test_node_rtree() {
        let mut index = NodeRTree::new();

        let mut node1 = Node::default();
        let mut node2 = Node::default();
        let mut node3 = Node::default();
        let mut node4 = Node::default();

        node1.spec.available_cpu = 0;  node1.spec.available_memory = 0;  node1.metadata.uid = 1;
        node2.spec.available_cpu = 10; node2.spec.available_memory = 0;  node2.metadata.uid = 2;
        node3.spec.available_cpu = 0;  node3.spec.available_memory = 10; node3.metadata.uid = 3;
        node4.spec.available_cpu = 10; node4.spec.available_memory = 10; node4.metadata.uid = 4;

        index.insert(node1);
        index.insert(node2);
        index.insert(node3);
        index.insert(node4);

        let mut result = Vec::<Node>::new();

        index.find_suitable_nodes(0, 0, &mut result);
        assert_eq!(result.len(), 4);

        index.find_suitable_nodes(10, 0, &mut result);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].is_consumable(10, 0), true);
        assert_eq!(result[1].is_consumable(10, 0), true);

        index.find_suitable_nodes(0, 10, &mut result);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].is_consumable(0, 10), true);
        assert_eq!(result[1].is_consumable(0, 10), true);

        index.find_suitable_nodes(10, 10, &mut result);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].is_consumable(10, 10), true);

        index.find_suitable_nodes(11, 0, &mut result);
        assert_eq!(result.len(), 0);
    }
}
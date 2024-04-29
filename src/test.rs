use crate::my_imports::*;


pub struct Test ();


impl Test {
    pub fn test_all() {
        Test::test_backoff_queue_exponential();
        Test::test_backoff_queue_constant();
        Test::test_active_queue_cmp_uid();
        Test::test_node_rtree();
        Test::test_active_queue_cmp_priority();
        Test::test_pod_qos_class();
    }

    pub fn test_backoff_queue_constant() {
        let mut q = BackOffQConstant::new(30.0);
        q.push(1, 0, 0.0);

        assert_eq!(q.try_pop(29.95), None);
        assert_eq!(q.try_pop(30.05), Some(1));
        assert_eq!(q.try_pop(30.05), None);

        q.push(1, 1000, 1.0);

        assert_eq!(q.try_pop(30.95), None);
        assert_eq!(q.try_pop(31.05), Some(1));
        assert_eq!(q.try_pop(31.05), None);

        q.push(1, 123, 5.0);

        assert_eq!(q.try_pop(34.95), None);
        assert_eq!(q.try_pop(35.05), Some(1));
        assert_eq!(q.try_pop(35.05), None);

        q.push(1, 3, 1.0);
        q.push(2, 1, 1.0);
        q.push(2, 2, 2.0);
        q.push(3, 2, 1.0);

        assert_eq!(q.try_pop(31.05), Some(1));
        assert_eq!(q.try_pop(31.05), Some(2));
        assert_eq!(q.try_pop(31.05), Some(3));
        assert_eq!(q.try_pop(31.05), None);

        q.push(3, 1, 1.0);
        q.push(1, 1, 1.0);
        q.push(2, 1, 1.0);

        assert_eq!(q.try_pop(31.05), Some(1));
        assert_eq!(q.try_pop(31.05), Some(2));
        assert_eq!(q.try_pop(31.05), Some(3));
        assert_eq!(q.try_pop(31.05), None);

        q.push(3, 1, 1.0);
        q.push(1, 1, 1.0);
        q.push(2, 1, 1.0);

        assert_eq!(q.try_remove(1), true);
        assert_eq!(q.try_remove(1), false);
        assert_eq!(q.try_pop(31.05), Some(2));
        assert_eq!(q.try_remove(2), false);
        assert_eq!(q.try_remove(3), true);
        assert_eq!(q.try_pop(31.05), None);
    }

    pub fn test_backoff_queue_exponential() {
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

        q.push(3, 1, 1.0);
        q.push(1, 1, 1.0);
        q.push(2, 1, 1.0);

        assert_eq!(q.try_pop(9.05), Some(1));
        assert_eq!(q.try_pop(9.05), Some(2));
        assert_eq!(q.try_pop(9.05), Some(3));
        assert_eq!(q.try_pop(9.05), None);

        q.push(3, 1, 1.0);
        q.push(1, 1, 1.0);
        q.push(2, 1, 1.0);

        assert_eq!(q.try_remove(1), true);
        assert_eq!(q.try_remove(1), false);
        assert_eq!(q.try_pop(9.05), Some(2));
        assert_eq!(q.try_remove(2), false);
        assert_eq!(q.try_remove(3), true);
        assert_eq!(q.try_pop(9.05), None);
    }

    pub fn test_pod_qos_class() {
        let mut q: BTreeSet<(QoSClass, i64, u64)> = BTreeSet::new();
        q.insert((QoSClass::Guaranteed, 2, 2));
        q.insert((QoSClass::BestEffort, 0, 1));
        q.insert((QoSClass::Guaranteed, 1, 0));
        q.insert((QoSClass::BestEffort, 0, 0));
        q.insert((QoSClass::Burstable, 0, 0));

        assert_eq!(q.pop_first(), Some((QoSClass::BestEffort, 0, 0)));
        assert_eq!(q.pop_first(), Some((QoSClass::BestEffort, 0, 1)));
        assert_eq!(q.pop_first(), Some((QoSClass::Burstable, 0, 0)));
        assert_eq!(q.pop_first(), Some((QoSClass::Guaranteed, 1, 0)));
        assert_eq!(q.pop_first(), Some((QoSClass::Guaranteed, 2, 2)));
        assert_eq!(q.pop_first(), None);
    }

    pub fn test_active_queue_cmp_priority() {
        let mut q = ActiveMaxQ::<ActiveQCmpPriority>::new();

        let mut p1 = Pod::default();
        let mut p2 = Pod::default();
        let mut p22 = Pod::default();
        let mut p3 = Pod::default();

        p1.spec.priority = 1;   p1.metadata.uid = 1;
        p2.spec.priority = 2;   p1.metadata.uid = 1;
        p22.spec.priority = 2;  p22.metadata.uid = 22;
        p3.spec.priority = 3;   p3.metadata.uid = 3;

        q.push(p2.clone());
        q.push(p3.clone());
        q.push(p1.clone());
        q.push(p22.clone());

        assert_eq!(q.try_pop().unwrap(), p3);
        assert_eq!(q.try_pop().unwrap(), p22);
        assert_eq!(q.try_pop().unwrap(), p2);
        assert_eq!(q.try_pop().unwrap(), p1);
        assert_eq!(q.try_pop(), None);

        q.push(p2.clone());
        q.push(p3.clone());
        q.push(p1.clone());
        q.push(p22.clone());

        assert_eq!(q.try_remove(p2.clone()), true);
        assert_eq!(q.try_remove(p2.clone()), false);
        assert_eq!(q.try_pop().unwrap(), p3);
        assert_eq!(q.try_pop().unwrap(), p22);
        assert_eq!(q.try_pop().unwrap(), p1);
        assert_eq!(q.try_pop(), None);
    }

    pub fn test_active_queue_cmp_uid() {
        let mut q = ActiveMinQ::<ActiveQCmpUid>::new();

        let mut p1 = Pod::default();
        let mut p2 = Pod::default();
        let mut p22 = Pod::default();
        let mut p3 = Pod::default();

        p1.metadata.uid = 1;
        p2.metadata.uid = 2;
        p22.metadata.uid = 2;
        p3.metadata.uid = 3;

        q.push(p2.clone());
        q.push(p1.clone());
        q.push(p22.clone());
        q.push(p3.clone());

        assert_eq!(q.try_pop().unwrap(), p1);
        assert_eq!(q.try_pop().unwrap(), p2);
        assert_eq!(q.try_pop().unwrap(), p3);
        assert_eq!(q.try_pop(), None);

        q.push(p2.clone());
        q.push(p1.clone());
        q.push(p22.clone());
        q.push(p3.clone());

        assert_eq!(q.try_remove(p1.clone()), true);
        assert_eq!(q.try_remove(p1.clone()), false);
        assert_eq!(q.try_pop().unwrap(), p22);
        assert_eq!(q.try_pop().unwrap(), p3);
        assert_eq!(q.try_pop(), None);
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

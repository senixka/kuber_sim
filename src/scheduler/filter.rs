use crate::my_imports::*;


pub type FilterPluginT = fn(&HashMap<u64, Pod>,
                            &HashMap<u64, Pod>,
                            &HashMap<u64, Node>,
                            &Pod,
                            &Node) -> bool;


pub fn filter_always_true(_: &HashMap<u64, Pod>,
                          _: &HashMap<u64, Pod>,
                          _: &HashMap<u64, Node>,
                          _: &Pod,
                          _: &Node) -> bool {
    return true;
}

pub fn filter_always_false(_: &HashMap<u64, Pod>,
                           _: &HashMap<u64, Pod>,
                           _: &HashMap<u64, Node>,
                           _: &Pod,
                           _: &Node) -> bool {
    return true;
}

pub fn filter_node_selector(_: &HashMap<u64, Pod>,
                            _: &HashMap<u64, Pod>,
                            _: &HashMap<u64, Node>,
                            pod: &Pod,
                            node: &Node) -> bool {
    for (key, pod_value) in &pod.spec.node_selector {
        match node.metadata.labels.get(key) {
            None => {
                return false
            }
            Some(node_value) => {
                if *pod_value != *node_value {
                    return false
                }
            }
        }
    }
    return true;
}

pub fn filter_requested_resources_available(_: &HashMap<u64, Pod>,
                                            _: &HashMap<u64, Pod>,
                                            _: &HashMap<u64, Node>,
                                            pod: &Pod,
                                            node: &Node) -> bool {
    return node.is_consumable(pod.spec.request_cpu, pod.spec.request_memory);
}

pub fn filter_taints_tolerations(_: &HashMap<u64, Pod>,
                                 _: &HashMap<u64, Pod>,
                                 _: &HashMap<u64, Node>,
                                 pod: &Pod,
                                 node: &Node) -> bool {
    for taint in &node.spec.taints {
        if taint.effect != TaintTolerationEffect::NoSchedule {
            continue;
        }

        // Hear only taints with NoSchedule
        let mut matches = false;
        for tol in &pod.spec.tolerations {
            matches |= taint.matches(tol);
        }

        if !matches {
            return false;
        }
    }
    return true;
}

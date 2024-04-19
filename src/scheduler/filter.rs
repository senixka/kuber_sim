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

pub fn filter_node_affinity(_: &HashMap<u64, Pod>,
                            _: &HashMap<u64, Pod>,
                            _: &HashMap<u64, Node>,
                            _: &Pod,
                            _: &Node) -> bool {
    // TODO: impl it
    panic!("Node affinity");
}

pub fn filter_taints_tolerations(_: &HashMap<u64, Pod>,
                                 _: &HashMap<u64, Pod>,
                                 _: &HashMap<u64, Node>,
                                 pod: &Pod,
                                 node: &Node) -> bool {
    let special = pod.spec.tolerations.get("all");
    if special.is_some() && (special.unwrap().operator == TaintTolerationOperator::Exists) {
        return true;
    }

    for (taint_key, taint_value) in &node.spec.taints {
        let toleration = pod.spec.tolerations.get(taint_key);
        match toleration {
            Some(TolerationValue { value, operator, effect} ) => {
                if taint_value.effect != TaintTolerationEffect::Empty
                    && *effect != TaintTolerationEffect::Empty
                    && taint_value.effect != *effect {
                    return false;
                }
                if *operator == TaintTolerationOperator::Equal && taint_value.value != *value {
                    return false;
                }
            }
            None => {
                return false;
            }
        }
    }

    return true;
}

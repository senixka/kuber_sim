use crate::my_imports::*;


pub enum PluginFilter {
    AlwaysTrue,
    AlwaysFalse,
    RequestedResourcesAvailable,
    NodeSelector,
}

impl PluginFilter {
    #[inline]
    pub fn filter(&self,
                  running_pods: &HashMap<u64, Pod>,
                  pending_pods: &HashMap<u64, Pod>,
                  all_nodes: &HashMap<u64, Node>,
                  pod: &Pod,
                  node: &Node) -> bool {
        match self {
            PluginFilter::AlwaysTrue => {
                true
            }
            PluginFilter::AlwaysFalse => {
                false
            }
            PluginFilter::RequestedResourcesAvailable => {
                node.is_consumable(pod.spec.request_cpu, pod.spec.request_memory)
            }
            PluginFilter::NodeSelector => {
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
        }
    }
}

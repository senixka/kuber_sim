use crate::my_imports::*;


pub enum PluginFilter {
    AlwaysTrue,
    AlwaysFalse,
    RequestedResourcesAvailable,
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
        }
    }
}

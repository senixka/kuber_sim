use crate::my_imports::*;


pub enum PluginFilter {
    AlwaysTrue,
    AlwaysFalse,
    RequestedResourcesAvailable,
}

impl PluginFilter {
    #[inline]
    pub fn is_schedulable(&self, pod: &Pod, node: &Node) -> bool {
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

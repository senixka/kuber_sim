use crate::my_imports::*;

pub type FilterPluginF = fn(&HashMap<u64, Pod>, &HashMap<u64, Pod>, &HashMap<u64, Node>, &Pod, &Node) -> bool;

pub trait IFilterPlugin {
    fn name(&self) -> String;

    fn filter(
        &self,
        running_pods: &HashMap<u64, Pod>,
        pending_pods: &HashMap<u64, Pod>,
        nodes: &HashMap<u64, Node>,
        pod: &Pod,
        node: &Node,
    ) -> bool;

    fn clone(&self) -> Box<dyn IFilterPlugin + Send>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct FilterAlwaysTrue;

impl IFilterPlugin for FilterAlwaysTrue {
    fn name(&self) -> String {
        return "FilterAlwaysTrue".to_string();
    }

    fn filter(&self, _: &HashMap<u64, Pod>, _: &HashMap<u64, Pod>, _: &HashMap<u64, Node>, _: &Pod, _: &Node) -> bool {
        return true;
    }

    fn clone(&self) -> Box<dyn IFilterPlugin + Send> {
        return Box::new(FilterAlwaysTrue);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct FilterAlwaysFalse;

impl IFilterPlugin for FilterAlwaysFalse {
    fn name(&self) -> String {
        return "FilterAlwaysFalse".to_string();
    }

    fn filter(&self, _: &HashMap<u64, Pod>, _: &HashMap<u64, Pod>, _: &HashMap<u64, Node>, _: &Pod, _: &Node) -> bool {
        return false;
    }

    fn clone(&self) -> Box<dyn IFilterPlugin + Send> {
        return Box::new(FilterAlwaysFalse);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct FilterNodeSelector;

impl IFilterPlugin for FilterNodeSelector {
    fn name(&self) -> String {
        return "FilterNodeSelector".to_string();
    }

    fn filter(
        &self,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Node>,
        pod: &Pod,
        node: &Node,
    ) -> bool {
        for (key, pod_value) in &pod.spec.node_selector {
            match node.metadata.labels.get(key) {
                None => return false,
                Some(node_value) => {
                    if *pod_value != *node_value {
                        return false;
                    }
                }
            }
        }
        return true;
    }

    fn clone(&self) -> Box<dyn IFilterPlugin + Send> {
        return Box::new(FilterNodeSelector);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct FilterRequestedResourcesAvailable;

impl IFilterPlugin for FilterRequestedResourcesAvailable {
    fn name(&self) -> String {
        return "FilterRequestedResourcesAvailable".to_string();
    }

    fn filter(
        &self,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Node>,
        pod: &Pod,
        node: &Node,
    ) -> bool {
        return node.is_both_consumable(pod.spec.request_cpu, pod.spec.request_memory);
    }

    fn clone(&self) -> Box<dyn IFilterPlugin + Send> {
        return Box::new(FilterRequestedResourcesAvailable);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct FilterTaintsTolerations;

impl IFilterPlugin for FilterTaintsTolerations {
    fn name(&self) -> String {
        return "FilterTaintsTolerations".to_string();
    }

    fn filter(
        &self,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Node>,
        pod: &Pod,
        node: &Node,
    ) -> bool {
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

    fn clone(&self) -> Box<dyn IFilterPlugin + Send> {
        return Box::new(FilterTaintsTolerations);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct FilterNodeAffinity;

impl IFilterPlugin for FilterNodeAffinity {
    fn name(&self) -> String {
        return "FilterNodeAffinity".to_string();
    }

    fn filter(
        &self,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Node>,
        pod: &Pod,
        node: &Node,
    ) -> bool {
        return pod.spec.node_affinity.is_required_matches(&node);
    }

    fn clone(&self) -> Box<dyn IFilterPlugin + Send> {
        return Box::new(FilterNodeAffinity);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct FilterPreemption;

impl IFilterPlugin for FilterPreemption {
    fn name(&self) -> String {
        return "FilterPreemption".to_string();
    }

    fn filter(
        &self,
        running_pods: &HashMap<u64, Pod>,
        _: &HashMap<u64, Pod>,
        _: &HashMap<u64, Node>,
        pod: &Pod,
        node: &Node,
    ) -> bool {
        let (mut cpu, mut memory) = (node.spec.available_cpu, node.spec.available_memory);
        for &tmp_uid in &node.status.pods {
            let tmp_pod = running_pods.get(&tmp_uid).unwrap();
            if tmp_pod.spec.priority >= pod.spec.priority {
                continue;
            }

            cpu += tmp_pod.spec.request_cpu;
            memory += tmp_pod.spec.request_memory;

            if cpu >= pod.spec.request_cpu && memory >= pod.spec.request_memory {
                return true;
            }
        }

        return false;
    }

    fn clone(&self) -> Box<dyn IFilterPlugin + Send> {
        return Box::new(FilterPreemption);
    }
}

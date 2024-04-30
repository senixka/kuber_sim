use crate::my_imports::*;


/////////////////////////////////////////// API ////////////////////////////////////////////////////

// [Emit]:      { Init | HPA } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventAddPod {
    pub pod: Pod,
}

// [Emit]:      {} -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventRemovePod {
    pub pod_uid: u64,
}

// [Emit]:      { Init | CA } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventAddNode {
    pub kubelet_sim_id: dsc::Id,
    pub node: Node,
}

// [Emit]:      { CA } -> Api
// [Consume]:   Api -> { Kubelet | Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventRemoveNode {
    pub node_uid: u64,
}


// [Emit]:      { Kubelet } -> Api
// [Consume]:   Api -> { CA }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventRemoveNodeAck {
    pub node_uid: u64,
}


// [Emit]:      { Scheduler } -> Api
// [Consume]:   Api -> { Kubelet }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventUpdatePodFromScheduler {
    pub pod: Option<Pod>,
    pub pod_uid: u64,
    pub new_phase: PodPhase,
    pub node_uid: u64,
}


// [Emit]:      { Kubelet } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventUpdatePodFromKubelet {
    pub pod_uid: u64,
    pub new_phase: PodPhase,
    pub node_uid: u64,
}


///////////////////////////////////////////// Common ///////////////////////////////////////////////

// [Emit self]:      { CA | HPA | Scheduler | Monitoring }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventSelfUpdate {
}


// [Emit]:      {} -> Api
// [Consume]:   Api -> { CA | HPA }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventTurnOn {}


// [Emit]:      {} -> Api
// [Consume]:   Api -> { CA | HPA }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventTurnOff {}



/////////////////////////////////////////// Kubelet ////////////////////////////////////////////////


// [Emit]:      { Kubelet } -> Api
// [Consume]:   Api -> {}
#[derive(Clone, Serialize, Deserialize)]
pub struct EventUpdatePodMetricsFromKubelet {
    pub pod_uid: u64,
    pub current_cpu: f64,
    pub current_memory: f64,
}


// [Emit self]:      Kubelet
#[derive(Clone, Serialize, Deserialize)]
pub struct EventKubeletNextChange {
    pub pod_uid: u64,
}


///////////////////////////////////////// HPA  ////////////////////////////////////////////////

// [Emit]:      { HPA } -> Api
// [Consume]:   Api -> {}
#[derive(Clone, Serialize, Deserialize)]
pub struct EventGetHPAMetrics {
    pub pod_groups: Vec<u64>,
}

// [Emit]:      {} -> Api
// [Consume]:   Api -> { HPA }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventPostHPAMetrics {
    pub group_utilization: Vec<(u64, f64, f64)>,
}


// [Emit]:      { HPA } -> Api
// [Consume]:   Api -> {}
#[derive(Clone, Serialize, Deserialize)]
pub struct EventRemoveAnyPodInGroup {
    pub group_uid: u64,
}


///////////////////////////////////////// CA ///////////////////////////////////////////////////////

// [Emit]:      { Scheduler } -> Api
// [Consume]:   Api -> { CA }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventPostCAMetrics {
    pub pending_pod_count: u64,
    pub used_nodes_utilization: Vec<(u64, f64, f64)>,
    pub may_help: Option<u64>,
}


// [Emit]:      { CA } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct EventGetCAMetrics {
    pub used_nodes: Vec<u64>,
    pub available_nodes: Vec<NodeGroup>,
}


////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common_imports::*;
use crate::objects::node::Node;
use crate::objects::pod::{Pod, PodPhase};
use crate::objects::pod_group::PodGroup;

/////////////////////////////////////////// API ////////////////////////////////////////////////////

// [Emit]:      { Init | HPA } -> Api
// [Consume]:   Api -> { Scheduler | HPA | VPA }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventAddPod {
    pub pod: Pod,
}

// [Emit]:      { HPA } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventRemovePod {
    pub pod_uid: u64,
}

// [Emit]:      { Init } -> Api
// [Consume]:   Api -> {}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvenAddPodGroup {
    pub pod_group: PodGroup,
}

// [Emit]:      { Init } -> Api
// [Consume]:   Api -> {}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventRemovePodGroup {
    pub group_uid: u64,
}

// [Emit]:      { Init | CA } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventAddNode {
    pub kubelet_sim_id: dsc::Id,
    pub node: Node,
}

// [Emit]:      { CA } -> Api
// [Consume]:   Api -> { Kubelet | Scheduler }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventRemoveNode {
    pub node_uid: u64,
}

// [Emit]:      { Kubelet } -> Api
// [Consume]:   Api -> { CA }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventRemoveNodeAck {
    pub node_uid: u64,
}

// [Emit]:      { Scheduler } -> Api
// [Consume]:   Api -> { Kubelet }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventUpdatePodFromScheduler {
    pub pod: Option<Pod>,
    pub pod_uid: u64,
    pub preempt_uids: Option<Vec<u64>>,
    pub new_phase: PodPhase,
    pub node_uid: u64,
}

// [Emit]:      {} -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventPodUpdateToScheduler {
    pub pod_uid: u64,
    pub current_phase: PodPhase,
}

// [Emit]:      { Kubelet } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventPodUpdateFromKubelet {
    pub pod_uid: u64,
    pub current_phase: PodPhase,
    pub current_cpu: f64,
    pub current_memory: f64,
}

// [Emit]:      {} -> Api
// [Consume]:   Api -> { HPA }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventPodMetricsPost {
    pub group_uid: u64,
    pub pod_uid: u64,
    pub current_phase: PodPhase,
    pub current_cpu: f64,
    pub current_memory: f64,
}

///////////////////////////////////////////// Common ///////////////////////////////////////////////

// [Emit self]:      { CA | HPA | Scheduler | Monitoring }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventSelfUpdate {}

// [Emit]:      {} -> Api
// [Consume]:   Api -> { CA | HPA }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventTurnOn {}

// [Emit]:      {} -> Api
// [Consume]:   Api -> { CA | HPA }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventTurnOff {}

/////////////////////////////////////////// Kubelet ////////////////////////////////////////////////

// [Emit self]:      Kubelet
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventKubeletNextChange {
    pub pod_uid: u64,
}

///////////////////////////////////////// CA ///////////////////////////////////////////////////////

// [Emit self]:      { CA }
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventUpdateCAMetrics {}

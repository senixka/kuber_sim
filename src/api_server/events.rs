use crate::my_imports::*;


/////////////////////////////////////////// API ////////////////////////////////////////////////////

// [Emit]:      { Init | HPA } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIAddPod {
    pub pod: Pod,
}

// [Emit]:      { HPA } -> Api
// [Consume]:   Api -> { Kubelet | Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIRemovePod {
    pub pod_uid: u64,
}

// [Emit]:      { Init | CA } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIAddNode {
    pub kubelet_sim_id: dsc::Id,
    pub node: Node,
}

// [Emit]:      { CA } -> Api
// [Consume]:   Api -> { Kubelet | Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIRemoveNode {
    pub node_uid: u64,
}


// [Emit]:      { Kubelet } -> Api
// [Consume]:   Api -> { CA }
#[derive(Clone, Serialize, Deserialize)]
pub struct APICommitNodeRemove {
    pub node_uid: u64,
}


// [Emit]:      { Scheduler } -> Api
// [Consume]:   Api -> { Kubelet }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIUpdatePodFromScheduler {
    pub pod: Option<Pod>,
    pub pod_uid: u64,
    pub new_phase: PodPhase,
    pub node_uid: u64,
}


// [Emit]:      { Kubelet } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIUpdatePodFromKubelet {
    pub pod_uid: u64,
    pub new_phase: PodPhase,
    pub node_uid: u64,
}

// [Emit]:      { Kubelet } -> Api
// [Consume]:   Api -> {}
#[derive(Clone, Serialize, Deserialize)]
pub struct APIUpdatePodMetricsFromKubelet {
    pub pod_uid: u64,
    pub current_cpu: f64,
    pub current_memory: f64,
}



/////////////////////////////////////// Scheduler inner ////////////////////////////////////////////

// [Emit self]:      Scheduler
#[derive(Clone, Serialize, Deserialize)]
pub struct APISchedulerSelfUpdate {
}


// [Emit self]:      Scheduler
#[derive(Clone, Serialize, Deserialize)]
pub struct APISchedulerSecondChance {
    pub pod_uid: u64,
}



/////////////////////////////////////// Kubelet inner //////////////////////////////////////////////

// [Emit self]:      Kubelet
#[derive(Clone, Serialize, Deserialize)]
pub struct APIKubeletSelfNextChange {
    pub pod_uid: u64,
}



////////////////////////////////////// Monitoring inner ////////////////////////////////////////////

// [Emit self]:      Monitoring
#[derive(Clone, Serialize, Deserialize)]
pub struct APIMonitoringSelfUpdate {
}



///////////////////////////////////////// HPA inner ////////////////////////////////////////////////

// [Emit self]:      HPA
#[derive(Clone, Serialize, Deserialize)]
pub struct APIHPASelfUpdate {
}


// [Emit]:      { HPA } -> Api
// [Consume]:   Api -> {}
#[derive(Clone, Serialize, Deserialize)]
pub struct APIGetHPAMetrics {
    pub pod_groups: Vec<u64>,
}

// [Emit]:      {} -> Api
// [Consume]:   Api -> { HPA }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIPostHPAMetrics {
    pub pod_groups: Vec<(u64, f64, f64)>, // (group_uid, current cpu usage, current memory usage)
    //        group_uid--^  cpu-^    ^--memory
}


// [Emit]:      {} -> Api
// [Consume]:   Api -> { HPA }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIHPATurnOn {}


// [Emit]:      {} -> Api
// [Consume]:   Api -> { HPA }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIHPATurnOff {}


// [Emit]:      { HPA } -> Api
// [Consume]:   Api -> {}
#[derive(Clone, Serialize, Deserialize)]
pub struct APIRemoveAnyPodInGroup {
    pub group_uid: u64,
}


///////////////////////////////////////// CA ///////////////////////////////////////////////////////

// [Emit]:      { Scheduler } -> Api
// [Consume]:   Api -> { CA }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIPostCAMetrics {
    pub insufficient_resources_pending: u64,
    pub requests: Vec<(u64, u64)>,
    pub node_info: Vec<(u64, f64, f64)>,
}


// [Emit]:      { CA } -> Api
// [Consume]:   Api -> { Scheduler }
#[derive(Clone, Serialize, Deserialize)]
pub struct APIGetCAMetrics {
    pub node_list: Vec<u64>,
}


// [Emit self]:      CA
#[derive(Clone, Serialize, Deserialize)]
pub struct APICASelfUpdate {}


// [Emit self]:      CA
#[derive(Clone, Serialize, Deserialize)]
pub struct APICATurnOn {}


// [Emit self]:      CA
#[derive(Clone, Serialize, Deserialize)]
pub struct APICATurnOff {}


////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::my_imports::*;


// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub enum APIServerEvent {
//     InsertPod = 0,
//     RemovePod = 1,
//     InsertNode = 2,
//     RemoveNode = 3,
// }


#[derive(Clone, Serialize, Deserialize)]
pub struct APIAddPod {
    pub pod: Pod,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIRemovePod {
    pub pod_uid: u64,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIAddNode {
    pub kubelet_sim_id: dsc::Id,
    pub node: Node,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIRemoveNode {
    pub node_uid: u64,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIRemoveKubelet {
    pub node_uid: u64,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIUpdatePodFromScheduler {
    pub pod: Pod,
    pub new_phase: PodPhase,
    pub node_uid: u64,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIUpdatePodFromKubelet {
    pub pod_uid: u64,
    pub new_phase: PodPhase,
    pub node_uid: u64,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APISchedulerSelfUpdate {
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APISchedulerSecondChance {
    pub pod_uid: u64,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIMonitoringSelfUpdate {
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIKubeletSelfNextChange {
    pub pod_uid: u64,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIUpdatePodMetricsFromKubelet {
    pub pod_uid: u64,
    pub current_cpu: f64,
    pub current_memory: f64,
}


///////////////////////////////////////// HPA //////////////////////////////////////////////////////

#[derive(Clone, Serialize, Deserialize)]
pub struct APIHPASelfUpdate {
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIGetHPAMetrics {
    pub pod_groups: Vec<u64>,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIPostHPAMetrics {
    pub pod_groups: Vec<(u64, f64, f64)>,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIHPATurnOn {}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIHPATurnOff {}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIRemoveAnyPodInGroup {
    pub group_uid: u64,
}


///////////////////////////////////////// CA ///////////////////////////////////////////////////////

#[derive(Clone, Serialize, Deserialize)]
pub struct APIPostCAMetrics {
    pub insufficient_resources_pending: u64,
    pub requests: Vec<(u64, u64)>,
    pub node_info: Vec<(u64, f64, f64)>,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIGetCAMetrics {
    pub node_list: Vec<u64>,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APICASelfUpdate {}


#[derive(Clone, Serialize, Deserialize)]
pub struct APICATurnOn {}


#[derive(Clone, Serialize, Deserialize)]
pub struct APICATurnOff {}


#[derive(Clone, Serialize, Deserialize)]
pub struct APICommitCANodeRemove {
    pub node_uid: u64,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

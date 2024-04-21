use crate::my_imports::*;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum APIServerEvent {
    InsertPod = 0,
    RemovePod = 1,
    InsertNode = 2,
    RemoveNode = 3,
}


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
pub struct APIMonitoringSelfUpdate {
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIKubeletSelfNextChange {
    pub pod_uid: u64,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APISchedulerSecondChance {
    pub pod_uid: u64,
}


///////////////////////////////////////// CA ///////////////////////////////////////////////////////

#[derive(Clone, Serialize, Deserialize)]
pub struct APIPostCAMetrics {
    pub insufficient_resources_pending: u64,
    pub requests: Vec<(u64, u64)>,
}


#[derive(Clone, Serialize, Deserialize)]
pub struct APIGetCAMetrics {}


#[derive(Clone, Serialize, Deserialize)]
pub struct APICASelfUpdate {}


#[derive(Clone, Serialize, Deserialize)]
pub struct APICATurnOn {}


#[derive(Clone, Serialize, Deserialize)]
pub struct APICATurnOff {}

////////////////////////////////////////////////////////////////////////////////////////////////////

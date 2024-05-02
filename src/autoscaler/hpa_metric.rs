use crate::my_imports::*;


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HPAPodGroupMetrics {
    group_uid: u64,
    metrics: Option<Vec<HPAPodMetrics>>,
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HPAPodMetrics {
    pod_uid: u64,
    pod_phase: PodPhase,
    percent_cpu: f64,
    percent_memory: f64,
}

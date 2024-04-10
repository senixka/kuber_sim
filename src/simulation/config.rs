use std::fs;
use serde::{Deserialize, Serialize};
use crate::my_imports::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkDelays {
    pub api2scheduler: f64,
    pub scheduler2api: f64,
    pub api2kubelet: f64,
    pub kubelet2api: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeGroup {
    pub amount: u64,
    pub node: Node,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Constants {
    pub kubelet_self_update_period: f64,
    pub scheduler_self_update_period: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterState {
    pub network_delays: NetworkDelays,
    pub nodes: Vec<NodeGroup>,
    pub constants: Constants,
}

impl ClusterState {
    pub fn from_yaml(path: &str) -> Self {
        let s: String = fs::read_to_string(path).expect(format!("Unable to read file: {}", path).as_str());
        let cluster_state: ClusterState = serde_yaml::from_str(s.as_str()).unwrap();
        return cluster_state;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
pub struct PodGroup {
    pub amount: u64,
    pub pod: Pod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkLoad {
    pub pods: Vec<PodGroup>
}

impl WorkLoad {
    pub fn from_yaml(path: &str) -> Self {
        let s: String = fs::read_to_string(path).expect(format!("Unable to read file: {}", path).as_str());
        let workload: WorkLoad = serde_yaml::from_str(s.as_str()).unwrap();
        return workload;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

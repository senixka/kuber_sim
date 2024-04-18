use crate::my_imports::*;


#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkDelays {
    pub api2scheduler: f64,
    pub scheduler2api: f64,
    pub api2kubelet: f64,
    pub kubelet2api: f64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Constants {
    pub kubelet_self_update_period: f64,
    pub monitoring_self_update_period: f64,
    pub scheduler_self_update_period: f64,
    pub scheduler_cycle_max_scheduled: u64,
    pub scheduler_cycle_max_to_try: u64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterState {
    pub network_delays: NetworkDelays,
    pub nodes: Vec<NodeGroup>,
    pub constants: Constants,
}


impl ClusterState {
    pub fn from_yaml(path: &str) -> Self {
        let s: String = fs::read_to_string(path).expect(format!("Unable to read file: {0}", path).as_str());
        let mut cluster_state: ClusterState = serde_yaml::from_str(s.as_str()).unwrap();

        for node_group in &mut cluster_state.nodes {
            node_group.prepare();
        }

        if cluster_state.constants.scheduler_cycle_max_scheduled == 0 {
            cluster_state.constants.scheduler_cycle_max_scheduled = u64::MAX;
        }
        if cluster_state.constants.scheduler_cycle_max_to_try == 0 {
            cluster_state.constants.scheduler_cycle_max_to_try = u64::MAX;
        }

        return cluster_state;
    }
}

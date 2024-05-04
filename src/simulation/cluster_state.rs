use crate::my_imports::*;


#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkDelays {
    pub api2scheduler: f64,
    pub scheduler2api: f64,
    pub api2kubelet: f64,
    pub kubelet2api: f64,
    pub api2ca: f64,
    pub ca2api: f64,
    pub api2hpa: f64,
    pub hpa2api: f64,
    pub api2vpa: f64,
    pub vpa2api: f64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Constants {
    pub kubelet_self_update_period: f64,
    pub monitoring_self_update_period: f64,
    pub scheduler_self_update_period: f64,
    pub scheduler_cycle_max_scheduled: u64,
    pub scheduler_cycle_max_to_try: u64,
    pub unschedulable_queue_period: f64,
    pub ca_self_update_period: f64,
    pub ca_add_node_delay_time: f64,
    pub ca_add_node_min_pending: u64,
    pub ca_remove_node_cpu_percent: f64,
    pub ca_remove_node_memory_percent: f64,
    pub ca_remove_node_delay_cycle: u64,
    pub hpa_self_update_period: f64,
    pub vpa_self_update_period: f64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterState {
    pub network_delays: NetworkDelays,
    pub constants: Constants,

    #[serde(default)]
    pub nodes: Vec<NodeGroup>,

    #[serde(default)]
    pub ca_nodes: Vec<NodeGroup>,
}


impl ClusterState {
    pub fn from_yaml(path: &String) -> Self {
        let s: String = fs::read_to_string(path).expect(format!("Unable to read file: {0}", path).as_str());
        let mut cluster_state: ClusterState = serde_yaml::from_str(s.as_str()).unwrap();

        for node_group in &mut cluster_state.nodes {
            node_group.prepare();
        }

        for node_group in &mut cluster_state.ca_nodes {
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

use std::fs;
use serde::{Deserialize, Serialize};
use crate::my_imports::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkDelays {
    api2scheduler: f64,
    scheduler2api: f64,
    api2kubelet: f64,
    kubelet2api: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodGroup {
    pub amount: u64,
    pub pod: Pod,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeGroup {
    pub amount: u64,
    pub node: Node,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimConfig {
    pub network_delays: NetworkDelays,
    pub pods: Vec<PodGroup>,
    pub nodes: Vec<NodeGroup>,
    pub constants: SimConstants,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimConstants {
    pub kubelet_self_update_period: f64,
}

impl SimConstants {
    pub const fn new() -> Self {
        Self {
            kubelet_self_update_period: 10.0,
        }
    }
}

static mut NETWORK_DELAYS: NetworkDelays = NetworkDelays::new();

static mut SIM_CONFIG: SimConfig = SimConfig {
    network_delays: NetworkDelays::new(),
    constants: SimConstants::new(),
    pods: Vec::new(),
    nodes: Vec::new(),
};

impl SimConfig {
    pub fn from_yaml(path: &str) {
        let s: String = fs::read_to_string(path).expect(format!("Unable to read file: {}", path).as_str());
        let sim_config: SimConfig = serde_yaml::from_str(s.as_str()).unwrap();
        unsafe {
            SIM_CONFIG = sim_config;
        }
    }

    pub fn pods() -> &'static Vec<PodGroup> {
        unsafe {
            &SIM_CONFIG.pods
        }
    }

    pub fn nodes() -> &'static Vec<NodeGroup> {
        unsafe {
            &SIM_CONFIG.nodes
        }
    }

    pub fn kubelet_self_update_period() -> f64 {
        unsafe {
            SIM_CONFIG.constants.kubelet_self_update_period
        }
    }
}

impl NetworkDelays {
    pub const fn new() -> Self {
        Self {
            api2scheduler: 0.0,
            scheduler2api: 0.0,
            api2kubelet: 0.0,
            kubelet2api: 0.0
        }
    }

    pub fn from_yaml(path: &str) {
        let s: String = fs::read_to_string(path).expect(format!("Unable to read file: {}", path).as_str());
        let sim_config: SimConfig = serde_yaml::from_str(s.as_str()).unwrap();
        unsafe {
            NETWORK_DELAYS = sim_config.network_delays;
        }
    }

    pub fn api2scheduler() -> f64 {
        unsafe {
            NETWORK_DELAYS.api2scheduler
        }
    }

    pub fn scheduler2api() -> f64 {
        unsafe {
            NETWORK_DELAYS.scheduler2api
        }
    }

    pub fn api2kubelet() -> f64 {
        unsafe {
            NETWORK_DELAYS.api2kubelet
        }
    }

    pub fn kubelet2api() -> f64 {
        unsafe {
            NETWORK_DELAYS.kubelet2api
        }
    }
}

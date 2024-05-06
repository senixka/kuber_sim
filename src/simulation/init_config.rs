use crate::my_imports::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDelays {
    // Scheduler
    pub api2scheduler: f64,
    pub scheduler2api: f64,

    // Kubelet
    pub api2kubelet: f64,
    pub kubelet2api: f64,

    // CA
    pub api2ca: f64,
    pub ca2api: f64,

    // HPA
    pub api2hpa: f64,
    pub hpa2api: f64,

    // VPA
    pub api2vpa: f64,
    pub vpa2api: f64,
}

impl NetworkDelays {
    pub fn prepare(&mut self) {
        assert!(self.api2scheduler >= 0.0, "NetworkDelays.api2scheduler must be >= 0.0");
        assert!(self.scheduler2api >= 0.0, "NetworkDelays.scheduler2api must be >= 0.0");
        assert!(self.api2kubelet >= 0.0, "NetworkDelays.api2kubelet must be >= 0.0");
        assert!(self.kubelet2api >= 0.0, "NetworkDelays.kubelet2api must be >= 0.0");
        assert!(self.api2ca >= 0.0, "NetworkDelays.api2ca must be >= 0.0");
        assert!(self.ca2api >= 0.0, "NetworkDelays.ca2api must be >= 0.0");
        assert!(self.api2hpa >= 0.0, "NetworkDelays.api2hpa must be >= 0.0");
        assert!(self.hpa2api >= 0.0, "NetworkDelays.hpa2api must be >= 0.0");
        assert!(self.api2vpa >= 0.0, "NetworkDelays.api2vpa must be >= 0.0");
        assert!(self.vpa2api >= 0.0, "NetworkDelays.vpa2api must be >= 0.0");
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMonitoring {
    pub self_update_period: f64,
}

impl ConfigMonitoring {
    pub fn prepare(&mut self) {
        assert!(self.self_update_period > 0.0, "ConfigMonitoring.self_update_period must be > 0.0");
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigScheduler {
    pub unschedulable_queue_backoff_delay: f64,
    pub self_update_period: f64,
    pub cycle_max_scheduled: u64,
    pub cycle_max_to_try: u64,
}

impl ConfigScheduler {
    pub fn prepare(&mut self) {
        assert!(self.self_update_period > 0.0, "ConfigScheduler.self_update_period must be > 0.0");
        assert!(self.unschedulable_queue_backoff_delay >= 0.0, "ConfigScheduler.unschedulable_queue_backoff_delay must be >= 0.0");

        // Zero is special value
        if self.cycle_max_scheduled == 0 {
            self.cycle_max_scheduled = u64::MAX;
        }
        if self.cycle_max_to_try == 0 {
            self.cycle_max_to_try = u64::MAX;
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCA {
    pub self_update_period: f64,

    // Scale up config
    pub add_node_isp_delay: f64,
    pub add_node_min_pending: u64,

    // Scale down config
    pub remove_node_cpu_fraction: f64,
    pub remove_node_memory_fraction: f64,
    pub remove_node_cycle_delay: u64,
}

impl ConfigCA {
    pub fn prepare(&mut self) {
        assert!(self.self_update_period > 0.0, "ConfigCA.self_update_period must be > 0.0");
        assert!(self.add_node_isp_delay >= 0.0, "ConfigCA.add_node_isp_delay must be >= 0.0");
        assert!(0.0 <= self.remove_node_cpu_fraction && self.remove_node_cpu_fraction <= 1.0, "ConfigCA.remove_node_cpu_fraction must be in [0.0, 1.0]");
        assert!(0.0 <= self.remove_node_memory_fraction && self.remove_node_memory_fraction <= 1.0, "ConfigCA.remove_node_memory_fraction must be in [0.0, 1.0]");
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHPA {
    pub self_update_period: f64,
}

impl ConfigHPA {
    pub fn prepare(&mut self) {
        assert!(self.self_update_period > 0.0, "ConfigHPA.self_update_period must be > 0.0");
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVPA {
    pub self_update_period: f64,
}

impl ConfigVPA {
    pub fn prepare(&mut self) {
        assert!(self.self_update_period > 0.0, "ConfigVPA.self_update_period must be > 0.0");
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitConfig {
    pub network_delays: NetworkDelays,
    pub monitoring: ConfigMonitoring,
    pub scheduler: ConfigScheduler,
    pub ca: ConfigCA,
    pub hpa: ConfigHPA,
    pub vpa: ConfigVPA,
}


impl InitConfig {
    pub fn from_yaml(path: &String) -> Self {
        // Read file to string
        let s: String = fs::read_to_string(path).expect(format!("Unable to read file: {0}", path).as_str());
        // Build struct from string
        let mut init_config: InitConfig = serde_yaml::from_str(s.as_str()).unwrap();

        // Prepare init_config
        init_config.prepare();
        return init_config;
    }

    pub fn prepare(&mut self) {
        self.network_delays.prepare();
        self.monitoring.prepare();
        self.scheduler.prepare();
        self.ca.prepare();
        self.hpa.prepare();
        self.vpa.prepare();
    }
}

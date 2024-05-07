use crate::my_imports::*;

/////////////////////////////////////////// NetworkDelays //////////////////////////////////////////

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkDelays {
    // Scheduler
    #[serde(default)]
    pub api2scheduler: f64,
    #[serde(default)]
    pub scheduler2api: f64,

    // Kubelet
    #[serde(default)]
    pub api2kubelet: f64,
    #[serde(default)]
    pub kubelet2api: f64,

    // CA
    #[serde(default)]
    pub api2ca: f64,
    #[serde(default)]
    pub ca2api: f64,

    // HPA
    #[serde(default)]
    pub api2hpa: f64,
    #[serde(default)]
    pub hpa2api: f64,

    // VPA
    #[serde(default)]
    pub api2vpa: f64,
    #[serde(default)]
    pub vpa2api: f64,

    #[serde(skip)]
    pub max_delay: f64,
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

        self.max_delay = self
            .api2scheduler
            .max(self.scheduler2api)
            .max(self.api2kubelet)
            .max(self.kubelet2api)
            .max(self.api2ca)
            .max(self.ca2api)
            .max(self.api2hpa)
            .max(self.hpa2api)
            .max(self.api2vpa)
            .max(self.vpa2api);
    }
}

///////////////////////////////////////// ConfigMonitoring /////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMonitoring {
    pub self_update_period: f64,
}

impl ConfigMonitoring {
    pub fn prepare(&mut self) {
        assert!(
            self.self_update_period > 0.0,
            "ConfigMonitoring.self_update_period must be > 0.0"
        );
    }
}

///////////////////////////////////////// ConfigScheduler //////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigScheduler {
    pub unschedulable_queue_backoff_delay: f64,
    pub self_update_period: f64,
    #[serde(default)]
    pub cycle_max_scheduled: u64,
    #[serde(default)]
    pub cycle_max_to_try: u64,
}

impl ConfigScheduler {
    pub fn prepare(&mut self) {
        assert!(
            self.self_update_period > 0.0,
            "ConfigScheduler.self_update_period must be > 0.0"
        );
        assert!(
            self.unschedulable_queue_backoff_delay >= 0.0,
            "ConfigScheduler.unschedulable_queue_backoff_delay must be >= 0.0"
        );

        // Zero is special value
        if self.cycle_max_scheduled == 0 {
            self.cycle_max_scheduled = u64::MAX;
        }
        if self.cycle_max_to_try == 0 {
            self.cycle_max_to_try = u64::MAX;
        }
    }
}

//////////////////////////////////////////// ConfigCA //////////////////////////////////////////////

/// Analog of --scan-interval
fn ca_self_update_period() -> f64 {
    10.0
}
/// Analog of scale-down-utilization-threshold
fn ca_remove_node_cpu_fraction_default() -> f64 {
    0.5
}
/// Analog of scale-down-utilization-threshold
fn ca_remove_node_memory_fraction_default() -> f64 {
    0.5
}
/// Analog of scale-down-unneeded-time
fn ca_remove_node_cycle_delay_default() -> u64 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCA {
    #[serde(default = "ca_self_update_period")]
    pub self_update_period: f64,

    // Scale up config
    #[serde(default)]
    pub add_node_isp_delay: f64,
    #[serde(default)]
    pub add_node_pending_threshold: u64,

    // Scale down config
    #[serde(default = "ca_remove_node_cpu_fraction_default")]
    pub remove_node_cpu_fraction: f64,
    #[serde(default = "ca_remove_node_memory_fraction_default")]
    pub remove_node_memory_fraction: f64,
    #[serde(default = "ca_remove_node_cycle_delay_default")]
    pub remove_node_cycle_delay: u64,
}

impl Default for ConfigCA {
    fn default() -> Self {
        Self {
            self_update_period: ca_self_update_period(),
            add_node_isp_delay: 0.0,
            add_node_pending_threshold: 0,
            remove_node_cpu_fraction: ca_remove_node_cpu_fraction_default(),
            remove_node_memory_fraction: ca_remove_node_memory_fraction_default(),
            remove_node_cycle_delay: ca_remove_node_cycle_delay_default(),
        }
    }
}

impl ConfigCA {
    pub fn prepare(&mut self) {
        assert!(
            self.self_update_period > 0.0,
            "ConfigCA.self_update_period must be > 0.0"
        );
        assert!(
            self.add_node_isp_delay >= 0.0,
            "ConfigCA.add_node_isp_delay must be >= 0.0"
        );
        assert!(
            0.0 <= self.remove_node_cpu_fraction && self.remove_node_cpu_fraction <= 1.0,
            "ConfigCA.remove_node_cpu_fraction must be in [0.0, 1.0]"
        );
        assert!(
            0.0 <= self.remove_node_memory_fraction && self.remove_node_memory_fraction <= 1.0,
            "ConfigCA.remove_node_memory_fraction must be in [0.0, 1.0]"
        );
    }
}

//////////////////////////////////////////// ConfigHPA /////////////////////////////////////////////

/// Analog of --horizontal-pod-autoscaler-sync-period
fn hpa_self_update_period() -> f64 {
    15.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHPA {
    #[serde(default = "hpa_self_update_period")]
    pub self_update_period: f64,
}

impl Default for ConfigHPA {
    fn default() -> Self {
        Self {
            self_update_period: hpa_self_update_period(),
        }
    }
}

impl ConfigHPA {
    pub fn prepare(&mut self) {
        assert!(
            self.self_update_period > 0.0,
            "ConfigHPA.self_update_period must be > 0.0"
        );
    }
}

//////////////////////////////////////////// ConfigVPA /////////////////////////////////////////////

/// Analog of VPA updater period
fn vpa_self_update_period() -> f64 {
    60.0
}
/// Analog of
fn vpa_reschedule_delay() -> f64 {
    5.0 * 60.0
}
/// Maybe some analogs
fn vpa_histogram_update_frequency() -> f64 {
    1.0
}
/// Analog of
fn vpa_gap_cpu() -> f64 {
    0.15
}
/// Analog of
fn vpa_gap_memory() -> f64 {
    0.15
}
/// Analog of recommendation-margin-fraction
fn vpa_recommendation_margin_fraction() -> f64 {
    0.15
}
/// Analog of
fn vpa_limit_margin_fraction() -> f64 {
    1.1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVPA {
    #[serde(default = "vpa_self_update_period")]
    pub self_update_period: f64,

    #[serde(default = "vpa_reschedule_delay")]
    pub reschedule_delay: f64,
    #[serde(default = "vpa_histogram_update_frequency")]
    pub histogram_update_frequency: f64,

    #[serde(default = "vpa_gap_cpu")]
    pub gap_cpu: f64,
    #[serde(default = "vpa_gap_memory")]
    pub gap_memory: f64,

    #[serde(default = "vpa_recommendation_margin_fraction")]
    pub recommendation_margin_fraction: f64,
    #[serde(default = "vpa_limit_margin_fraction")]
    pub limit_margin_fraction: f64,
}

impl Default for ConfigVPA {
    fn default() -> Self {
        Self {
            self_update_period: vpa_self_update_period(),
            reschedule_delay: vpa_reschedule_delay(),
            histogram_update_frequency: vpa_histogram_update_frequency(),
            gap_cpu: vpa_gap_cpu(),
            gap_memory: vpa_gap_memory(),
            recommendation_margin_fraction: vpa_recommendation_margin_fraction(),
            limit_margin_fraction: vpa_limit_margin_fraction(),
        }
    }
}

impl ConfigVPA {
    pub fn prepare(&mut self) {
        assert!(
            self.self_update_period > 0.0,
            "ConfigVPA.self_update_period must be > 0.0"
        );
        assert!(self.reschedule_delay > 0.0, "ConfigVPA.reschedule_delay must be > 0.0");
        assert!(
            self.histogram_update_frequency > 0.0,
            "ConfigVPA.histogram_update_frequency must be > 0.0"
        );
        assert!(self.gap_cpu >= 0.0, "ConfigVPA.gap_cpu must be > 0.0");
        assert!(self.gap_memory >= 0.0, "ConfigVPA.gap_memory must be > 0.0");
        assert!(
            self.recommendation_margin_fraction >= 0.0,
            "ConfigVPA.recommendation_margin_fraction must be > 0.0"
        );
        assert!(
            self.limit_margin_fraction >= 0.0,
            "ConfigVPA.limit_margin_fraction must be > 0.0"
        );
    }
}

//////////////////////////////////////////// InitConfig ////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitConfig {
    #[serde(default)]
    pub network_delays: NetworkDelays,

    pub monitoring: ConfigMonitoring,
    pub scheduler: ConfigScheduler,

    #[serde(default)]
    pub ca: ConfigCA,
    #[serde(default)]
    pub hpa: ConfigHPA,
    #[serde(default)]
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

use crate::my_imports::*;
use histogram::Histogram;

#[derive(Debug, Clone)]
pub struct VPAPodInfo {
    pub start_time: f64,

    pub last_time: f64,
    pub last_phase: PodPhase,
    pub last_cpu: f64,
    pub last_memory: f64,

    pub baseline_request_cpu: i64,
    pub baseline_request_memory: i64,
    pub baseline_limit_cpu: i64,
    pub baseline_limit_memory: i64,

    pub hist_cpu: Histogram,
    pub hist_memory: Histogram,

    pub is_rescheduled: bool,
}

impl VPAPodInfo {
    pub fn new(pod: &Pod, current_time: f64) -> Self {
        Self {
            start_time: current_time,
            last_time: current_time,
            last_phase: PodPhase::Pending,
            last_cpu: 0.0,
            last_memory: 0.0,

            baseline_request_cpu: pod.spec.request_cpu,
            baseline_request_memory: pod.spec.request_memory,
            baseline_limit_cpu: pod.spec.limit_cpu,
            baseline_limit_memory: pod.spec.limit_memory,

            hist_cpu: Histogram::new(7, 8).unwrap(),
            hist_memory: Histogram::new(7, 8).unwrap(),

            is_rescheduled: false,
        }
    }

    pub fn update_with_metrics(
        &mut self,
        init_config: &InitConfig,
        current_time: f64,
        current_phase: PodPhase,
        current_cpu: f64,
        current_memory: f64,
    ) {
        match self.last_phase {
            PodPhase::Running => {
                match current_phase {
                    PodPhase::Running => {
                        // Changing: Running -> Running

                        // Count past number of hist updates
                        let past =
                            ((current_time - self.last_time) / init_config.vpa.histogram_update_frequency) as u64;

                        // Update cpu hist
                        self.hist_cpu.add(self.last_cpu as u64, past).unwrap();
                        // Update memory hist
                        self.hist_memory.add(self.last_memory as u64, past).unwrap();
                    }
                    PodPhase::Succeeded | PodPhase::Failed | PodPhase::Removed => {
                        // Running -> Finished
                        // Do nothing
                    }
                    PodPhase::Pending | PodPhase::Evicted | PodPhase::Preempted => {
                        // Running -> OnReschedule
                        // Do nothing
                    }
                }
            }
            PodPhase::Pending | PodPhase::Evicted | PodPhase::Preempted => {
                match current_phase {
                    PodPhase::Running => {
                        // Changing: OnReschedule -> Running
                        // TODO: maybe clear hists?
                    }
                    PodPhase::Succeeded | PodPhase::Failed | PodPhase::Removed => {
                        // OnReschedule -> Finished
                        // Do nothing
                    }
                    PodPhase::Pending | PodPhase::Evicted | PodPhase::Preempted => {
                        // OnReschedule -> OnReschedule
                        // Do nothing
                    }
                }
            }
            PodPhase::Succeeded | PodPhase::Failed | PodPhase::Removed => {
                // Finished pod should not get new phase updates
                panic!(
                    "Logic error in HPA metric update. Bad pod phase change:({:?} -> {:?})",
                    self.last_phase, current_phase
                );
            }
        }

        self.last_time = current_time;
        self.last_phase = current_phase;
        self.last_cpu = current_cpu;
        self.last_memory = current_memory;
    }

    pub fn update_with_time(&mut self, init_config: &InitConfig, current_time: f64) {
        if self.last_phase == PodPhase::Running {
            // Count past number of hist updates
            let past = ((current_time - self.last_time) / init_config.vpa.histogram_update_frequency) as u64;

            // Update cpu hist
            self.hist_cpu.add(self.last_cpu as u64, past).unwrap();
            // Update memory hist
            self.hist_memory.add(self.last_memory as u64, past).unwrap();
        }

        // Update last time
        self.last_time = current_time;
    }

    pub fn is_failed(&self) -> bool {
        return self.last_phase == PodPhase::Failed;
    }

    pub fn is_finished(&self) -> bool {
        return self.last_phase == PodPhase::Failed
            || self.last_phase == PodPhase::Succeeded
            || self.last_phase == PodPhase::Removed;
    }

    pub fn suggest(&self, init_config: &InitConfig, profile: &VPAProfile) -> (i64, i64, i64, i64) {
        // Get cpu percentiles
        let cpu_data = self.hist_cpu.percentiles(&[50.0, 90.0, 95.0]).unwrap();
        assert_eq!(cpu_data.len(), 3);
        dp_vpa!("VPA Suggest cpu percentiles: {:?}", cpu_data);

        // Count cpu absolute values from baseline and given percent
        let _lower_bound_cpu = ((self.baseline_request_cpu as f64
            * cpu_data[0].0
            * init_config.vpa.recommendation_margin_fraction) as i64)
            .max(profile.min_allowed_cpu)
            .min(profile.max_allowed_cpu);
        let target_cpu = ((self.baseline_request_cpu as f64
            * cpu_data[1].0
            * init_config.vpa.recommendation_margin_fraction) as i64)
            .max(profile.min_allowed_cpu)
            .min(profile.max_allowed_cpu);
        let _upper_bound_cpu = ((self.baseline_request_cpu as f64
            * cpu_data[2].0
            * init_config.vpa.recommendation_margin_fraction) as i64)
            .max(profile.min_allowed_cpu)
            .min(profile.max_allowed_cpu);

        // Get memory percentiles
        let memory_data = self.hist_memory.percentiles(&[50.0, 90.0, 95.0]).unwrap();
        assert_eq!(memory_data.len(), 3);
        dp_vpa!("VPA Suggest memory percentiles: {:?}", memory_data);

        // Count memory absolute values from baseline and given percent
        let _lower_bound_memory = ((self.baseline_request_memory as f64
            * memory_data[0].0
            * init_config.vpa.recommendation_margin_fraction) as i64)
            .max(profile.min_allowed_memory)
            .min(profile.max_allowed_memory);
        let target_memory = ((self.baseline_request_memory as f64
            * memory_data[1].0
            * init_config.vpa.recommendation_margin_fraction) as i64)
            .max(profile.min_allowed_memory)
            .min(profile.max_allowed_memory);
        let _upper_bound_memory = ((self.baseline_request_memory as f64
            * memory_data[2].0
            * init_config.vpa.recommendation_margin_fraction) as i64)
            .max(profile.min_allowed_memory)
            .min(profile.max_allowed_memory);

        dp_vpa!("########### VPA Suggests #########");
        dp_vpa!("# Lower CPU:     {:<15} #", _lower_bound_cpu);
        dp_vpa!("# Lower Memory:  {:<15} #", _lower_bound_memory);
        dp_vpa!("# ------------------------------ #");
        dp_vpa!("# Target CPU:    {:<15} #", target_cpu);
        dp_vpa!("# Target Memory: {:<15} #", target_memory);
        dp_vpa!("# ------------------------------ #");
        dp_vpa!("# Upper CPU:     {:<15} #", _upper_bound_cpu);
        dp_vpa!("# Upper Memory:  {:<15} #", _upper_bound_memory);
        dp_vpa!("##################################");

        return (
            target_cpu,
            target_memory,
            (target_cpu as f64 * init_config.vpa.limit_margin_fraction) as i64,
            (target_memory as f64 * init_config.vpa.limit_margin_fraction) as i64,
        );
    }

    pub fn need_reschedule(&self, init_config: &InitConfig, profile: &VPAProfile, current_time: f64) -> bool {
        if current_time - self.start_time <= init_config.vpa.reschedule_delay {
            return false;
        }

        // Get suggested spec resources
        let (request_cpu, request_memory, _, _) = self.suggest(init_config, profile);

        // Compare suggested with current requested resources
        let d_cpu: f64 = ((request_cpu as f64 / self.baseline_request_cpu as f64) - 1.0).abs();
        let d_memory: f64 = ((request_memory as f64 / self.baseline_request_memory as f64) - 1.0).abs();

        return d_cpu >= init_config.vpa.gap_cpu || d_memory >= init_config.vpa.gap_memory;
    }
}

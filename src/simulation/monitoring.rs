use super::super::my_imports::*;


pub struct Monitoring {
    pub ctx: dsc::SimulationContext,
    pub self_update_enabled: bool,
    pub cluster_state: Rc<RefCell<ClusterState>>,

    makespan_time: f64,

    total_installed_cpu: u64,
    total_installed_memory: u64,

    scheduler_used_cpu: u64,
    scheduler_used_memory: u64,

    kubelets_used_cpu: u64,
    kubelets_used_memory: u64,

    // utilization_cpu_numerator: Vec<u64>,
    // utilization_memory_numerator: Vec<u64>,
    // utilization_measurements_time: Vec<f64>,

    // pod_start_time: HashMap<u64, f64>,
    // pod_end_time: HashMap<u64, f64>,

    // pod_unfinished_task_count: HashMap<u64, u64>,
    // pod_ideal_estimate_time: HashMap<u64, u64>,

    // pending_pod: Vec<u64>,
    // working_pod: Vec<u64>,
    
    n_pod_in_simulation: u64, // const
    
    pending_pod_counter: usize,
    // running_pod_counter: u64,
    finished_pod_counter: u64,

    // max_pending_pod: u64,
    // max_running_pod: u64,
}

impl Monitoring {
    pub fn new(ctx: dsc::SimulationContext, cluster_state: Rc<RefCell<ClusterState>>) -> Self {
        Self {
            ctx,
            self_update_enabled: false,
            cluster_state,
            makespan_time: 0.0, total_installed_cpu: 0, total_installed_memory: 0,
            scheduler_used_cpu: 0, scheduler_used_memory: 0,
            kubelets_used_cpu: 0, kubelets_used_memory: 0,
            n_pod_in_simulation: 0, finished_pod_counter: 0, pending_pod_counter: 0,
        }
    }

    pub fn presimulation_init(&mut self) {
        if !self.self_update_enabled {
            self.self_update_enabled = true;
            self.ctx.emit_self(APIMonitoringSelfUpdate {}, self.cluster_state.borrow().constants.monitoring_self_update_period);
        }
    }

    pub fn presimulation_check(&mut self) {
        assert_eq!(self.self_update_enabled, true);
    }

    pub fn set_n_pod_in_simulation(&mut self, n_pod_in_simulation: u64) {
        self.n_pod_in_simulation = n_pod_in_simulation;
    }

    pub fn scheduler_on_node_consume(&mut self, cpu: u64, memory: u64) {
        self.scheduler_used_cpu += cpu;
        self.scheduler_used_memory += memory;
    }

    pub fn scheduler_on_node_restore(&mut self, cpu: u64, memory: u64) {
        assert!(self.scheduler_used_cpu >= cpu);
        assert!(self.scheduler_used_memory >= memory);

        self.scheduler_used_cpu -= cpu;
        self.scheduler_used_memory -= memory;
    }

    pub fn scheduler_on_node_added(&mut self, node: &Node) {
        assert_eq!(node.spec.available_cpu, node.spec.installed_cpu);
        assert_eq!(node.spec.available_memory, node.spec.installed_memory);

        self.total_installed_cpu += node.spec.installed_cpu;
        self.total_installed_memory += node.spec.installed_memory;
    }

    pub fn scheduler_update_pending_pod_count(&mut self, count: usize) {
        self.pending_pod_counter = count;
    }

    pub fn kubelet_on_pod_placed(&mut self, cpu: u64, memory: u64) {
        self.kubelets_used_cpu += cpu;
        self.kubelets_used_memory += memory;
    }

    pub fn kubelet_on_pod_unplaced(&mut self, cpu: u64, memory: u64) {
        assert!(self.kubelets_used_cpu >= cpu);
        assert!(self.kubelets_used_memory >= memory);

        self.kubelets_used_cpu -= cpu;
        self.kubelets_used_memory -= memory;
    }

    pub fn kubelet_on_pod_finished(&mut self) {
        self.finished_pod_counter += 1;
        if self.finished_pod_counter == self.n_pod_in_simulation {
            self.self_update_enabled = false;
            self.makespan_time = self.ctx.time();
        }
        // println!("{:?} Finished pod", self.ctx.time());
    }

    pub fn print_statistics(&self) {
        print!(
            "{:12.1}  CPU: {:7.3}% / {:7.3}%  Memory: {:7.3}% / {:7.3}%  Pod finished: {:>9} / {:?}  Pending: {:?}\n",
            self.ctx.time(),
            (self.kubelets_used_cpu as f64) / (self.total_installed_cpu as f64) * 100.0f64,
            (self.scheduler_used_cpu as f64) / (self.total_installed_cpu as f64) * 100.0f64,
            (self.kubelets_used_memory as f64) / (self.total_installed_memory as f64) * 100.0f64,
            (self.scheduler_used_memory as f64) / (self.total_installed_memory as f64) * 100.0f64,
            self.finished_pod_counter, self.n_pod_in_simulation, self.pending_pod_counter,
        );
        // print!("  Job submitted: %lu / %lu  Task finished: %lu / %lu", jobSubmittedCounter_, nJobInSimulation_, taskFinishedCounter_, nTaskInSimulation_);
        // print!("  Task working: %lu  Task pending: %lu  Current Job: %lu\n", currentWorkingTaskCounter_, currentPendingTaskCounter_, currentUnfinishedJobCounter_);
    }
}

impl dsc::EventHandler for Monitoring {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            APIMonitoringSelfUpdate { } => {
                self.print_statistics();
                if self.self_update_enabled {
                    self.ctx.emit_self(APIMonitoringSelfUpdate {}, self.cluster_state.borrow().constants.monitoring_self_update_period);
                }
            }
        });
    }
}

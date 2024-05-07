use super::super::my_imports::*;
use std::io::Write;

pub struct Monitoring {
    pub ctx: dsc::SimulationContext,
    pub self_update_enabled: bool,
    pub dynamic_update_enabled: bool,
    pub init_config: Rc<RefCell<InitConfig>>,

    total_installed_cpu: i64,
    total_installed_memory: i64,

    scheduler_used_cpu: i64,
    scheduler_used_memory: i64,

    kubelets_used_cpu: i64,
    kubelets_used_memory: i64,

    scheduler_utilization_cpu_numerator: Vec<i64>,
    scheduler_utilization_memory_numerator: Vec<i64>,
    kubelet_utilization_cpu_numerator: Vec<i64>,
    kubelet_utilization_memory_numerator: Vec<i64>,
    // utilization_measurements_time: Vec<f64>,

    // pod_start_time: HashMap<u64, f64>,
    // pod_end_time: HashMap<u64, f64>,

    // pod_unfinished_task_count: HashMap<u64, u64>,
    // pod_ideal_estimate_time: HashMap<u64, u64>,
    pending_pod: Vec<usize>,
    // working_pod: Vec<u64>,
    pending_pod_counter: usize,
    running_pod_counter: usize,
    succeed_pod_counter: u64,
    failed_pod_counter: u64,
    evicted_pod_counter: u64,
    removed_pod_counter: u64,
    preempted_pod_counter: u64,

    node_counter: u64,

    // max_pending_pod: u64,
    // max_running_pod: u64,
    out_path: String,
}

impl Monitoring {
    pub fn new(ctx: dsc::SimulationContext, init_config: Rc<RefCell<InitConfig>>, out_path: &String) -> Self {
        Self {
            ctx,
            self_update_enabled: false,
            dynamic_update_enabled: false,
            init_config,
            total_installed_cpu: 0,
            total_installed_memory: 0,
            scheduler_used_cpu: 0,
            scheduler_used_memory: 0,
            kubelets_used_cpu: 0,
            kubelets_used_memory: 0,
            succeed_pod_counter: 0,
            pending_pod_counter: 0,
            preempted_pod_counter: 0,
            failed_pod_counter: 0,
            running_pod_counter: 0,
            evicted_pod_counter: 0,
            removed_pod_counter: 0,
            node_counter: 0,
            kubelet_utilization_cpu_numerator: Vec::new(),
            kubelet_utilization_memory_numerator: Vec::new(),
            scheduler_utilization_cpu_numerator: Vec::new(),
            scheduler_utilization_memory_numerator: Vec::new(),
            pending_pod: Vec::new(),
            out_path: out_path.clone(),
        }
    }

    pub fn prepare(&mut self) {
        if !self.self_update_enabled {
            self.self_update_enabled = true;
            self.ctx.emit_self(
                EventSelfUpdate {},
                self.init_config.borrow().monitoring.self_update_period,
            );
        }
    }

    pub fn presimulation_check(&mut self) {
        assert_eq!(self.self_update_enabled, true);
    }

    pub fn enable_dynamic_update(&mut self) {
        self.dynamic_update_enabled = true;
    }

    pub fn disable_dynamic_update(&mut self) {
        self.dynamic_update_enabled = false;
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn scheduler_on_node_consume(&mut self, cpu: i64, memory: i64) {
        self.scheduler_used_cpu += cpu;
        self.scheduler_used_memory += memory;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn scheduler_on_node_restore(&mut self, cpu: i64, memory: i64) {
        assert!(self.scheduler_used_cpu >= cpu);
        assert!(self.scheduler_used_memory >= memory);

        self.scheduler_used_cpu -= cpu;
        self.scheduler_used_memory -= memory;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn scheduler_on_node_added(&mut self, node: &Node) {
        assert_eq!(node.spec.available_cpu, node.spec.installed_cpu);
        assert_eq!(node.spec.available_memory, node.spec.installed_memory);

        self.total_installed_cpu += node.spec.installed_cpu;
        self.total_installed_memory += node.spec.installed_memory;

        self.node_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn scheduler_on_node_removed(&mut self, node: &Node) {
        self.scheduler_on_node_restore(
            node.spec.installed_cpu - node.spec.available_cpu,
            node.spec.installed_memory - node.spec.available_memory,
        );

        assert!(self.total_installed_cpu >= node.spec.installed_cpu);
        assert!(self.total_installed_memory >= node.spec.installed_memory);
        assert!(self.node_counter > 0);

        self.total_installed_cpu -= node.spec.installed_cpu;
        self.total_installed_memory -= node.spec.installed_memory;

        self.node_counter -= 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn scheduler_update_pending_pod_count(&mut self, count: usize) {
        self.pending_pod_counter = count;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn scheduler_update_running_pod_count(&mut self, count: usize) {
        self.running_pod_counter = count;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn scheduler_on_pod_succeed(&mut self) {
        self.succeed_pod_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn scheduler_on_pod_failed(&mut self) {
        self.failed_pod_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn scheduler_on_pod_removed(&mut self) {
        self.removed_pod_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn scheduler_on_pod_evicted(&mut self) {
        self.evicted_pod_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn scheduler_on_pod_preempted(&mut self) {
        self.preempted_pod_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn kubelet_on_pod_placed(&mut self, cpu: i64, memory: i64) {
        self.kubelets_used_cpu += cpu;
        self.kubelets_used_memory += memory;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    pub fn kubelet_on_pod_unplaced(&mut self, cpu: i64, memory: i64) {
        self.kubelets_used_cpu -= cpu;
        self.kubelets_used_memory -= memory;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn print_statistics(&mut self) {
        self.kubelet_utilization_cpu_numerator.push(self.kubelets_used_cpu);
        self.kubelet_utilization_memory_numerator
            .push(self.kubelets_used_memory);
        self.scheduler_utilization_cpu_numerator.push(self.scheduler_used_cpu);
        self.scheduler_utilization_memory_numerator
            .push(self.scheduler_used_memory);

        self.pending_pod.push(self.pending_pod_counter);
        print!(
            "{:.12}  CPU: {:7.3}% / {:7.3}%  Memory: {:7.3}% / {:7.3}%  Nodes:{:<9} Pending:{:<9} Running:{:<9} Succeed:{:<9} Failed:{:<9} Removed:{:<9} Evicted:{:<9} Preempted:{:<9}\n",
            self.ctx.time(),
            (self.kubelets_used_cpu as f64) / (self.total_installed_cpu as f64) * 100.0f64,
            (self.scheduler_used_cpu as f64) / (self.total_installed_cpu as f64) * 100.0f64,
            (self.kubelets_used_memory as f64) / (self.total_installed_memory as f64) * 100.0f64,
            (self.scheduler_used_memory as f64) / (self.total_installed_memory as f64) * 100.0f64,
            self.node_counter,
            self.pending_pod_counter,
            self.running_pod_counter,
            self.succeed_pod_counter,
            self.failed_pod_counter,
            self.removed_pod_counter,
            self.evicted_pod_counter,
            self.preempted_pod_counter,
        );
    }

    pub fn dump_statistics(&self) {
        let file = File::create(self.out_path.clone()).unwrap();
        file.set_len(0).unwrap();
        let mut fout = BufWriter::new(file);

        for i in 0..self.kubelet_utilization_cpu_numerator.len() {
            write!(
                fout,
                "{:?} {:?} {:?} {:?} {:?}\n",
                self.kubelet_utilization_cpu_numerator[i],
                self.kubelet_utilization_memory_numerator[i],
                self.scheduler_utilization_cpu_numerator[i],
                self.scheduler_utilization_memory_numerator[i],
                self.pending_pod[i]
            )
            .unwrap();
        }
    }
}

impl dsc::EventHandler for Monitoring {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            EventSelfUpdate {} => {
                self.print_statistics();

                if self.self_update_enabled {
                    self.ctx.emit_self(
                        EventSelfUpdate {},
                        self.init_config.borrow().monitoring.self_update_period,
                    );
                }
            }
        });
    }
}

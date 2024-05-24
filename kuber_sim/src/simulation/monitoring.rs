use super::super::common_imports::*;
use crate::api_server::events::*;
use crate::objects::node::Node;
use crate::simulation::init_config::InitConfig;
use std::cell::RefCell;
use std::io::{BufWriter, Write};
use std::rc::Rc;

pub struct Monitoring {
    pub ctx: dsc::SimulationContext,
    pub self_update_enabled: bool,
    pub dynamic_update_enabled: bool,
    pub print_enabled: bool,
    pub init_config: Rc<RefCell<InitConfig>>,

    time_record: Vec<f64>,

    node_counter: u64,

    node_counter_record: Vec<u64>,

    total_installed_cpu: i64,
    total_installed_memory: i64,

    total_installed_cpu_record: Vec<i64>,
    total_installed_memory_record: Vec<i64>,

    scheduler_used_cpu: i64,
    scheduler_used_memory: i64,
    kubelets_used_cpu: i64,
    kubelets_used_memory: i64,

    scheduler_used_cpu_record: Vec<i64>,
    scheduler_used_memory_record: Vec<i64>,
    kubelets_used_cpu_record: Vec<i64>,
    kubelets_used_memory_record: Vec<i64>,

    pending_pod_counter: usize,
    running_pod_counter: usize,
    succeed_pod_counter: u64,
    failed_pod_counter: u64,
    evicted_pod_counter: u64,
    removed_pod_counter: u64,
    preempted_pod_counter: u64,

    pending_pod_counter_record: Vec<usize>,
    running_pod_counter_record: Vec<usize>,
    succeed_pod_counter_record: Vec<u64>,
    failed_pod_counter_record: Vec<u64>,
    evicted_pod_counter_record: Vec<u64>,
    removed_pod_counter_record: Vec<u64>,
    preempted_pod_counter_record: Vec<u64>,

    out_path_prefix: String,
}

impl Monitoring {
    pub fn new(ctx: dsc::SimulationContext, init_config: Rc<RefCell<InitConfig>>, out_path_prefix: &String) -> Self {
        Self {
            ctx,
            self_update_enabled: false,
            dynamic_update_enabled: false,
            print_enabled: true,
            init_config,
            total_installed_cpu: 0,
            total_installed_memory: 0,
            total_installed_cpu_record: vec![],
            total_installed_memory_record: vec![],
            scheduler_used_cpu: 0,
            scheduler_used_memory: 0,
            kubelets_used_cpu: 0,
            kubelets_used_memory: 0,
            scheduler_used_cpu_record: vec![],
            scheduler_used_memory_record: vec![],
            kubelets_used_cpu_record: vec![],
            succeed_pod_counter: 0,
            pending_pod_counter: 0,
            preempted_pod_counter: 0,
            pending_pod_counter_record: vec![],
            running_pod_counter_record: vec![],
            succeed_pod_counter_record: vec![],
            failed_pod_counter_record: vec![],
            evicted_pod_counter_record: vec![],
            removed_pod_counter_record: vec![],
            failed_pod_counter: 0,
            running_pod_counter: 0,
            evicted_pod_counter: 0,
            removed_pod_counter: 0,
            node_counter: 0,
            out_path_prefix: out_path_prefix.clone(),
            node_counter_record: vec![],
            kubelets_used_memory_record: vec![],
            preempted_pod_counter_record: vec![],
            time_record: vec![],
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

    pub fn enable_print(&mut self) {
        self.print_enabled = true;
    }

    pub fn disable_print(&mut self) {
        self.print_enabled = false;
    }

    pub fn clear_records(&mut self) {
        self.time_record.clear();

        self.node_counter_record.clear();

        self.total_installed_cpu_record.clear();
        self.total_installed_memory_record.clear();

        self.scheduler_used_cpu_record.clear();
        self.scheduler_used_memory_record.clear();
        self.kubelets_used_cpu_record.clear();
        self.kubelets_used_memory_record.clear();

        self.pending_pod_counter_record.clear();
        self.running_pod_counter_record.clear();
        self.succeed_pod_counter_record.clear();
        self.failed_pod_counter_record.clear();
        self.evicted_pod_counter_record.clear();
        self.removed_pod_counter_record.clear();
        self.preempted_pod_counter_record.clear();
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    #[inline(always)]
    pub fn scheduler_on_node_consume(&mut self, cpu: i64, memory: i64) {
        self.scheduler_used_cpu += cpu;
        self.scheduler_used_memory += memory;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    #[inline(always)]
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

    #[inline(always)]
    pub fn scheduler_update_pending_pod_count(&mut self, count: usize) {
        self.pending_pod_counter = count;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    #[inline(always)]
    pub fn scheduler_update_running_pod_count(&mut self, count: usize) {
        self.running_pod_counter = count;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    #[inline(always)]
    pub fn scheduler_on_pod_succeed(&mut self) {
        self.succeed_pod_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    #[inline(always)]
    pub fn scheduler_on_pod_failed(&mut self) {
        self.failed_pod_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    #[inline(always)]
    pub fn scheduler_on_pod_removed(&mut self) {
        self.removed_pod_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    #[inline(always)]
    pub fn scheduler_on_pod_evicted(&mut self) {
        self.evicted_pod_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    #[inline(always)]
    pub fn scheduler_on_pod_preempted(&mut self) {
        self.preempted_pod_counter += 1;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    #[inline(always)]
    pub fn kubelet_on_pod_placed(&mut self, cpu: i64, memory: i64) {
        self.kubelets_used_cpu += cpu;
        self.kubelets_used_memory += memory;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    #[inline(always)]
    pub fn kubelet_on_pod_unplaced(&mut self, cpu: i64, memory: i64) {
        self.kubelets_used_cpu -= cpu;
        self.kubelets_used_memory -= memory;

        if self.dynamic_update_enabled {
            self.print_statistics();
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////

    pub fn print_statistics(&mut self) {
        self.time_record.push(self.ctx.time());

        self.node_counter_record.push(self.node_counter);

        self.total_installed_cpu_record.push(self.total_installed_cpu);
        self.total_installed_memory_record.push(self.total_installed_memory);

        self.scheduler_used_cpu_record.push(self.scheduler_used_cpu);
        self.scheduler_used_memory_record.push(self.scheduler_used_memory);
        self.kubelets_used_cpu_record.push(self.kubelets_used_cpu);
        self.kubelets_used_memory_record.push(self.kubelets_used_memory);

        self.pending_pod_counter_record.push(self.pending_pod_counter);
        self.running_pod_counter_record.push(self.running_pod_counter);
        self.succeed_pod_counter_record.push(self.succeed_pod_counter);
        self.failed_pod_counter_record.push(self.failed_pod_counter);
        self.evicted_pod_counter_record.push(self.evicted_pod_counter);
        self.removed_pod_counter_record.push(self.removed_pod_counter);
        self.preempted_pod_counter_record.push(self.preempted_pod_counter);

        if self.print_enabled {
            print!(
                "{:>6.3}  CPU: {:7.3}% / {:7.3}%  Memory: {:7.3}% / {:7.3}%  Nodes:{:<9} Pending:{:<9} Running:{:<9} Succeed:{:<9} Failed:{:<9} Removed:{:<9} Evicted:{:<9} Preempted:{:<9}\n",
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
    }

    pub fn dump_statistics(&self) {
        let mut file = None;
        let mut counter: usize = 0;
        while file.is_none() {
            match std::fs::File::create_new(self.out_path_prefix.clone() + "_" + &*counter.to_string() + ".csv") {
                Ok(new_file) => file = Some(new_file),
                Err(_) => counter += 1,
            }
        }
        let mut fout = BufWriter::new(file.unwrap());

        write!(
            fout,
            "time,nodes,total_cpu,total_memory,scheduler_used_cpu,scheduler_used_memory,kubelets_used_cpu,kubelets_used_memory,pending,running,succeed,failed,evicted,removed,preempted\n"
        ).unwrap();

        for i in 0..self.time_record.len() {
            write!(
                fout,
                "{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?},{:?}\n",
                self.time_record[i],
                self.node_counter_record[i],
                self.total_installed_cpu_record[i],
                self.total_installed_memory_record[i],
                self.scheduler_used_cpu_record[i],
                self.scheduler_used_memory_record[i],
                self.kubelets_used_cpu_record[i],
                self.kubelets_used_memory_record[i],
                self.pending_pod_counter_record[i],
                self.running_pod_counter_record[i],
                self.succeed_pod_counter_record[i],
                self.failed_pod_counter_record[i],
                self.evicted_pod_counter_record[i],
                self.removed_pod_counter_record[i],
                self.preempted_pod_counter_record[i],
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

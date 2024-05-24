use crate::api_server::events::*;
use crate::common_imports::*;
use crate::dp_ca;
use crate::kubelet::kubelet::Kubelet;
use crate::objects::node::Node;
use crate::objects::node_group::NodeGroup;
use crate::scheduler::scheduler::Scheduler;
use crate::simulation::init_config::InitConfig;
use crate::simulation::init_nodes::InitNodes;
use crate::simulation::monitoring::Monitoring;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

/// The component of the Kubernetes responsible for cluster autoscaling.
pub struct CA {
    /// DSLab-Core simulation context of CA.
    ctx: dsc::SimulationContext,
    /// Configuration constants of components in simulation.
    init_config: Rc<RefCell<InitConfig>>,
    /// Scheduler.
    scheduler: Rc<RefCell<Scheduler>>,
    /// API-Server simulation DSlab-Core Id.
    api_sim_id: dsc::Id,

    /// Is CA turned on
    is_turned_on: bool,

    /// Pool of free kubelets
    kubelet_pool: Vec<(dsc::Id, Rc<RefCell<Kubelet>>)>, // Vec<(kubelet_sim_id, kubelet)>
    /// Free nodes in each node group
    free_nodes_by_group: BTreeMap<u64, NodeGroup>, // BTreeMap<group_uid, node_group>
    /// Currently used nodes
    used_nodes: BTreeMap<u64, (dsc::Id, Rc<RefCell<Kubelet>>, u64)>, // BTreeMap<node_uid, (kubelet_sim_id, kubelet, group_uid)>
    /// Node candidates on removal
    low_utilization: BTreeMap<u64, u64>, // BTreeMap<node_uid, cycle_counter>
}

impl CA {
    pub fn new(
        sim: &mut dsc::Simulation,
        ctx: dsc::SimulationContext,
        init_config: Rc<RefCell<InitConfig>>,
        init_nodes: Rc<RefCell<InitNodes>>,
        scheduler: Rc<RefCell<Scheduler>>,
        monitoring: Rc<RefCell<Monitoring>>,
        api_sim_id: dsc::Id,
    ) -> Self {
        let mut ca = Self {
            ctx,
            init_config: init_config.clone(),
            scheduler: scheduler.clone(),
            api_sim_id,

            // CA is created in turned off state
            is_turned_on: false,

            // Inner state
            kubelet_pool: Vec::new(),
            free_nodes_by_group: BTreeMap::new(),
            used_nodes: BTreeMap::new(),
            low_utilization: BTreeMap::new(),
        };

        // Prepare kubelet pool from nodes_config ca nodes
        let mut uid_counter = 0;
        for group in init_nodes.borrow().ca_nodes.iter() {
            // Process node group
            for _ in 0..group.amount {
                // Create kubelet unique name
                uid_counter += 1;
                let name = "kubelet_ca_".to_owned() + &*uid_counter.to_string();

                // Create and register kubelet in simulation
                let kubelet = Rc::new(RefCell::new(Kubelet::new(
                    sim.create_context(name.clone()),
                    init_config.clone(),
                    monitoring.clone(),
                    api_sim_id,
                    Node::default(),
                )));
                let kubelet_id = sim.add_handler(name, kubelet.clone());

                // Add kubelet to pool
                ca.kubelet_pool.push((kubelet_id, kubelet));
            }

            // Add full group to free nodes
            ca.free_nodes_by_group.insert(group.group_uid, group.clone());

            // Prepare group's node to get correct values of available resources
            ca.free_nodes_by_group
                .get_mut(&group.group_uid)
                .unwrap()
                .node
                .prepare(group.group_uid);
        }

        return ca;
    }

    ////////////////// CA Turn On/Off //////////////////

    pub fn turn_on(&mut self) {
        if !self.is_turned_on {
            self.is_turned_on = true;
            self.ctx
                .emit_self(EventSelfUpdate {}, self.init_config.borrow().ca.self_update_period);
        }
    }

    pub fn turn_off(&mut self) {
        if self.is_turned_on {
            self.is_turned_on = false;
            self.ctx
                .cancel_heap_events(|x| x.src == self.ctx.id() && x.dst == self.ctx.id());
        }
    }

    ////////////////// Process metrics //////////////////

    pub fn process_metrics(
        &mut self,
        pending_pod_count: u64,
        used_nodes_utilization: &Vec<(u64, f64, f64)>,
        may_help: Option<u64>,
    ) {
        // Remove nodes with low utilization
        let (min_cpu, min_memory) = (
            self.init_config.borrow().ca.remove_node_cpu_fraction,
            self.init_config.borrow().ca.remove_node_memory_fraction,
        );
        for &(node_uid, cpu, memory) in used_nodes_utilization {
            // If utilization is not low -> remove its cycle count and skip
            if cpu > min_cpu || memory > min_memory {
                self.low_utilization.remove(&node_uid);
                continue;
            }

            // If node is not in used -> remove its cycle count and skip
            if !self.used_nodes.contains_key(&node_uid) {
                self.low_utilization.remove(&node_uid);
                continue;
            }

            // Increase cycle count with low utilization for this node
            let cycles = self.low_utilization.entry(node_uid).or_default();
            *cycles += 1;

            // If cycle count more than remove threshold
            if *cycles >= self.init_config.borrow().ca.remove_node_cycle_delay {
                dp_ca!(
                    "{:.3} ca Issues EventRemoveNode node_uid:{:?}",
                    self.ctx.time(),
                    node_uid
                );
                self.ctx.emit(
                    EventRemoveNode { node_uid },
                    self.api_sim_id,
                    self.init_config.borrow().network_delays.ca2api,
                );
            }
        }

        // If it's not enough pending pods to start up-scaling -> return
        if pending_pod_count <= self.init_config.borrow().ca.add_node_pending_threshold {
            return;
        }

        match may_help {
            None => {
                // If no one node could help -> return
                return;
            }
            Some(group_uid) => {
                // Locate node group
                let group = self.free_nodes_by_group.get_mut(&group_uid).unwrap();

                // Use one node from group
                assert!(group.amount > 0);
                let mut node = group.node.clone();
                group.amount -= 1;

                // Prepare node
                node.prepare(group_uid);

                // Take kubelet from pool
                let (kubelet_sim_id, kubelet) = self.kubelet_pool.pop().unwrap();
                // Replace node in kubelet
                kubelet.borrow_mut().replace_node(&node);
                // Turn on kubelet
                kubelet.borrow_mut().turn_on();

                // Update used nodes
                self.used_nodes
                    .insert(node.metadata.uid, (kubelet_sim_id, kubelet, group_uid));

                // Emit AddNode event
                dp_ca!(
                    "{:.3} ca node:{:?} added -> cluster",
                    self.ctx.time(),
                    node.metadata.uid
                );
                self.ctx.emit(
                    EventAddNode { kubelet_sim_id, node },
                    self.api_sim_id,
                    self.init_config.borrow().network_delays.ca2api + self.init_config.borrow().ca.add_node_isp_delay,
                );
            }
        }
    }

    pub fn update_metrics(&mut self) {
        let available_nodes: Vec<NodeGroup> = self
            .free_nodes_by_group
            .iter()
            .filter_map(|(_, group)| if group.amount > 0 { Some(group.clone()) } else { None })
            .collect();

        // Get view on scheduler
        let scheduler = self.scheduler.borrow();

        // For pending pods look available node which may help
        let mut may_help: Option<u64> = None;
        let mut pending_pod_count = 0;
        for (_, pod) in scheduler.pending_pods.iter() {
            // Only pods which cannot be scheduled due to insufficient resources on nodes
            if !pod.status.cluster_resource_starvation {
                continue;
            }
            pending_pod_count += 1;

            // Try to find node which may help
            if may_help.is_none() {
                for node_group in available_nodes.iter() {
                    if node_group
                        .node
                        .is_both_consumable(pod.spec.request_cpu, pod.spec.request_memory)
                    {
                        may_help = Some(node_group.group_uid);
                        break;
                    }
                }
            }
        }

        // Count utilization for requested nodes
        let mut used_nodes_utilization: Vec<(u64, f64, f64)> = Vec::with_capacity(self.used_nodes.len());
        for node_uid in self.used_nodes.keys() {
            let node = scheduler.nodes.get(node_uid);
            if node.is_some() {
                let spec = node.unwrap().spec.clone();
                let cpu: f64 = ((spec.installed_cpu - spec.available_cpu) as f64) / (spec.installed_cpu as f64);
                let memory: f64 =
                    ((spec.installed_memory - spec.available_memory) as f64) / (spec.installed_memory as f64);

                used_nodes_utilization.push((*node_uid, cpu, memory));
            }
        }
        drop(scheduler);

        // Process scheduler metrics
        self.process_metrics(pending_pod_count, &used_nodes_utilization, may_help);
    }
}

impl dsc::EventHandler for CA {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            EventTurnOn {} => {
                dp_ca!("{:.3} ca EventTurnOn", self.ctx.time());

                self.turn_on();
            }

            EventTurnOff {} => {
                dp_ca!("{:.3} ca EventTurnOff", self.ctx.time());

                self.turn_off();
            }

            EventSelfUpdate {} => {
                dp_ca!("{:.3} ca EventSelfUpdate", self.ctx.time());

                assert!(self.is_turned_on, "Logic error. Self update should be canceled for CA.");

                // Emulate metrics request
                self.ctx
                    .emit_self(EventUpdateCAMetrics {}, self.init_config.borrow().network_delays.ca2api);

                // Emit Self-Update
                self.ctx
                    .emit_self(EventSelfUpdate {}, self.init_config.borrow().ca.self_update_period);
            }

            EventUpdateCAMetrics {} => {
                dp_ca!("{:.3} ca EventUpdateCAMetrics", self.ctx.time());

                // Update with scheduler metrics
                self.update_metrics();
            }

            EventRemoveNodeAck { node_uid } => {
                dp_ca!("{:.3} ca EventRemoveNodeAck node_uid:{:?}", self.ctx.time(), node_uid);

                // Kubelet now turned off. Remove node from used
                let (kubelet_sim_id, kubelet, group_uid) = self.used_nodes.remove(&node_uid).unwrap();
                // Return kubelet to pool
                self.kubelet_pool.push((kubelet_sim_id, kubelet));
                // Increase free nodes in group counter
                self.free_nodes_by_group.get_mut(&group_uid).unwrap().amount += 1;
                // Remove node's utilization info
                self.low_utilization.remove(&node_uid);
            }
        });
    }
}

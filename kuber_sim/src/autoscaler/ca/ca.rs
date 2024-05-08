use crate::my_imports::*;

/// The component of the Kubernetes responsible for cluster autoscaling.
pub struct CA {
    /// DSLab-Core simulation context of CA.
    ctx: dsc::SimulationContext,
    /// Configuration constants of components in simulation.
    init_config: Rc<RefCell<InitConfig>>,
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
        monitoring: Rc<RefCell<Monitoring>>,
        api_sim_id: dsc::Id,
    ) -> Self {
        let mut ca = Self {
            ctx,
            init_config: init_config.clone(),
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
        for group in &init_nodes.borrow().ca_nodes {
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
                    "{:.12} ca Issues EventRemoveNode node_uid:{:?}",
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

        // If it's not enough pending pods to start up-scaling-> return
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
                    "{:.12} ca node:{:?} added -> cluster",
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
}

impl dsc::EventHandler for CA {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            EventTurnOn {} => {
                dp_ca!("{:.12} ca EventTurnOn", self.ctx.time());

                self.turn_on();
            }

            EventTurnOff {} => {
                dp_ca!("{:.12} ca EventTurnOff", self.ctx.time());

                self.turn_off();
            }

            EventSelfUpdate {} => {
                dp_ca!("{:.12} ca EventSelfUpdate", self.ctx.time());

                assert!(self.is_turned_on, "Logic error. Self update should be canceled for CA.");

                // Request metrics for future process
                self.ctx.emit(
                    EventGetCAMetrics {
                        used_nodes: self.used_nodes.keys().map(|x| *x).collect(),
                        available_nodes: self
                            .free_nodes_by_group
                            .iter()
                            .filter_map(|(_, group)| if group.amount > 0 { Some(group.clone()) } else { None })
                            .collect(),
                    },
                    self.api_sim_id,
                    self.init_config.borrow().network_delays.ca2api,
                );

                // Emit Self-Update
                self.ctx
                    .emit_self(EventSelfUpdate {}, self.init_config.borrow().ca.self_update_period);
            }

            EventPostCAMetrics {
                pending_pod_count,
                used_nodes_utilization,
                may_help,
            } => {
                dp_ca!(
                    "{:.12} ca EventPostCAMetrics pending_pod_count:{:?} used_nodes_utilization:{:?} may_help:{:?}",
                    self.ctx.time(),
                    pending_pod_count,
                    used_nodes_utilization,
                    may_help
                );

                self.process_metrics(pending_pod_count, &used_nodes_utilization, may_help);
            }

            EventRemoveNodeAck { node_uid } => {
                dp_ca!("{:.12} ca EventRemoveNodeAck node_uid:{:?}", self.ctx.time(), node_uid);

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

use crate::my_imports::*;


pub struct CA {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    api_sim_id: dsc::Id,
    // monitoring: Rc<RefCell<Monitoring>>,

    is_turned_on: bool,

    kubelet_pool: Vec<(dsc::Id, Rc<RefCell<Kubelet>>)>, // (kubelet_sim_id, kubelet)
    free_nodes: BTreeMap<u64, NodeGroup>, // node_uid -> node_group
    used_nodes: BTreeMap<u64, (dsc::Id, Rc<RefCell<Kubelet>>, u64)>, // node_uid -> (kubelet_sim_id, kubelet, group_uid)

    low_utilization: BTreeMap<u64, u64>, // node_uid -> cycle counter
}


impl CA {
    pub fn new(ctx: dsc::SimulationContext,
               cluster_state: Rc<RefCell<ClusterState>>,
               monitoring: Rc<RefCell<Monitoring>>,
               api_sim_id: dsc::Id,
               sim: &mut dsc::Simulation) -> Self {
        let mut ca = Self {
            ctx,
            cluster_state: cluster_state.clone(),
            api_sim_id,
            // monitoring: monitoring.clone(),
            is_turned_on: false,
            kubelet_pool: Vec::new(),
            free_nodes: BTreeMap::new(),
            used_nodes: BTreeMap::new(),
            low_utilization: BTreeMap::new(),
        };

        let mut counter = 0;
        for group in &cluster_state.borrow().ca_nodes {
            for _ in 0..group.amount {
                counter += 1;
                let name = "kubelet_ca_".to_owned() + &*counter.to_string();

                let kubelet = Rc::new(RefCell::new(Kubelet::new(
                    sim.create_context(name.clone()),
                    Node::default(),
                    cluster_state.clone(),
                    monitoring.clone(),
                )));
                kubelet.borrow_mut().presimulation_init(api_sim_id);
                let kubelet_id = sim.add_handler(name, kubelet.clone());

                ca.kubelet_pool.push((kubelet_id, kubelet));
            }

            ca.free_nodes.insert(group.group_uid, group.clone());
            ca.free_nodes.get_mut(&group.group_uid).unwrap().node.prepare();
        }

        return ca;
    }

    pub fn turn_on(&mut self) {
        if !self.is_turned_on {
            self.is_turned_on = true;
            self.ctx.emit_self(APICASelfUpdate {}, self.cluster_state.borrow().constants.ca_self_update_period);
        }
    }

    pub fn turn_off(&mut self) {
        if self.is_turned_on {
            self.is_turned_on = false;
            self.ctx.cancel_heap_events(|x| x.src == self.ctx.id() && x.dst == self.ctx.id());
        }
    }

    pub fn process_metrics(&mut self, insufficient_resources_pending: u64, requests: &Vec<(u64, u64)>, node_info: &Vec<(u64, f64, f64)>) {
        // Clear not loaded nodes
        let (min_cpu, min_memory) = (
            self.cluster_state.borrow().constants.ca_remove_node_cpu_percent,
            self.cluster_state.borrow().constants.ca_remove_node_memory_percent,
        );
        for (node_uid, ref cpu, ref memory) in node_info {
            if *cpu <= min_cpu && *memory <= min_memory {
                if self.used_nodes.contains_key(&node_uid) {
                    if !self.low_utilization.contains_key(node_uid) {
                        self.low_utilization.insert(*node_uid, 0);
                    }
                    let cycles = self.low_utilization.get_mut(&node_uid).unwrap();
                    *cycles += 1;

                    if *cycles >= self.cluster_state.borrow().constants.ca_remove_node_delay_cycle {
                        dp_ca!("{:.12} ca Issues APIRemoveNode node_uid:{:?}", self.ctx.time(), node_uid);
                        self.ctx.emit(APIRemoveNode { node_uid: *node_uid }, self.api_sim_id, self.cluster_state.borrow().network_delays.ca2api);
                    }
                }
            }
        }

        if insufficient_resources_pending < self.cluster_state.borrow().constants.ca_add_node_min_pending {
            return;
        }

        // Find best node
        let mut best_group: Option<u64> = None;
        for (_, group) in &self.free_nodes {
            if group.amount == 0 {
                continue;
            }

            for (cpu, memory) in requests {
                if group.node.is_consumable(*cpu, *memory) {
                    best_group = Some(group.group_uid);
                    break;
                }
            }

            if best_group.is_some() {
                break;
            }
        }
        // println!("Best:{:?}", best_group);
        // println!("FreeNodes:{:?}", self.free_nodes);

        // Submit best node
        match best_group {
            Some(group_uid) => {
                let group = self.free_nodes.get_mut(&group_uid).unwrap();

                // Use one node from group
                assert!(group.amount > 0);
                let mut node = group.node.clone();
                group.amount -= 1;

                // Prepare node
                node.prepare();

                // Set node in kubelet
                let (kubelet_sim_id, kubelet) = self.kubelet_pool.pop().unwrap();
                kubelet.borrow_mut().set_node(&node);
                kubelet.borrow_mut().turn_on();

                // Update used nodes
                self.used_nodes.insert(node.metadata.uid, (kubelet_sim_id, kubelet, group_uid));

                dp_ca!("{:.12} ca node:{:?} added -> cluster", self.ctx.time(), node.metadata.uid);
                self.ctx.emit(
                    APIAddNode { kubelet_sim_id, node },
                    self.api_sim_id,
                    self.cluster_state.borrow().network_delays.ca2api + self.cluster_state.borrow().constants.ca_add_node_delay_time
                );
            }
            None => {}
        }
    }
}


impl dsc::EventHandler for CA {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            APICATurnOn {} => {
                dp_ca!("{:.12} ca APICATurnOn", self.ctx.time());

                self.turn_on();
            }
            APICATurnOff {} => {
                dp_ca!("{:.12} ca APICATurnOff", self.ctx.time());

                self.turn_off();
            }
            APICASelfUpdate {} => {
                dp_ca!("{:.12} ca APICASelfUpdate", self.ctx.time());

                if !self.is_turned_on {
                    panic!("Bad logic. Self update should be canceled.");
                }

                self.ctx.emit(APIGetCAMetrics { node_list: self.used_nodes.keys().map(|x| *x).collect() }, self.api_sim_id, self.cluster_state.borrow().network_delays.ca2api);
                self.ctx.emit_self(APICASelfUpdate {}, self.cluster_state.borrow().constants.ca_self_update_period);
            }
            APIPostCAMetrics { insufficient_resources_pending, requests, node_info } => {
                dp_ca!("{:.12} ca APIPostCAMetrics insufficient_resources_pending:{:?} requests:{:?} node_info:{:?}", self.ctx.time(), insufficient_resources_pending, requests, node_info);

                self.process_metrics(insufficient_resources_pending, &requests, &node_info);
            }
            APICommitNodeRemove { node_uid } => {
                dp_ca!("{:.12} ca APICommitNodeRemove node_uid:{:?}", self.ctx.time(), node_uid);

                let (kubelet_sim_id, kubelet, group_uid) = self.used_nodes.remove(&node_uid).unwrap();
                self.kubelet_pool.push((kubelet_sim_id, kubelet));
                self.free_nodes.get_mut(&group_uid).unwrap().amount += 1;
                self.low_utilization.remove(&node_uid);
            }
        });
    }
}

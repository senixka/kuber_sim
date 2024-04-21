use crate::my_imports::*;


pub struct CA {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    api_sim_id: dsc::Id,
    monitoring: Rc<RefCell<Monitoring>>,

    is_turned_on: bool,

    kubelet_pool: Vec<(dsc::Id, Rc<RefCell<Kubelet>>)>,
    free_nodes: BTreeMap<u64, NodeGroup>,
    used_nodes: BTreeMap<u64, (dsc::Id, Rc<RefCell<Kubelet>>, u64)>,
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
            monitoring: monitoring.clone(),
            is_turned_on: false,
            kubelet_pool: Vec::new(),
            free_nodes: BTreeMap::new(),
            used_nodes: BTreeMap::new(),
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

    pub fn process_metrics(&mut self, insufficient_resources_pending: u64, requests: &Vec<(u64, u64)>) {
        // TODO: config consts
        if insufficient_resources_pending == 0 {
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
                    self.cluster_state.borrow().network_delays.ca2api + self.cluster_state.borrow().constants.ca_add_node_delay);
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

                self.ctx.emit(APIGetCAMetrics {}, self.api_sim_id, self.cluster_state.borrow().network_delays.ca2api);
                self.ctx.emit_self(APICASelfUpdate {}, self.cluster_state.borrow().constants.ca_self_update_period);
            }
            APIPostCAMetrics { insufficient_resources_pending, requests } => {
                dp_ca!("{:.12} ca APIPostCAMetrics insufficient_resources_pending:{:?} requests:{:?}", self.ctx.time(), insufficient_resources_pending, requests);

                self.process_metrics(insufficient_resources_pending, &requests);
            }
        });
    }
}

use crate::my_imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitNodes {
    #[serde(default)]
    pub nodes: Vec<NodeGroup>,

    #[serde(default)]
    pub ca_nodes: Vec<NodeGroup>,
}

impl InitNodes {
    pub fn from_yaml(path: &String) -> Self {
        // Read file to string
        let s: String = fs::read_to_string(path).expect(format!("Unable to read file: {0}", path).as_str());
        // Build struct from string
        let mut cluster_state: InitNodes = serde_yaml::from_str(s.as_str()).unwrap();

        // Prepare cluster_state
        cluster_state.prepare();
        return cluster_state;
    }

    pub fn prepare(&mut self) {
        // Prepare each node group for nodes
        for node_group in &mut self.nodes {
            node_group.prepare();
        }

        // Prepare each node group for CA nodes
        for node_group in &mut self.ca_nodes {
            node_group.prepare();
        }
    }

    pub fn submit(
        &self,
        sim: &mut dsc::Simulation,
        emitter: &dsc::SimulationContext,
        init_config: Rc<RefCell<InitConfig>>,
        monitoring: Rc<RefCell<Monitoring>>,
        api_sim_id: dsc::Id,
    ) {
        for node_group in self.nodes.iter() {
            for _ in 0..node_group.amount {
                // Get node template
                let mut node = node_group.node.clone();
                // Prepare node from template
                node.prepare(node_group.group_uid);

                // Create unique kubelet name
                let name = "kubelet_".to_owned() + &*node.metadata.uid.to_string();

                // Create kubelet
                let kubelet = Rc::new(RefCell::new(Kubelet::new(
                    sim.create_context(name.clone()),
                    init_config.clone(),
                    monitoring.clone(),
                    api_sim_id,
                    node.clone(),
                )));
                // Turn on kubelet
                kubelet.borrow_mut().turn_on();

                // Register kubelet in simulation
                let kubelet_id = sim.add_handler(name, kubelet.clone());
                // Emit AddNode event
                emitter.emit_now(
                    EventAddNode {
                        kubelet_sim_id: kubelet_id,
                        node: node.clone(),
                    },
                    api_sim_id,
                );
            }
        }
    }
}

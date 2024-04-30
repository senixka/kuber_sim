use crate::my_imports::*;


pub struct Kubelet {
    pub ctx: dsc::SimulationContext,
    pub cluster_state: Rc<RefCell<ClusterState>>,
    pub monitoring: Rc<RefCell<Monitoring>>,
    pub api_sim_id: dsc::Id,

    // Underlying node
    pub node: Node,

    // Inner state
    pub pods: HashMap<u64, Pod>,                            // HashMap<pod_uid, Pod>
    pub evict_order: BTreeSet<(QoSClass, i64, u64)>,        // BTreeSet<(pod_QoSClass, pod_priority, pod_uid)>
    pub running_loads: BTreeMap<u64, (u64, u64, LoadType)>, // BTreeMap<pod_uid, (current_cpu, current_memory, load_profile)>

    // Is kubelet turned on
    pub is_turned_on: bool,
}

impl Kubelet {
    pub fn new(ctx: dsc::SimulationContext,
               cluster_state: Rc<RefCell<ClusterState>>,
               monitoring: Rc<RefCell<Monitoring>>,
               api_sim_id: dsc::Id,
               node: Node) -> Self {
        Self {
            ctx,
            cluster_state,
            monitoring,
            api_sim_id,
            node,

            // Inner state
            pods: HashMap::new(),
            evict_order: BTreeSet::new(),
            running_loads: BTreeMap::new(),
            is_turned_on: false,
        }
    }

    ////////////////// Process pod events //////////////////

    pub fn process_new_pod(&mut self, pod: Pod) {
        // Get pod_uid
        let pod_uid = pod.metadata.uid;

        // Store original pod
        assert!(!self.pods.contains_key(&pod_uid));
        self.pods.insert(pod_uid, pod.clone());

        // Run pod's load
        let mut load = pod.spec.load.clone();
        let (cpu, memory, next_change, is_finished) = load.start(self.ctx.time());
        assert!(!is_finished);

        // If pod request exceeds limits
        if !pod.request_matches_limits(cpu, memory) {
            // Remove stored pod
            self.pods.remove(&pod_uid).unwrap();

            // Update pod phase
            self.send_pod_phase_update(pod_uid, PodPhase::Failed);
            return;
        }

        // If there are insufficient resources on the node
        if !self.node.is_consumable(cpu, memory) {
            // Remove stored pod
            self.pods.remove(&pod_uid).unwrap();

            // Update pod phase
            self.send_pod_phase_update(pod_uid, PodPhase::Pending);
            return;
        }

        // Consume node resources
        self.node.consume(cpu, memory);
        self.monitoring.borrow_mut().kubelet_on_pod_placed(cpu, memory);

        // Update inner state
        self.running_loads.insert(pod_uid, (cpu, memory, load));
        self.evict_order.insert((pod.status.qos_class, pod.spec.priority, pod.metadata.uid));

        // Send pod consumption to Api-server
        let pod_spec = &self.pods.get(&pod_uid).unwrap().spec;
        self.send_pod_metrics(pod_uid,
                              (cpu as f64) * 100.0 / pod_spec.request_cpu as f64,
                              (memory as f64) * 100.0 / pod_spec.request_memory as f64);


        // Pod's load next change event
        self.ctx.emit_self(EventKubeletNextChange { pod_uid }, next_change);
    }

    pub fn process_evicted_pod(&mut self, pod_uid: u64) {
        let (prev_cpu, prev_memory, _) = self.running_loads.get_mut(&pod_uid).unwrap();

        // Restore previous resources
        self.node.restore(*prev_cpu, *prev_memory);
        self.monitoring.borrow_mut().kubelet_on_pod_unplaced(*prev_cpu, *prev_memory);

        // Remove pod
        self.remove_pod_without_restoring_resources(pod_uid, PodPhase::Pending);
    }

    pub fn on_pod_next_change(&mut self, pod_uid: u64) {
        // Get Previous and New pod's load
        let (prev_cpu, prev_memory, load) = self.running_loads.get_mut(&pod_uid).unwrap();
        let (new_cpu, new_memory, next_change, is_finished) = load.update(self.ctx.time());

        // Restore previous resources
        self.node.restore(*prev_cpu, *prev_memory);
        self.monitoring.borrow_mut().kubelet_on_pod_unplaced(*prev_cpu, *prev_memory);

        // If pod finished -> pod Succeeded
        if is_finished {
            self.remove_pod_without_restoring_resources(pod_uid, PodPhase::Succeeded);
            return;
        }

        // If pod exceeds its limits -> pod Failed
        if !self.pods.get(&pod_uid).unwrap().request_matches_limits(new_cpu, new_memory) {
            self.remove_pod_without_restoring_resources(pod_uid, PodPhase::Failed);
            return;
        }

        // If node has not enough resources -> try eviction
        if !self.node.is_consumable(new_cpu, new_memory) {
            // TODO: eviction with respect to Priority & QoS
            self.remove_pod_without_restoring_resources(pod_uid, PodPhase::Pending);
            return;
        }

        assert!(self.node.is_consumable(new_cpu, new_memory));

        // Consume node resources
        *prev_cpu = new_cpu;
        *prev_memory = new_memory;
        self.node.consume(new_cpu, new_memory);
        self.monitoring.borrow_mut().kubelet_on_pod_placed(new_cpu, new_memory);

        // Update pod consumption in api server
        let pod_spec = &self.pods.get(&pod_uid).unwrap().spec;
        self.send_pod_metrics(pod_uid,
                              (new_cpu as f64) * 100.0 / pod_spec.request_cpu as f64,
                              (new_memory as f64) * 100.0 / pod_spec.request_memory as f64);

        // Next change self update
        self.ctx.emit_self(EventKubeletNextChange { pod_uid }, next_change);
    }

    pub fn remove_pod_without_restoring_resources(&mut self, pod_uid: u64, new_phase: PodPhase) {
        // Update inner state
        self.running_loads.remove(&pod_uid).unwrap();
        let pod = self.pods.remove(&pod_uid).unwrap();
        self.evict_order.remove(&(pod.status.qos_class, pod.spec.priority, pod_uid));

        // Send PodPhase update
        self.send_pod_phase_update(pod_uid, new_phase);
    }

    ////////////////// Kubelet Turn On/Off //////////////////

    pub fn turn_on(&mut self) {
        self.is_turned_on = true;
    }

    pub fn turn_off(&mut self) {
        for &pod_uid in self.pods.keys() {
            // Send pod phase update
            self.send_pod_phase_update(pod_uid, PodPhase::Pending);

            // Restore previous resources
            let (prev_cpu, prev_memory, _) = self.running_loads.get_mut(&pod_uid).unwrap();
            self.node.restore(*prev_cpu, *prev_memory);
            self.monitoring.borrow_mut().kubelet_on_pod_unplaced(*prev_cpu, *prev_memory);
        }

        // All resources should be restored
        assert_eq!(self.node.spec.installed_cpu, self.node.spec.available_cpu);
        assert_eq!(self.node.spec.installed_memory, self.node.spec.available_memory);

        // Clear inner state
        self.pods.clear();
        self.evict_order.clear();
        self.running_loads.clear();

        // Cancel future events
        self.ctx.cancel_heap_events(|x| x.src == self.ctx.id() && x.dst == self.ctx.id());

        // Turn off kubelet
        self.is_turned_on = false;

        // Send RemoveNode ACK
        self.ctx.emit(EventRemoveNodeAck { node_uid: self.node.metadata.uid },
                      self.api_sim_id,
                      self.cluster_state.borrow().network_delays.kubelet2api
        );
    }

    ////////////////// Kubelet replace node //////////////////

    pub fn replace_node(&mut self, new_node: &Node) {
        assert_eq!(self.is_turned_on, false);
        assert!(self.pods.is_empty());
        assert!(self.running_loads.is_empty());
        assert!(self.evict_order.is_empty());

        self.node = new_node.clone();
    }

    ////////////////// Export metrics //////////////////

    pub fn send_pod_metrics(&self, pod_uid: u64, current_cpu: f64, current_memory: f64) {
        self.ctx.emit(EventUpdatePodMetricsFromKubelet { pod_uid, current_cpu, current_memory},
                      self.api_sim_id,
                      self.cluster_state.borrow().network_delays.kubelet2api
        );
    }

    pub fn send_pod_phase_update(&self, pod_uid: u64, new_phase: PodPhase) {
        self.ctx.emit(
            EventUpdatePodFromKubelet { pod_uid, new_phase, node_uid: self.node.metadata.uid },
            self.api_sim_id,
            self.cluster_state.borrow().network_delays.kubelet2api
        );
    }
}

impl dsc::EventHandler for Kubelet {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            EventUpdatePodFromScheduler { pod , pod_uid, new_phase, node_uid } => {
                dp_kubelet!("{:.12} node:{:?} EventUpdatePodFromScheduler pod_uid:{:?} new_phase:{:?}", self.ctx.time(), self.node.metadata.uid, pod_uid, new_phase);

                // Some invariants assertions
                assert_eq!(node_uid, self.node.metadata.uid);
                assert_eq!(self.running_loads.len(), self.pods.len());
                assert!(self.is_turned_on, "Logic error. Api-server should stop routing if kubelet turned off.");

                // Process PodPhase
                match new_phase {
                    PodPhase::Running => {
                        // If pod already Running -> return
                        if self.pods.contains_key(&pod_uid) {
                            return;
                        }

                        // Update inner state
                        self.process_new_pod(pod.unwrap());
                    }
                    PodPhase::Pending => {
                        // If pod is not managed by kubelet -> return
                        if !self.pods.contains_key(&pod_uid) {
                            return;
                        }

                        // Update inner state
                        self.process_evicted_pod(pod_uid);
                    }
                    PodPhase::Succeeded | PodPhase::Failed => {
                        panic!("Logic error. Unexpected pod phase:{:?},", new_phase);
                    }
                }

                // Invariant assertion
                assert_eq!(self.running_loads.len(), self.pods.len());
            }

            EventKubeletNextChange { pod_uid } => {
                dp_kubelet!("{:.12} node:{:?} EventKubeletNextChange pod_uid:{:?}", self.ctx.time(), self.node.metadata.uid, pod_uid);

                assert!(self.is_turned_on, "Logic error. All self-events should be canceled.");

                // If pod still managed by kubelet
                if self.pods.contains_key(&pod_uid) {
                    self.on_pod_next_change(pod_uid);
                }
            }

            EventRemoveNode { node_uid } => {
                dp_kubelet!("{:.12} node:{:?} EventRemoveNode", self.ctx.time(), self.node.metadata.uid);

                assert_eq!(node_uid, self.node.metadata.uid, "Api-server error. Wrong routing.");

                // Graceful kubelet shutdown
                self.turn_off();
            }
        });
    }
}

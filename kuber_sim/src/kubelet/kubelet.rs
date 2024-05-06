use crate::my_imports::*;


pub struct Kubelet {
    pub ctx: dsc::SimulationContext,
    pub api_sim_id: dsc::Id,
    pub init_config: Rc<RefCell<InitConfig>>,
    pub monitoring: Rc<RefCell<Monitoring>>,

    // Underlying node
    pub node: Node,

    // Pod info
    pub pods: HashMap<u64, Pod>,                            // HashMap<pod_uid, Pod>
    // Pod's load profiles
    pub running_loads: BTreeMap<u64, (i64, i64, LoadType)>, // BTreeMap<pod_uid, (current_cpu, current_memory, load_profile)>
    // Eviction order
    pub eviction_order: EvictionOrder,

    // Is kubelet turned on
    pub is_turned_on: bool,
}

impl Kubelet {
    pub fn new(ctx: dsc::SimulationContext,
               init_config: Rc<RefCell<InitConfig>>,
               monitoring: Rc<RefCell<Monitoring>>,
               api_sim_id: dsc::Id,
               node: Node) -> Self {
        Self {
            ctx,
            api_sim_id,
            init_config,
            monitoring,

            // Underlying node
            node,

            // Inner state
            pods: HashMap::new(),
            eviction_order: EvictionOrder::new(),
            running_loads: BTreeMap::new(),
            is_turned_on: false,
        }
    }

    ////////////////// Eviction  //////////////////

    pub fn do_eviction(&mut self) {
        // Do eviction should be called only in case of resource overuse
        assert!(self.node.spec.available_cpu < 0 || self.node.spec.available_memory < 0);
        // Inner invariant
        assert_eq!(self.pods.len(), self.eviction_order.len());

        // While there are pods and eviction needed
        while !self.eviction_order.is_empty()
              && (self.node.spec.available_cpu < 0 || self.node.spec.available_memory < 0) {
            // Get first order pod to evict
            let pod_uid = self.eviction_order.first().unwrap();

            // Evict this pod
            self.remove_pod_with_restoring_resources(pod_uid, PodPhase::Evicted, None, None);
        }

        // Assert (Purpose of eviction achieved)
        assert!(self.node.spec.available_cpu >= 0 && self.node.spec.available_memory >= 0);
        // Inner invariant
        assert_eq!(self.pods.len(), self.eviction_order.len());
    }

    ////////////////// Process pod events //////////////////

    pub fn add_new_pod(&mut self, pod: Pod, preempt_uids: &Option<Vec<u64>>) {
        // Firstly, preempt if scheduler asks
        match preempt_uids {
            Some(uids) => {
                for uid in uids {
                    if self.pods.contains_key(uid) {
                        self.remove_pod_with_restoring_resources(*uid, PodPhase::Preempted, None, None);
                    }
                }
            }
            None => {}
        }

        // Get pod_uid
        let pod_uid = pod.metadata.uid;
        assert!(!self.pods.contains_key(&pod_uid));

        // Send pod update to Api-server
        self.send_pod_update_zero_usage(pod_uid, PodPhase::Running);

        // Get pod's load
        let mut load = pod.spec.load.clone();
        let (cpu, memory, next_change, is_finished) = load.start(self.ctx.time());
        assert!(!is_finished);

        // If pod usage exceeds limits -> pod Failed
        if !pod.is_usage_matches_limits(cpu, memory) {
            self.send_pod_update(&pod.spec, pod_uid, PodPhase::Failed, cpu, memory);
            return;
        }

        // If there are insufficient resources on the node -> place pod anyway and trigger eviction
        let need_eviction = !self.node.is_consumable(cpu, memory);

        // Consume node resources
        self.node.consume(cpu, memory);
        self.monitoring.borrow_mut().kubelet_on_pod_placed(cpu, memory);

        // Store pod
        self.pods.insert(pod_uid, pod.clone());
        // Store pod's load
        self.running_loads.insert(pod_uid, (cpu, memory, load));
        // Add pod to eviction order
        self.eviction_order.add(&pod, memory);

        // Send pod update to Api-server
        self.send_pod_update(&pod.spec, pod_uid, PodPhase::Running, cpu, memory);

        // Pod's load next change event
        self.ctx.emit_self(EventKubeletNextChange { pod_uid }, next_change);

        // Do eviction if needed
        if need_eviction {
            self.do_eviction();
        }

        // Inner invariants
        assert!(self.node.spec.available_cpu >= 0);
        assert!(self.node.spec.available_memory >= 0);
        assert!(self.node.spec.available_cpu <= self.node.spec.installed_cpu);
        assert!(self.node.spec.available_memory <= self.node.spec.installed_memory);
        assert_eq!(self.pods.len(), self.eviction_order.len());
    }

    pub fn on_pod_next_change(&mut self, pod_uid: u64) {
        // Get previous and new pod's load
        let (prev_cpu, prev_memory, load) = self.running_loads.get_mut(&pod_uid).unwrap();
        let (new_cpu, new_memory, next_change, is_finished) = load.update(self.ctx.time());

        // Restore previous resources
        self.node.restore(*prev_cpu, *prev_memory);
        self.monitoring.borrow_mut().kubelet_on_pod_unplaced(*prev_cpu, *prev_memory);

        // If pod finished -> pod Succeeded
        if is_finished {
            self.remove_pod_without_restoring_resources(pod_uid, PodPhase::Succeeded, 0, 0);
            return;
        }

        // Locate pod
        let pod = self.pods.get(&pod_uid).unwrap();

        // If pod usage exceeds limits -> pod Failed
        if !pod.is_usage_matches_limits(new_cpu, new_memory) {
            self.remove_pod_without_restoring_resources(pod_uid, PodPhase::Failed, new_cpu, new_memory);
            return;
        }

        // If there are insufficient resources on the node -> place pod anyway and trigger eviction
        let need_eviction = !self.node.is_consumable(new_cpu, new_memory);

        // Consume node resources
        self.node.consume(new_cpu, new_memory);
        self.monitoring.borrow_mut().kubelet_on_pod_placed(new_cpu, new_memory);

        // Update eviction order
        self.eviction_order.remove(&pod, *prev_memory);
        self.eviction_order.add(&pod, new_memory);

        // Update pod's load
        (*prev_cpu, *prev_memory) = (new_cpu, new_memory);

        // Send pod update to Api-server
        self.send_pod_update(&pod.spec, pod_uid, PodPhase::Running, new_cpu, new_memory);

        // Next change self update
        self.ctx.emit_self(EventKubeletNextChange { pod_uid }, next_change);

        // Do eviction if needed
        if need_eviction {
            self.do_eviction();
        }

        // Inner invariants
        assert!(self.node.spec.available_cpu >= 0);
        assert!(self.node.spec.available_memory >= 0);
        assert!(self.node.spec.available_cpu <= self.node.spec.installed_cpu);
        assert!(self.node.spec.available_memory <= self.node.spec.installed_memory);
        assert_eq!(self.pods.len(), self.eviction_order.len());
    }

    ////////////////// Remove pod //////////////////

    pub fn remove_pod_with_restoring_resources(&mut self, pod_uid: u64, end_phase: PodPhase, end_cpu: Option<i64>, end_memory: Option<i64>) {
        let (prev_cpu, prev_memory, _) = self.running_loads.get(&pod_uid).unwrap();

        // Restore previous resources
        self.node.restore(*prev_cpu, *prev_memory);
        self.monitoring.borrow_mut().kubelet_on_pod_unplaced(*prev_cpu, *prev_memory);

        // Restore other stuff
        self.remove_pod_without_restoring_resources(
            pod_uid, end_phase, end_cpu.unwrap_or(*prev_cpu), end_memory.unwrap_or(*prev_memory)
        );
    }

    pub fn remove_pod_without_restoring_resources(&mut self, pod_uid: u64, end_phase: PodPhase, end_cpu: i64, end_memory: i64) {
        // Pod to remove cannot be Running
        assert_ne!(end_phase, PodPhase::Running);

        // Remove load info
        let (_, memory, _) = self.running_loads.remove(&pod_uid).unwrap();
        // Remove pod info
        let pod = self.pods.remove(&pod_uid).unwrap();
        // Remove from eviction order
        self.eviction_order.remove(&pod, memory);

        // Send pod update to Api-server
        self.send_pod_update(&pod.spec, pod_uid, end_phase, end_cpu, end_memory);
    }

    ////////////////// Kubelet Turn On/Off //////////////////

    pub fn turn_on(&mut self) {
        self.is_turned_on = true;
    }

    pub fn turn_off(&mut self) {
        for &pod_uid in self.pods.keys() {
            // Send pod metrics to Api-server
            self.send_pod_update_zero_usage(pod_uid, PodPhase::Pending);

            // Restore previous resources
            let (prev_cpu, prev_memory, _) = self.running_loads.get_mut(&pod_uid).unwrap();
            self.node.restore(*prev_cpu, *prev_memory);
            self.monitoring.borrow_mut().kubelet_on_pod_unplaced(*prev_cpu, *prev_memory);
        }

        // All resources should be restored
        assert_eq!(self.node.spec.installed_cpu, self.node.spec.available_cpu);
        assert_eq!(self.node.spec.installed_memory, self.node.spec.available_memory);
        // Inner state invariants
        assert_eq!(self.pods.len(), self.running_loads.len());
        assert_eq!(self.pods.len(), self.eviction_order.len());

        // Clear inner state
        self.pods.clear();
        self.eviction_order.clear();
        self.running_loads.clear();

        // Cancel future all self-emitted events
        self.ctx.cancel_heap_events(|x| x.src == self.ctx.id() && x.dst == self.ctx.id());

        // Turn off kubelet
        self.is_turned_on = false;

        // Send RemoveNode ACK
        self.ctx.emit(EventRemoveNodeAck { node_uid: self.node.metadata.uid },
                      self.api_sim_id,
                      self.init_config.borrow().network_delays.kubelet2api
        );
    }

    ////////////////// Kubelet replace node //////////////////

    pub fn replace_node(&mut self, new_node: &Node) {
        assert_eq!(self.is_turned_on, false);
        assert!(self.pods.is_empty());
        assert!(self.running_loads.is_empty());
        assert!(self.eviction_order.is_empty());

        self.node = new_node.clone();
    }

    ////////////////// Export metrics //////////////////

    pub fn send_pod_update(&self, spec: &PodSpec, pod_uid: u64, phase: PodPhase, cpu: i64, memory: i64) {
        self.ctx.emit(
            EventPodUpdateFromKubelet {
                pod_uid,
                current_phase: phase,
                current_cpu: cpu as f64 / spec.request_cpu as f64,
                current_memory: memory as f64 / spec.request_memory as f64,
            },
            self.api_sim_id,
            self.init_config.borrow().network_delays.kubelet2api
        );
    }

    pub fn send_pod_update_zero_usage(&self, pod_uid: u64, phase: PodPhase) {
        self.ctx.emit(
            EventPodUpdateFromKubelet {
                pod_uid,
                current_phase: phase,
                current_cpu: 0.0,
                current_memory: 0.0,
            },
            self.api_sim_id,
            self.init_config.borrow().network_delays.kubelet2api
        );
    }
}

impl dsc::EventHandler for Kubelet {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            EventUpdatePodFromScheduler { pod , pod_uid, preempt_uids, new_phase, node_uid } => {
                dp_kubelet!("{:.12} node:{:?} EventUpdatePodFromScheduler pod_uid:{:?} preempt_uids:{:?} new_phase:{:?}", self.ctx.time(), self.node.metadata.uid, pod_uid, preempt_uids, new_phase);

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

                        // Run new pod
                        self.add_new_pod(pod.unwrap(), &preempt_uids);
                    }
                    PodPhase::Preempted | PodPhase::Removed => {
                        // If pod is not managed by kubelet -> return
                        if !self.pods.contains_key(&pod_uid) {
                            return;
                        }

                        // Preempt or Remove this pod depending on new_phase
                        self.remove_pod_with_restoring_resources(pod_uid, new_phase, None, None);
                    }
                    PodPhase::Pending | PodPhase::Succeeded | PodPhase::Failed | PodPhase::Evicted => {
                        panic!("Logic error. Kubelet unexpected PodPhase:{:?},", new_phase);
                    }
                }

                // Invariant assertion
                assert_eq!(self.running_loads.len(), self.pods.len());
            }

            EventKubeletNextChange { pod_uid } => {
                dp_kubelet!("{:.12} node:{:?} EventKubeletNextChange pod_uid:{:?}", self.ctx.time(), self.node.metadata.uid, pod_uid);

                assert!(self.is_turned_on, "Logic error. All self-events should be canceled.");

                // If pod still managed by kubelet -> update its load
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

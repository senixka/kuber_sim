use crate::my_imports::*;


pub struct Kubelet {
    pub ctx: dsc::SimulationContext,
    pub cluster_state: Rc<RefCell<ClusterState>>,
    pub api_sim_id: dsc::Id,
    pub node: Node,
    monitoring: Rc<RefCell<Monitoring>>,

    pub pods: HashMap<u64, Pod>,
    pub evict_order: BTreeSet<(QoSClass, i64, u64)>,
    pub running_loads: BTreeMap<u64, (u64, u64, LoadType)>,
    pub self_update_enabled: bool,
}

impl Kubelet {
    pub fn new(ctx: dsc::SimulationContext, node: Node, cluster_state: Rc<RefCell<ClusterState>>, monitoring: Rc<RefCell<Monitoring>>) -> Self {
        Self {
            ctx,
            cluster_state,
            api_sim_id: dsc::Id::MAX,
            monitoring,
            node,
            pods: HashMap::new(),
            evict_order: BTreeSet::new(),
            running_loads: BTreeMap::new(),
            self_update_enabled: false,
        }
    }

    pub fn presimulation_init(&mut self, api_sim_id: dsc::Id) {
        self.api_sim_id = api_sim_id;
    }

    pub fn place_new_pod(&mut self, pod: Pod) -> bool {
        let pod_uid = pod.metadata.uid;

        // Store pod
        assert!(!self.pods.contains_key(&pod_uid));
        self.pods.insert(pod_uid, pod.clone());

        // Run pod's load
        let mut load = pod.spec.load.clone();
        let (cpu, memory, next_change, is_finished) = load.start(self.ctx.time());
        assert!(!is_finished);

        if !self.node.is_consumable(cpu, memory) {
            self.pods.remove(&pod_uid).unwrap();
            return false;
        }
        self.node.consume(cpu, memory);
        self.running_loads.insert(pod_uid, (cpu, memory, load));
        self.evict_order.insert((pod.status.qos_class, pod.spec.priority, pod.metadata.uid));

        if self.ctx.time() >= 65640.0 {
            debug_print!("Start, Next change: {0}", next_change);
        }
        self.ctx.emit_self(APIKubeletSelfNextChange { pod_uid }, next_change);

        self.monitoring.borrow_mut().kubelet_on_pod_placed(cpu, memory);
        return true;
    }

    pub fn on_pod_next_change(&mut self, pod_uid: u64) {
        let (prev_cpu, prev_memory, load) = self.running_loads.get_mut(&pod_uid).unwrap();
        let (new_cpu, new_memory, next_change, is_finished) = load.update(self.ctx.time());

        if self.ctx.time() >= 65640.0 {
            debug_print!("[{5}] Pod update: cpu {0} -> {1}, mem: {2} -> {3}, next_change: {4}", prev_cpu, new_cpu, prev_memory, new_memory, next_change, self.ctx.time());
        }

        // Restore previous resources
        self.node.restore(*prev_cpu, *prev_memory);
        self.monitoring.borrow_mut().kubelet_on_pod_unplaced(*prev_cpu, *prev_memory);

        if is_finished {
            self.remove_pod(pod_uid, PodPhase::Succeeded);
            return;
        }

        // TODO: eviction with respect to QoS class
        if !self.node.is_consumable(new_cpu, new_memory) {
            // let pod = self.pods.get(&pod_uid).unwrap();
            //
            // // Evict with respect to QoS and Priority
            // for (qos, priority, other_uid) in self.evict_order {
            //     if *priority < pod.spec.priority {
            //
            //     }
            // }
        }

        assert!(self.node.is_consumable(new_cpu, new_memory));

        // Consume resources
        *prev_cpu = new_cpu;
        *prev_memory = new_memory;
        self.node.consume(new_cpu, new_memory);
        self.monitoring.borrow_mut().kubelet_on_pod_placed(new_cpu, new_memory);

        // Next change self update
        self.ctx.emit_self(APIKubeletSelfNextChange { pod_uid }, next_change);
    }

    pub fn remove_pod(&mut self, pod_uid: u64, new_phase: PodPhase) {
        self.running_loads.remove(&pod_uid).unwrap();
        let pod = self.pods.remove(&pod_uid).unwrap();
        let _was_present = self.evict_order.remove(&(pod.status.qos_class, pod.spec.priority, pod_uid)); assert!(_was_present);
        self.monitoring.borrow_mut().kubelet_on_pod_finished();

        let data = APIUpdatePodFromKubelet {
            pod_uid,
            new_phase,
            node_uid: self.node.metadata.uid,
        };
        self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.kubelet2api);
    }
}

impl dsc::EventHandler for Kubelet {
    fn on(&mut self, event: dsc::Event) {
        if self.ctx.time() >= 65640.0 {
            debug_print!("Kubelet Node_{0} EventHandler ------>", self.node.metadata.uid);
        }
        dsc::cast!(match event.data {
            APIUpdatePodFromScheduler { pod, new_phase, node_uid } => {
                if self.ctx.time() >= 65640.0 {
                    debug_print!("New pod");
                }

                assert_eq!(node_uid, self.node.metadata.uid);
                assert_eq!(new_phase, PodPhase::Running);
                assert_eq!(self.running_loads.len(), self.pods.len());

                if !self.pods.contains_key(&pod.metadata.uid) {

                    if !self.place_new_pod(pod.clone()) {
                        let data = APIUpdatePodFromKubelet {
                            pod_uid: pod.metadata.uid,
                            new_phase: PodPhase::Pending,
                            node_uid: self.node.metadata.uid,
                        };
                        self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.kubelet2api);
                    }
                    assert_eq!(self.running_loads.len(), self.pods.len());
                }
            }
            APIKubeletSelfUpdate {} => {
                if self.ctx.time() >= 65640.0 {
                    debug_print!("Self update");
                }

                // assert_eq!(self.running_loads.len(), self.pods.len());
                // self.update_load();
                // assert_eq!(self.running_loads.len(), self.pods.len());
                //
                // if !self.pods.is_empty() {
                //     self.self_update_enabled = true;
                //     self.ctx.emit_self(APIKubeletSelfUpdate{}, self.cluster_state.borrow().constants.kubelet_self_update_period);
                // } else {
                //     self.self_update_enabled = false;
                // }
            }
            APIKubeletSelfNextChange { pod_uid } => {
                if self.ctx.time() >= 65640.0 {
                    debug_print!("[{1}] Next change for {0}", pod_uid, self.ctx.time());
                }

                self.on_pod_next_change(pod_uid);
            }
        });
        if self.ctx.time() >= 65640.0 {
            debug_print!("Kubelet Node_{0} EventHandler <------", self.node.metadata.uid);
        }
    }
}

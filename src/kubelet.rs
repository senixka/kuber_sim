use crate::my_imports::*;


pub struct Kubelet {
    pub ctx: dsc::SimulationContext,
    pub cluster_state: Rc<RefCell<ClusterState>>,
    pub api_sim_id: dsc::Id,
    pub node: Node,
    monitoring: Rc<RefCell<Monitoring>>,

    pub pods: HashMap<u64, Pod>,
    pub evict_order: BTreeSet<(QoSClass, i64, u64)>, // TODO: use it correctly
    pub running_loads: BTreeMap<u64, (u64, u64, LoadType)>,

    pub is_turned_on: bool,
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
            is_turned_on: false,
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

        self.monitoring.borrow_mut().kubelet_on_pod_placed(cpu, memory);
        self.ctx.emit_self(APIKubeletSelfNextChange { pod_uid }, next_change);
        return true;
    }

    pub fn on_pod_next_change(&mut self, pod_uid: u64) {
        let (prev_cpu, prev_memory, load) = self.running_loads.get_mut(&pod_uid).unwrap();
        let (new_cpu, new_memory, next_change, is_finished) = load.update(self.ctx.time());

        // Restore previous resources
        self.node.restore(*prev_cpu, *prev_memory);
        self.monitoring.borrow_mut().kubelet_on_pod_unplaced(*prev_cpu, *prev_memory);

        if is_finished {
            self.remove_pod(pod_uid, PodPhase::Succeeded);
            return;
        }

        // TODO: eviction with respect to Priority & QoS
        if !self.node.is_consumable(new_cpu, new_memory) {
            self.remove_pod(pod_uid, PodPhase::Pending);
            return;
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

        let data = APIUpdatePodFromKubelet {
            pod_uid,
            new_phase,
            node_uid: self.node.metadata.uid,
        };
        self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.kubelet2api);
    }

    pub fn kubelet_turn_off(&mut self) {
        for &pod_uid in self.pods.keys() {
            let data = APIUpdatePodFromKubelet {
                pod_uid,
                new_phase: PodPhase::Pending,
                node_uid: self.node.metadata.uid,
            };
            self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.kubelet2api);

            // Restore previous resources
            let (prev_cpu, prev_memory, _) = self.running_loads.get_mut(&pod_uid).unwrap();
            self.node.restore(*prev_cpu, *prev_memory);
            self.monitoring.borrow_mut().kubelet_on_pod_unplaced(*prev_cpu, *prev_memory);
        }

        assert_eq!(self.node.spec.installed_cpu, self.node.spec.available_cpu);
        assert_eq!(self.node.spec.installed_memory, self.node.spec.available_memory);

        self.pods.clear();
        self.evict_order.clear();
        self.running_loads.clear();
        self.ctx.cancel_heap_events(|x| x.src == self.ctx.id() && x.dst == self.ctx.id());

        self.is_turned_on = false;

        self.ctx.emit(APICommitCANodeRemove { node_uid: self.node.metadata.uid }, self.api_sim_id, self.cluster_state.borrow().network_delays.kubelet2api);
    }

    pub fn turn_on(&mut self) {
        self.is_turned_on = true;
    }

    pub fn set_node(&mut self, node: &Node) {
        assert_eq!(self.is_turned_on, false);
        assert!(self.pods.is_empty());
        assert!(self.running_loads.is_empty());
        assert!(self.evict_order.is_empty());

        self.node = node.clone();
    }
}

impl dsc::EventHandler for Kubelet {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            APIUpdatePodFromScheduler { pod, new_phase, node_uid } => {
                dp_kubelet!("{:.12} node:{:?} APIUpdatePodFromScheduler pod_uid:{:?} new_phase:{:?}", self.ctx.time(), self.node.metadata.uid, pod.metadata.uid, new_phase);

                if !self.is_turned_on {
                    panic!("Logic error. API-Server should stop routing if kubelet turned off.");
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
            APIKubeletSelfNextChange { pod_uid } => {
                dp_kubelet!("{:.12} node:{:?} APIKubeletSelfNextChange pod_uid:{:?}", self.ctx.time(), self.node.metadata.uid, pod_uid);

                if !self.is_turned_on {
                    panic!("Logic error. All self-events should be canceled.");
                }

                self.on_pod_next_change(pod_uid);
            }
            APIRemoveNode { node_uid } => {
                dp_kubelet!("{:.12} node:{:?} APITurnOffNode", self.ctx.time(), self.node.metadata.uid);

                assert_eq!(node_uid, self.node.metadata.uid);
                self.kubelet_turn_off();
            }
        });
    }
}

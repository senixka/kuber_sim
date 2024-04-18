use crate::my_imports::*;


pub struct Scheduler<
    ActiveQCmp,
    BackOffQ,
    const NFILTER: usize,
    const NSCORE: usize,
> {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    api_sim_id: dsc::Id,
    monitoring: Rc<RefCell<Monitoring>>,

    self_update_enabled: bool,

    // Cache
    running_pods: HashMap<u64, Pod>,
    pending_pods: HashMap<u64, Pod>, // TODO: remove it
    nodes: HashMap<u64, Node>,
    node_rtree: NodeRTree,

    // Queues
    active_queue: BinaryHeap<ActiveQCmp>,
    backoff_queue: BackOffQ,
    failed_attempts: HashMap<u64, u64>,

    // Pipeline
    filters: [FilterPluginT; NFILTER],
    scorers: [ScorePluginT; NSCORE],
    scorer_weights: [i64; NSCORE],
    score_normalizers: [NormalizeScorePluginT; NSCORE],
}

impl <
    ActiveQCmp: TraitActiveQCmp,
    BackOffQ: TraitBackOffQ,
    const NFILTER: usize,
    const NSCORE: usize,
> Scheduler<ActiveQCmp, BackOffQ, NFILTER, NSCORE> {
    pub fn new(
        ctx: dsc::SimulationContext,
        cluster_state: Rc<RefCell<ClusterState>>,
        monitoring: Rc<RefCell<Monitoring>>,
        filters: [FilterPluginT; NFILTER],
        scorers: [ScorePluginT; NSCORE],
        score_normalizers: [NormalizeScorePluginT; NSCORE],
        scorer_weights: [i64; NSCORE],
        backoff_queue: BackOffQ,
    ) -> Scheduler<ActiveQCmp, BackOffQ, NFILTER, NSCORE> {
        Self {
            ctx,
            cluster_state,
            api_sim_id: dsc::Id::MAX,
            monitoring,
            running_pods: HashMap::new(),
            pending_pods: HashMap::new(),
            nodes: HashMap::new(),
            node_rtree: NodeRTree::new(),
            self_update_enabled: false,
            active_queue: BinaryHeap::new(),
            failed_attempts: HashMap::new(),
            backoff_queue,
            filters,
            scorers,
            score_normalizers,
            scorer_weights,
        }
    }

    pub fn presimulation_init(&mut self, api_sim_id: dsc::Id) {
        self.api_sim_id = api_sim_id;
    }

    pub fn presimulation_check(&self) {
        assert_ne!(self.api_sim_id, dsc::Id::MAX);
    }

    pub fn self_update_on(&mut self) {
        if !self.self_update_enabled {
            self.self_update_enabled = true;
            self.ctx.emit_self(APISchedulerSelfUpdate {}, self.cluster_state.borrow().constants.scheduler_self_update_period);
        }
    }


    pub fn schedule(&mut self) {
        let mut result: Vec<Node> = Vec::new();
        let mut score_matrix: Vec<Vec<i64>> = vec![vec![0; self.nodes.len()]; NSCORE];

        while let Some(wrapper) = self.active_queue.pop() {
            let mut pod = wrapper.inner();
            let pod_uid = pod.metadata.uid;
            let cpu = pod.spec.request_cpu;
            let memory = pod.spec.request_memory;


            // Query all suitable nodes
            self.node_rtree.find_suitable_nodes(cpu, memory, &mut result);

            // Apply node selector
            result.retain(|node| node_selector(
                &self.running_pods, &self.pending_pods, &self.nodes, &pod, node
            ));


            // Filter
            for filter_plugin in self.filters.iter() {
                result.retain(|node| filter_plugin(
                    &self.running_pods, &self.pending_pods, &self.nodes, &pod, node
                ));
            }


            // TODO: if result is empty run PostFilter
            if result.len() == 0 {
                // Place pod to BackOffQ and increase backoff attempts
                let attempts = self.failed_attempts.entry(pod_uid).or_default();
                self.backoff_queue.push(pod_uid, *attempts, self.ctx.time());
                *attempts += 1;

                continue;
            }


            // Score
            for (i, score_plugin) in self.scorers.iter().enumerate() {
                for (j, node) in result.iter().enumerate() {
                    score_matrix[i][j] = score_plugin(
                        &self.running_pods, &self.pending_pods, &self.nodes, &pod, node
                    );
                }
            }


            // Normalize Score
            for (i, score_normalizer) in self.score_normalizers.iter().enumerate() {
                score_normalizer(
                    &self.running_pods, &self.pending_pods, &self.nodes, &pod, &result, &mut score_matrix[i]
                );
            }


            // Find best node
            let mut best_node_index: usize = 0;
            let mut max_score: i64 = 0;
            for i in 0..NSCORE {
                max_score += score_matrix[i][0] * self.scorer_weights[i];
            }

            let mut tmp_score: i64;
            for i in 0..result.len() {
                tmp_score = 0;
                for j in 0..NSCORE {
                    tmp_score += score_matrix[j][i]  * self.scorer_weights[i];
                }
                if tmp_score > max_score {
                    max_score = tmp_score;
                    best_node_index = i;
                }
            }

            let node_uid = result[best_node_index].metadata.uid;


            // Place pod to node

            assert!(self.nodes.get(&node_uid).unwrap().is_consumable(cpu, memory));
            // println!("{2} Assign pod_{0} to node_{1}", pod_uid, node_uid, self.ctx.time());

            // Move cached pod from pending to running
            let mut cached = self.pending_pods.remove(&pod_uid).unwrap();
            cached.status.phase = PodPhase::Running;
            cached.status.node_uid = Some(node_uid);
            self.running_pods.insert(pod_uid, cached);

            // Update pod status
            pod.status.node_uid = Some(node_uid);
            pod.status.phase = PodPhase::Running;

            // Consume node resources
            self.place_pod_to_node(pod_uid, node_uid, cpu, memory);

            let data = APIUpdatePodFromScheduler {
                pod,
                new_phase: PodPhase::Running,
                node_uid,
            };

            // println!("{:?} Scheduler Pod_{:?} placed to Node_{:?} artime: {:?}", self.ctx.time(), pod.metadata.uid, node.metadata.uid, pod.spec.arrival_time);
            self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.scheduler2api);
        }

        // TODO: Implement API event with time period
        while let Some(pod_uid) = self.backoff_queue.try_pop(self.ctx.time()) {
            self.active_queue.push(ActiveQCmp::wrap(self.pending_pods.get(&pod_uid).unwrap().clone()));
        }
    }


    pub fn is_node_consumable(node: &Node, cpu: u64, memory: u64) -> bool {
        return cpu <= node.spec.available_cpu && memory <= node.spec.available_memory;
    }

    pub fn place_pod_to_node(&mut self, pod_uid: u64, node_uid: u64, cpu: u64, memory: u64) {
        let node = self.nodes.get_mut(&node_uid).unwrap();

        self.node_rtree.remove(&node);

        node.consume(cpu, memory);
        let _not_presented = node.status.pods.insert(pod_uid); assert!(_not_presented);

        self.node_rtree.insert(node.clone());

        self.monitoring.borrow_mut().scheduler_on_node_consume(cpu, memory);
    }

    pub fn remove_pod_from_node(&mut self, pod_uid: u64, node_uid: u64, cpu: u64, memory: u64) {
        let node = self.nodes.get_mut(&node_uid).unwrap();

        self.node_rtree.remove(&node);

        node.restore(cpu, memory);
        let _was_presented = node.status.pods.remove(&pod_uid); assert!(_was_presented);

        self.node_rtree.insert(node.clone());

        self.monitoring.borrow_mut().scheduler_on_node_restore(cpu, memory);
    }


    pub fn process_new_pod(&mut self, pod: Pod) {
        let pod_uid = pod.metadata.uid;

        assert_eq!(self.running_pods.contains_key(&pod_uid), false);
        assert_eq!(self.pending_pods.contains_key(&pod_uid), false);
        assert_eq!(pod.status.phase, PodPhase::Pending);

        self.pending_pods.insert(pod_uid, pod.clone());
        self.active_queue.push(ActiveQCmp::wrap(pod));
    }

    pub fn process_evicted_pod(&mut self, pod_uid: u64) {
        assert_eq!(self.running_pods.contains_key(&pod_uid), true);
        assert_eq!(self.pending_pods.contains_key(&pod_uid), false);

        // Remove pod from running set
        let mut pod = self.running_pods.remove(&pod_uid).unwrap();

        // Restore node resources
        let node_uid = pod.status.node_uid.unwrap();
        self.remove_pod_from_node(pod_uid, node_uid, pod.spec.request_cpu, pod.spec.request_memory);
        pod.status.node_uid = None;

        // Place pod to pending set
        pod.status.phase = PodPhase::Pending;
        self.pending_pods.insert(pod_uid, pod.clone());

        // Place pod to ActiveQ
        self.active_queue.push(ActiveQCmp::wrap(pod));

        // Place pod to BackOffQ
        // self.backoff_queue.push(pod_uid, *attempts - 1, self.ctx.time());
    }

    pub fn process_finished_pod(&mut self, pod_uid: u64) {
        assert_eq!(self.running_pods.contains_key(&pod_uid), true);
        assert_eq!(self.pending_pods.contains_key(&pod_uid), false);

        // Remove pod from running
        let pod = self.running_pods.remove(&pod_uid).unwrap();

        // Restore node resources
        let node_uid = pod.status.node_uid.unwrap();
        self.remove_pod_from_node(pod_uid, node_uid, pod.spec.request_cpu, pod.spec.request_memory);

        // Remove pod's filed attempts
        self.failed_attempts.remove(&pod_uid);
    }
}

impl <
    ActiveQCmp: TraitActiveQCmp,
    BackOffQ: TraitBackOffQ,
    const NFILTER: usize,
    const NSCORE: usize,
> dsc::EventHandler for Scheduler<ActiveQCmp, BackOffQ, NFILTER, NSCORE> {
    fn on(&mut self, event: dsc::Event) {
        if self.ctx.time() > 65641.0 {
            debug_print!("Scheduler EventHandler ------>");
        }
        dsc::cast!(match event.data {
            APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid: _ } => {
                if self.ctx.time() >= 65640.0 {
                    debug_print!("{:?} Scheduler <Update Pod From Kubelet> pod_{:?} new_phase: {:?}", self.ctx.time(), pod_uid, new_phase);
                }

                match new_phase {
                    PodPhase::Pending => {
                        self.process_evicted_pod(pod_uid);
                    }
                    PodPhase::Succeeded => {
                        self.process_finished_pod(pod_uid);
                    }
                    PodPhase::Running => {
                    }
                    PodPhase::Failed => {
                        panic!("Bad PodPhase Failed");
                    }
                    PodPhase::Unknown => {
                        panic!("Bad PodPhase Unknown");
                    }
                }

                // self.schedule();
                self.self_update_on();

                self.monitoring.borrow_mut().scheduler_update_pending_pod_count(self.pending_pods.len());
            }
            APIAddPod { pod } => {
                if self.ctx.time() >= 65640.0 {
                    debug_print!("Scheduler <Add Pod> {:?}", pod);
                }
                self.process_new_pod(pod);

                // self.schedule();
                self.self_update_on();

                self.monitoring.borrow_mut().scheduler_update_pending_pod_count(self.pending_pods.len());
            }
            APIAddNode { kubelet_sim_id: _ , node } => {
                if self.ctx.time() >= 65640.0 {
                    debug_print!("Scheduler <Add Kubelet> {:?}", node);
                }
                self.monitoring.borrow_mut().scheduler_on_node_added(&node);
                self.nodes.insert(node.metadata.uid, node.clone());
                self.node_rtree.insert(node);
            }
            APISchedulerSelfUpdate { } => {
                if self.ctx.time() >= 65640.0 {
                    debug_print!("Scheduler <Self Update>");
                }
                self.schedule();

                if self.pending_pods.len() > 0 {
                    self.ctx.emit_self(APISchedulerSelfUpdate{}, self.cluster_state.borrow().constants.scheduler_self_update_period);
                } else {
                    self.self_update_enabled = false;
                }
            }
        });
        if self.ctx.time() >= 65640.0 {
            debug_print!("Scheduler EventHandler <------");
        }
    }
}

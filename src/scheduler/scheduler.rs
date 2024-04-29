use crate::my_imports::*;
use crate::scheduler::filter;


pub struct Scheduler<
    ActiveQCmp,
    BackOffQ,
    const N_POST_FILTER: usize,
> {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    api_sim_id: dsc::Id,
    monitoring: Rc<RefCell<Monitoring>>,

    self_update_enabled: bool,

    // Cache
    running_pods: HashMap<u64, Pod>,
    pending_pods: HashMap<u64, Pod>,
    nodes: HashMap<u64, Node>,
    node_rtree: NodeRTree,

    // Queues
    active_queue: BinaryHeap<ActiveQCmp>,
    backoff_queue: BackOffQ,
    failed_attempts: HashMap<u64, u64>,

    // Pipeline
    filters: Vec<Box<dyn IFilterPlugin>>,
    post_filters: Vec<Box<dyn IFilterPlugin>>,
    scorers: Vec<Box<dyn IScorePlugin>>,
    scorer_weights: [i64; 1],
    score_normalizers: [NormalizeScorePluginT; 1],

    // To remove running pods
    removed_pod: HashSet<u64>,
}

impl <
    ActiveQCmp: TraitActiveQCmp,
    BackOffQ: TraitBackOffQ,
    const N_POST_FILTER: usize,
> Scheduler<ActiveQCmp, BackOffQ, N_POST_FILTER> {
    pub fn new(
        ctx: dsc::SimulationContext,
        cluster_state: Rc<RefCell<ClusterState>>,
        monitoring: Rc<RefCell<Monitoring>>,
        filters: Vec<Box<dyn IFilterPlugin>>,
        post_filters: Vec<Box<dyn IFilterPlugin>>,
        //filters: [FilterPluginT; N_FILTER],
        //post_filters: [FilterPluginT; N_POST_FILTER],
        scorers: Vec<Box<dyn IScorePlugin>>,
        score_normalizers: [NormalizeScorePluginT; 1],
        scorer_weights: [i64; 1],
        backoff_queue: BackOffQ,
    ) -> Scheduler<ActiveQCmp, BackOffQ, N_POST_FILTER> {
        let mut filter_names: Vec<String> = Vec::with_capacity(filters.len());
        filters[0].name();

        // for filter in filters {
        //     filter.name();
        //     filter_names.push("aboba".to_string());
        // }

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
            post_filters,
            scorers,
            score_normalizers,
            scorer_weights,
            removed_pod: HashSet::new(),
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
        // From backoffQ to activeQ
        while let Some(pod_uid) = self.backoff_queue.try_pop(self.ctx.time()) {
            self.active_queue.push(ActiveQCmp::wrap(self.pending_pods.get(&pod_uid).unwrap().clone()));
        }

        let mut possible_nodes: Vec<Node> = Vec::new();
        let mut resulted_nodes: Vec<Node> = Vec::new();
        let mut is_schedulable: Vec<bool> = Vec::new();
        let mut score_matrix: Vec<Vec<i64>> = vec![vec![0; self.nodes.len()]; self.scorers.len()];

        let (mut scheduled_left, mut try_schedule_left): (u64, u64) = (
            self.cluster_state.borrow().constants.scheduler_cycle_max_scheduled,
            self.cluster_state.borrow().constants.scheduler_cycle_max_to_try,
        );

        dp_scheduler!("{:.12} scheduler cycle activeQ:{:?}", self.ctx.time(), self.active_queue.len());

        // Main scheduling cycle
        while let Some(wrapper) = self.active_queue.pop() {
            if scheduled_left == 0 || try_schedule_left == 0 {
                break;
            }
            try_schedule_left -= 1;

            let mut pod = wrapper.inner();
            let pod_uid = pod.metadata.uid;
            let cpu = pod.spec.request_cpu;
            let memory = pod.spec.request_memory;


            // Query all suitable nodes
            self.node_rtree.find_suitable_nodes(cpu, memory, &mut possible_nodes);
            pod.status.cluster_resource_starvation = possible_nodes.is_empty();
            self.pending_pods.get_mut(&pod_uid).unwrap().status.cluster_resource_starvation = possible_nodes.is_empty();

            // Prepare node description
            is_schedulable.clear();
            is_schedulable.resize(possible_nodes.len(), true);


            // Filter
            let mut suitable_count: usize = 0;
            for (i, node) in possible_nodes.iter().enumerate() {
                for filter_plugin in self.filters.iter() {
                    // If node marked as infeasible, the remaining plugins will not be called
                    if !is_schedulable[i] {
                        break;
                    }

                    is_schedulable[i] = filter_plugin.filter(
                        &self.running_pods, &self.pending_pods, &self.nodes, &pod, node
                    );
                }

                if is_schedulable[i] {
                    suitable_count += 1;
                }
            }

            // Apply PostFilter if necessary
            if suitable_count == 0 {
                // for (i, node) in possible_nodes.iter().enumerate() {
                //     for post_filter_plugin in self.post_filters.iter() {
                //         is_schedulable[i] = post_filter_plugin(
                //             &self.running_pods, &self.pending_pods, &self.nodes, &pod, node
                //         );
                //
                //         //  If any marks the node as Schedulable, the remaining will not be called
                //         if is_schedulable[i] {
                //             break;
                //         }
                //     }
                //
                //     if is_schedulable[i] {
                //         suitable_count += 1;
                //     }
                // }
            }

            // If PostFilter does not help
            if suitable_count == 0 {
                let attempts = self.failed_attempts.entry(pod_uid).or_default();

                if *attempts == 0 {
                    // Simulate UnschedulableQ
                    self.ctx.emit_self(APISchedulerSecondChance { pod_uid }, self.cluster_state.borrow().constants.unschedulable_queue_period);
                } else {
                    // Place pod to BackoffQ
                    self.backoff_queue.push(pod_uid, *attempts, self.ctx.time());
                }

                *attempts += 1;

                continue;
            }

            // Prepare schedulable nodes
            resulted_nodes.clear();
            resulted_nodes.reserve(suitable_count);
            for i in 0..possible_nodes.len() {
                if is_schedulable[i] {
                    resulted_nodes.push(possible_nodes[i].clone());
                }
            }
            assert_eq!(resulted_nodes.len(), suitable_count);


            // Score
            for (i, score_plugin) in self.scorers.iter().enumerate() {
                for (j, node) in resulted_nodes.iter().enumerate() {
                    score_matrix[i][j] = score_plugin.score(
                        &self.running_pods, &self.pending_pods, &self.nodes, &pod, node
                    );
                }
            }


            // Normalize Score
            for (i, score_normalizer) in self.score_normalizers.iter().enumerate() {
                score_normalizer(
                    &self.running_pods, &self.pending_pods, &self.nodes, &pod, &resulted_nodes, &mut score_matrix[i]
                );
            }


            // Find best node
            let mut best_node_index: usize = 0;
            let mut max_score: i64 = 0;
            for i in 0..self.scorers.len() {
                max_score += score_matrix[i][0] * self.scorer_weights[i];
            }

            let mut tmp_score: i64;
            for i in 0..resulted_nodes.len() {
                tmp_score = 0;
                for j in 0..self.scorers.len() {
                    tmp_score += score_matrix[j][i] * self.scorer_weights[j];
                }
                if tmp_score > max_score {
                    max_score = tmp_score;
                    best_node_index = i;
                }
            }

            let node_uid = resulted_nodes[best_node_index].metadata.uid;


            // Place pod to node
            assert!(self.nodes.get(&node_uid).unwrap().is_consumable(cpu, memory));

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

            // Clear failed attempts
            self.failed_attempts.remove(&pod_uid);

            let data = APIUpdatePodFromScheduler {
                pod,
                new_phase: PodPhase::Running,
                node_uid,
            };

            dp_scheduler!("{:.12} scheduler pod_uid:{:?} placed -> node_uid:{:?}", self.ctx.time(), pod_uid, node_uid);
            self.ctx.emit(data, self.api_sim_id, self.cluster_state.borrow().network_delays.scheduler2api);
            scheduled_left -= 1;
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
        assert_eq!(pod.status.cluster_resource_starvation, false);

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
        if self.nodes.contains_key(&node_uid) {
            self.remove_pod_from_node(pod_uid, node_uid, pod.spec.request_cpu, pod.spec.request_memory);
            pod.status.node_uid = None;
        }

        // If pod was removed by api-server -> drop pod
        if self.removed_pod.contains(&pod_uid) {
            self.removed_pod.remove(&pod_uid);
            return;
        }

        // Place pod to pending set
        pod.status.phase = PodPhase::Pending;
        self.pending_pods.insert(pod_uid, pod.clone());

        // Place pod to ActiveQ
        self.active_queue.push(ActiveQCmp::wrap(pod));
    }

    pub fn process_finished_pod(&mut self, pod_uid: u64) {
        assert_eq!(self.running_pods.contains_key(&pod_uid), true);
        assert_eq!(self.pending_pods.contains_key(&pod_uid), false);

        // Remove pod from running
        let pod = self.running_pods.remove(&pod_uid).unwrap();

        // Restore node resources
        let node_uid = pod.status.node_uid.unwrap();
        if self.nodes.contains_key(&node_uid) {
                self.remove_pod_from_node(pod_uid, node_uid, pod.spec.request_cpu, pod.spec.request_memory);
        }

        // Remove pod's failed attempts
        self.failed_attempts.remove(&pod_uid);
    }


    pub fn send_ca_metrics(&mut self, node_list: &Vec<u64>) {
        let mut pending = 0;
        let mut requests: Vec<(u64, u64)> = Vec::new();
        for (_, pod) in &self.pending_pods {
            if pod.status.cluster_resource_starvation {
                pending += 1;
                requests.push((pod.spec.request_cpu, pod.spec.request_memory));
            }
        }

        let mut node_info: Vec<(u64, f64, f64)> = Vec::new();
        for node_uid in node_list {
            let node = self.nodes.get(node_uid);
            if node.is_some() {
                let spec = node.unwrap().spec.clone();
                let cpu: f64 = ((spec.installed_cpu - spec.available_cpu) as f64 * 100.0) / (spec.installed_cpu as f64);
                let memory: f64 = ((spec.installed_memory - spec.available_memory) as f64 * 100.0) / (spec.installed_memory as f64);

                node_info.push((*node_uid, cpu, memory));
            }
        }

        self.ctx.emit(
            APIPostCAMetrics {
                insufficient_resources_pending: pending,
                requests,
                node_info,
            }, self.api_sim_id, self.cluster_state.borrow().network_delays.scheduler2api
        );
    }

    pub fn preempt_pod(&mut self, pod_uid: u64) {
        // If pod in pending -> do nothing
        if self.pending_pods.contains_key(&pod_uid) {
            return;
        }

        // If pod in running -> preempt
        if self.running_pods.contains_key(&pod_uid) {
            let pod = self.running_pods.get(&pod_uid).unwrap();
            self.ctx.emit(
                APIUpdatePodFromScheduler { pod: pod.clone(), new_phase: PodPhase::Pending, node_uid: pod.status.node_uid.unwrap() },
                self.api_sim_id,
                self.cluster_state.borrow().network_delays.scheduler2api
            );
        }
    }
}

impl <
    ActiveQCmp: TraitActiveQCmp,
    BackOffQ: TraitBackOffQ,
    const N_POST_FILTER: usize,
> dsc::EventHandler for Scheduler<ActiveQCmp, BackOffQ, N_POST_FILTER> {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            APIUpdatePodFromKubelet { pod_uid, new_phase, node_uid: _node_uid } => {
                dp_scheduler!("{:.12} scheduler APIUpdatePodFromKubelet pod_uid:{:?} node_uid:{:?} new_phase:{:?}", self.ctx.time(), pod_uid, _node_uid, new_phase);

                match new_phase {
                    PodPhase::Pending => {
                        self.process_evicted_pod(pod_uid);
                        self.monitoring.borrow_mut().scheduler_on_pod_evicted();
                    }
                    PodPhase::Succeeded => {
                        self.process_finished_pod(pod_uid);
                        self.monitoring.borrow_mut().scheduler_on_pod_succeed();
                    }
                    PodPhase::Running => {
                        panic!("Bad Logic PodPhase Running");
                    }
                    PodPhase::Failed => {
                        self.process_finished_pod(pod_uid);
                        self.monitoring.borrow_mut().scheduler_on_pod_failed();
                    }
                }

                self.self_update_on();
                self.monitoring.borrow_mut().scheduler_update_pending_pod_count(self.pending_pods.len());
                self.monitoring.borrow_mut().scheduler_update_running_pod_count(self.running_pods.len());
            }
            APIAddPod { pod } => {
                dp_scheduler!("{:.12} scheduler APIAddPod pod_uid:{:?}", self.ctx.time(), pod.metadata.uid);

                self.process_new_pod(pod);
                self.self_update_on();
                self.monitoring.borrow_mut().scheduler_update_pending_pod_count(self.pending_pods.len());
                self.monitoring.borrow_mut().scheduler_update_running_pod_count(self.running_pods.len());
            }
            APIRemovePod { pod_uid } => {
                dp_scheduler!("{:.12} scheduler APIRemovePod pod_uid:{:?}", self.ctx.time(), pod_uid);

                if self.running_pods.contains_key(&pod_uid) || self.pending_pods.contains_key(&pod_uid) {
                    self.removed_pod.insert(pod_uid);
                    self.preempt_pod(pod_uid);

                    self.monitoring.borrow_mut().scheduler_update_pending_pod_count(self.pending_pods.len());
                    self.monitoring.borrow_mut().scheduler_update_running_pod_count(self.running_pods.len());
                }
            }
            APIAddNode { kubelet_sim_id: _ , node } => {
                dp_scheduler!("{:.12} scheduler APIAddNode node_uid:{:?}", self.ctx.time(), node.metadata.uid);

                self.monitoring.borrow_mut().scheduler_on_node_added(&node);
                self.nodes.insert(node.metadata.uid, node.clone());
                self.node_rtree.insert(node);
            }
            APIRemoveNode { node_uid } => {
                dp_scheduler!("{:.12} scheduler APIRemoveNode node_uid:{:?}", self.ctx.time(), node_uid);

                let node = self.nodes.remove(&node_uid).unwrap();
                self.node_rtree.remove(&node);
                self.monitoring.borrow_mut().scheduler_on_node_removed(&node);
            }
            APISchedulerSelfUpdate { } => {
                dp_scheduler!("{:.12} scheduler APISchedulerSelfUpdate", self.ctx.time());

                self.schedule();
                self.monitoring.borrow_mut().scheduler_update_pending_pod_count(self.pending_pods.len());
                self.monitoring.borrow_mut().scheduler_update_running_pod_count(self.running_pods.len());

                if self.pending_pods.len() > 0 {
                    self.ctx.emit_self(APISchedulerSelfUpdate{}, self.cluster_state.borrow().constants.scheduler_self_update_period);
                } else {
                    self.self_update_enabled = false;
                }
            }
            APISchedulerSecondChance { pod_uid } => {
                dp_scheduler!("{:.12} scheduler APISchedulerSecondChance pod_uid:{:?}", self.ctx.time(), pod_uid);

                self.active_queue.push(ActiveQCmp::wrap(self.pending_pods.get(&pod_uid).unwrap().clone()));
            }
            APIGetCAMetrics { node_list } => {
                dp_scheduler!("{:.12} scheduler APIGetCAMetrics", self.ctx.time());

                self.send_ca_metrics(&node_list);
            }
        });
    }
}

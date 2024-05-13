use crate::my_imports::*;

pub struct Scheduler {
    ctx: dsc::SimulationContext,
    init_config: Rc<RefCell<InitConfig>>,
    monitoring: Rc<RefCell<Monitoring>>,

    api_sim_id: dsc::Id,

    self_update_enabled: bool,

    // Cache
    running_pods: HashMap<u64, Pod>, // HashMap<pod_uid, Pod>
    pending_pods: HashMap<u64, Pod>, // HashMap<pod_uid, Pod>
    nodes: HashMap<u64, Node>,       // HashMap<node_uid, Node>
    node_rtree: NodeRTree,

    // Queues
    active_queue: Box<dyn IActiveQ + Send>,
    unschedulable_queue: BackOffQConstant,
    backoff_queue: Box<dyn IBackOffQ + Send>,
    failed_attempts: HashMap<u64, u64>,

    // Pipeline
    filters: Vec<Box<dyn IFilterPlugin + Send>>,
    post_filters: Vec<Box<dyn IFilterPlugin + Send>>,
    scorers: Vec<Box<dyn IScorePlugin + Send>>,
    score_normalizers: Vec<Box<dyn IScoreNormalizePlugin + Send>>,
    scorer_weights: Vec<i64>,
}

impl Scheduler {
    pub fn new(
        ctx: dsc::SimulationContext,
        init_config: Rc<RefCell<InitConfig>>,
        monitoring: Rc<RefCell<Monitoring>>,
        api_sim_id: dsc::Id,

        // Queues
        active_queue: Box<dyn IActiveQ + Send>,
        backoff_queue: Box<dyn IBackOffQ + Send>,

        // Pipline
        filters: Vec<Box<dyn IFilterPlugin + Send>>,
        post_filters: Vec<Box<dyn IFilterPlugin + Send>>,
        scorers: Vec<Box<dyn IScorePlugin + Send>>,
        score_normalizers: Vec<Box<dyn IScoreNormalizePlugin + Send>>,
        scorer_weights: Vec<i64>,
    ) -> Scheduler {
        Self {
            ctx,
            init_config: init_config.clone(),
            api_sim_id,
            monitoring,
            self_update_enabled: false,

            // Cache
            running_pods: HashMap::new(),
            pending_pods: HashMap::new(),
            nodes: HashMap::new(),
            node_rtree: NodeRTree::new(),

            // Queues
            active_queue,
            unschedulable_queue: BackOffQConstant::new(
                init_config.borrow().scheduler.unschedulable_queue_backoff_delay,
            ),
            backoff_queue,
            failed_attempts: HashMap::new(),

            // Pipeline
            filters,
            post_filters,
            scorers,
            score_normalizers,
            scorer_weights,
        }
    }

    ////////////////// SelfUpdate controller //////////////////

    pub fn self_update_on(&mut self) {
        if !self.self_update_enabled {
            self.self_update_enabled = true;
            self.ctx.emit_self(
                EventSelfUpdate {},
                self.init_config.borrow().scheduler.self_update_period,
            );
        }
    }

    pub fn self_update_off(&mut self) {
        self.self_update_enabled = false;
    }

    ////////////////// Main scheduling cycle //////////////////

    pub fn schedule(&mut self) {
        // From unschedulableQ to activeQ
        while let Some(pod_uid) = self.unschedulable_queue.try_pop(self.ctx.time()) {
            self.active_queue.push(self.pending_pods.get(&pod_uid).unwrap().clone());
        }

        // From backoffQ to activeQ
        while let Some(pod_uid) = self.backoff_queue.try_pop(self.ctx.time()) {
            self.active_queue.push(self.pending_pods.get(&pod_uid).unwrap().clone());
        }

        let mut possible_nodes: Vec<Node> = Vec::new();
        let mut resulted_nodes: Vec<Node> = Vec::new();
        let mut is_schedulable: Vec<bool> = Vec::new();
        let mut score_matrix: Vec<Vec<i64>> = vec![vec![0; self.nodes.len()]; self.scorers.len()];

        let (mut scheduled_left, mut try_schedule_left): (u64, u64) = (
            self.init_config.borrow().scheduler.cycle_max_scheduled,
            self.init_config.borrow().scheduler.cycle_max_to_try,
        );

        dp_scheduler!(
            "{:.3} scheduler cycle activeQ:{:?}",
            self.ctx.time(),
            self.active_queue.len()
        );

        // Main scheduling cycle
        while let Some(mut pod) = self.active_queue.try_pop() {
            if scheduled_left == 0 || try_schedule_left == 0 {
                break;
            }
            try_schedule_left -= 1;

            let pod_uid = pod.metadata.uid;
            let cpu = pod.spec.request_cpu;
            let memory = pod.spec.request_memory;

            // Query all suitable nodes
            self.node_rtree.find_suitable_nodes(cpu, memory, &mut possible_nodes);
            pod.status.cluster_resource_starvation = possible_nodes.is_empty();
            self.pending_pods
                .get_mut(&pod_uid)
                .unwrap()
                .status
                .cluster_resource_starvation = possible_nodes.is_empty();

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

                    is_schedulable[i] =
                        filter_plugin.filter(&self.running_pods, &self.pending_pods, &self.nodes, &pod, node);
                }

                if is_schedulable[i] {
                    suitable_count += 1;
                }
            }

            // Apply PostFilter if necessary
            if suitable_count == 0 {
                for (i, node) in possible_nodes.iter().enumerate() {
                    for post_filter_plugin in self.post_filters.iter() {
                        is_schedulable[i] =
                            post_filter_plugin.filter(&self.running_pods, &self.pending_pods, &self.nodes, &pod, node);

                        //  If any marks the node as Schedulable, the remaining will not be called
                        if is_schedulable[i] {
                            break;
                        }
                    }

                    if is_schedulable[i] {
                        suitable_count += 1;
                    }
                }
            }

            // If PostFilter does not help
            if suitable_count == 0 {
                let attempts = self.failed_attempts.entry(pod_uid).or_default();

                if *attempts == 0 {
                    // Place pod to UnschedulableQ
                    self.unschedulable_queue.push(pod_uid, 0, self.ctx.time());
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
                    score_matrix[i][j] =
                        score_plugin.score(&self.running_pods, &self.pending_pods, &self.nodes, &pod, node);
                }
            }

            // Normalize Score
            for (i, score_normalizer) in self.score_normalizers.iter().enumerate() {
                score_normalizer.normalize(
                    &self.running_pods,
                    &self.pending_pods,
                    &self.nodes,
                    &pod,
                    &resulted_nodes,
                    &mut score_matrix[i],
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

            // Get best node
            let node_uid = resulted_nodes[best_node_index].metadata.uid;
            let node = self.nodes.get(&node_uid).unwrap();

            // If not enough resources -> build preemption list
            let mut preempt_uids: Vec<u64> = Vec::new();
            if !node.is_both_consumable(cpu, memory) {
                let (mut cpu, mut memory) = (node.spec.available_cpu, node.spec.available_memory);
                for &tmp_uid in node.status.pods.iter() {
                    let tmp_pod = self.running_pods.get(&tmp_uid).unwrap();
                    if tmp_pod.spec.priority >= pod.spec.priority {
                        continue;
                    }

                    cpu += tmp_pod.spec.request_cpu;
                    memory += tmp_pod.spec.request_memory;
                    preempt_uids.push(tmp_uid);

                    if cpu >= pod.spec.request_cpu && memory >= pod.spec.request_memory {
                        break;
                    }
                }
            }

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

            // Send PodPhase update
            self.send_pod_phase_update(Some(pod), pod_uid, Some(preempt_uids), node_uid, PodPhase::Running);
            scheduled_left -= 1;

            dp_scheduler!(
                "{:.3} scheduler pod_uid:{:?} placed -> node_uid:{:?}",
                self.ctx.time(),
                pod_uid,
                node_uid
            );
        }
    }

    ////////////////// Helpers for Node cache and RTree cache //////////////////

    pub fn is_node_consumable(node: &Node, cpu: i64, memory: i64) -> bool {
        return cpu <= node.spec.available_cpu && memory <= node.spec.available_memory;
    }

    pub fn place_pod_to_node(&mut self, pod_uid: u64, node_uid: u64, cpu: i64, memory: i64) {
        // Get pod's node
        let node = self.nodes.get_mut(&node_uid).unwrap();

        // Remove node from RTree
        self.node_rtree.remove(&node);

        // Update node
        node.consume(cpu, memory);
        let _not_presented = node.status.pods.insert(pod_uid);
        assert!(_not_presented);

        // Add node to RTree
        self.node_rtree.insert(node.clone());

        // Update monitoring
        self.monitoring.borrow_mut().scheduler_on_node_consume(cpu, memory);
    }

    pub fn remove_pod_from_node(&mut self, pod_uid: u64, node_uid: u64, cpu: i64, memory: i64) {
        // Get pod's node
        let node = self.nodes.get_mut(&node_uid).unwrap();

        // Remove node from RTree
        self.node_rtree.remove(&node);

        // Update node
        node.restore(cpu, memory);
        let _was_presented = node.status.pods.remove(&pod_uid);
        assert!(_was_presented);

        // Add node to RTree
        self.node_rtree.insert(node.clone());

        // Update monitoring
        self.monitoring.borrow_mut().scheduler_on_node_restore(cpu, memory);
    }

    ////////////////// Process pod by phase ////////////////////////////////////

    pub fn process_new_pod(&mut self, pod: Pod) {
        let pod_uid = pod.metadata.uid;

        assert_eq!(self.running_pods.contains_key(&pod_uid), false);
        assert_eq!(self.pending_pods.contains_key(&pod_uid), false);
        assert_eq!(pod.status.phase, PodPhase::Pending);
        assert_eq!(pod.status.cluster_resource_starvation, false);

        self.pending_pods.insert(pod_uid, pod.clone());
        self.active_queue.push(pod);
    }

    pub fn process_reschedule_pod(&mut self, pod_uid: u64, phase: PodPhase) {
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

        // Place pod to pending set
        pod.status.phase = PodPhase::Pending;
        self.pending_pods.insert(pod_uid, pod.clone());

        // Place pod to ActiveQ
        self.active_queue.push(pod);

        // Update monitoring
        match phase {
            PodPhase::Evicted => {
                self.monitoring.borrow_mut().scheduler_on_pod_evicted();
            }
            PodPhase::Preempted => {
                self.monitoring.borrow_mut().scheduler_on_pod_preempted();
            }
            PodPhase::Pending => {
                // Do nothing
            }
            _ => {
                panic!("Logic error. This fn can be called only with phase Pending, Evicted or Preempted.");
            }
        }
    }

    pub fn process_finished_pod(&mut self, pod_uid: u64, phase: PodPhase) {
        assert_eq!(self.running_pods.contains_key(&pod_uid), true);
        assert_eq!(self.pending_pods.contains_key(&pod_uid), false);

        // Remove pod from running
        let pod = self.running_pods.remove(&pod_uid).unwrap();

        // Restore node resources if node exist
        let node_uid = pod.status.node_uid.unwrap();
        if self.nodes.contains_key(&node_uid) {
            self.remove_pod_from_node(pod_uid, node_uid, pod.spec.request_cpu, pod.spec.request_memory);
        }

        // Remove pod's failed attempts
        self.failed_attempts.remove(&pod_uid);

        // Update monitoring
        match phase {
            PodPhase::Succeeded => {
                self.monitoring.borrow_mut().scheduler_on_pod_succeed();
            }
            PodPhase::Failed => {
                self.monitoring.borrow_mut().scheduler_on_pod_failed();
            }
            _ => {
                panic!("Logic error. This fn can be called only with phase Succeeded or Failed.");
            }
        }
    }

    pub fn process_removed_pod(&mut self, pod_uid: u64) {
        // Remove pod's failed attempts
        self.failed_attempts.remove(&pod_uid);

        // Remove pod from cache
        match (self.running_pods.remove(&pod_uid), self.pending_pods.remove(&pod_uid)) {
            (Some(pod), None) => {
                // Restore node resources if node exist
                let node_uid = pod.status.node_uid.unwrap();
                if self.nodes.contains_key(&node_uid) {
                    self.remove_pod_from_node(pod_uid, node_uid, pod.spec.request_cpu, pod.spec.request_memory);
                }

                // Update monitoring
                self.monitoring.borrow_mut().scheduler_on_pod_removed();
            }
            (None, Some(pod)) => {
                // Try remove from BackOffQ or UnschedulableQ or ActiveQ
                let pod_uid = pod.metadata.uid;
                let result = self.backoff_queue.try_remove(pod_uid)
                    || self.unschedulable_queue.try_remove(pod_uid)
                    || self.active_queue.try_remove(pod);
                assert!(result, "Invariant violated. Pending pod not in any queue.");

                // Update monitoring
                self.monitoring.borrow_mut().scheduler_on_pod_removed()
            }
            (None, None) => {
                // Nothing to do
            }
            (Some(_), Some(_)) => {
                panic!("Invariant violated. Same pod in pending and running.");
            }
        }
    }

    ////////////////// Helpers //////////////////

    pub fn is_pod_cached(&self, pod_uid: u64) -> bool {
        return self.running_pods.contains_key(&pod_uid) || self.pending_pods.contains_key(&pod_uid);
    }

    ////////////////// Export metrics //////////////////

    pub fn send_pod_phase_update(
        &self,
        pod: Option<Pod>,
        pod_uid: u64,
        preempt_uids: Option<Vec<u64>>,
        node_uid: u64,
        new_phase: PodPhase,
    ) {
        self.ctx.emit(
            EventUpdatePodFromScheduler {
                pod,
                pod_uid,
                preempt_uids,
                new_phase,
                node_uid,
            },
            self.api_sim_id,
            self.init_config.borrow().network_delays.kubelet2api,
        );
    }

    pub fn count_and_send_ca_metrics(&mut self, used_nodes: &Vec<u64>, available_nodes: &Vec<NodeGroup>) {
        // For pending pods look available node which may help
        let mut may_help: Option<u64> = None;
        let mut pending_pod_count = 0;
        for (_, pod) in self.pending_pods.iter() {
            // Only pods which cannot be scheduled due to insufficient resources on nodes
            if !pod.status.cluster_resource_starvation {
                continue;
            }
            pending_pod_count += 1;

            // Try to find node which may help
            if may_help.is_none() {
                for node_group in available_nodes {
                    if node_group
                        .node
                        .is_both_consumable(pod.spec.request_cpu, pod.spec.request_memory)
                    {
                        may_help = Some(node_group.group_uid);
                        break;
                    }
                }
            }
        }

        // Count utilization for requested nodes
        let mut used_nodes_utilization: Vec<(u64, f64, f64)> = Vec::with_capacity(used_nodes.len());
        for node_uid in used_nodes {
            let node = self.nodes.get(node_uid);
            if node.is_some() {
                let spec = node.unwrap().spec.clone();
                let cpu: f64 = ((spec.installed_cpu - spec.available_cpu) as f64) / (spec.installed_cpu as f64);
                let memory: f64 =
                    ((spec.installed_memory - spec.available_memory) as f64) / (spec.installed_memory as f64);

                used_nodes_utilization.push((*node_uid, cpu, memory));
            }
        }

        // Send metrics
        self.ctx.emit(
            EventPostCAMetrics {
                pending_pod_count,
                used_nodes_utilization,
                may_help,
            },
            self.api_sim_id,
            self.init_config.borrow().network_delays.scheduler2api,
        );
    }
}

impl dsc::EventHandler for Scheduler {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            EventPodUpdateToScheduler { pod_uid, current_phase } => {
                dp_scheduler!(
                    "{:.3} scheduler EventPodUpdateFromKubelet pod_uid:{:?} current_phase:{:?}",
                    self.ctx.time(),
                    pod_uid,
                    current_phase
                );

                // If this pod was previously removed -> do nothing
                if !self.is_pod_cached(pod_uid) {
                    return;
                }

                // Process PodPhase
                match current_phase {
                    PodPhase::Succeeded | PodPhase::Failed => {
                        self.process_finished_pod(pod_uid, current_phase);
                    }
                    PodPhase::Pending | PodPhase::Evicted | PodPhase::Preempted => {
                        self.process_reschedule_pod(pod_uid, current_phase);

                        // New pending pod -> run self update
                        self.self_update_on();
                    }
                    PodPhase::Removed => {
                        // self.process_removed_pod(pod_uid);
                        // This process_removed_pod should already be called in EventRemovePod.
                        // So there this no more information about this pod and we cannot be here.
                        panic!("Logic error. PodPhase Removed is not expected.");
                    }
                    PodPhase::Running => {
                        // This PodPhase update should not reach the scheduler.
                        panic!("Logic error. PodPhase Running is not expected.");
                    }
                }
            }

            EventAddPod { pod } => {
                dp_scheduler!(
                    "{:.3} scheduler EventAddPod pod_uid:{:?}",
                    self.ctx.time(),
                    pod.metadata.uid
                );

                // Update inner state
                self.process_new_pod(pod);

                // New pending pod -> run self update
                self.self_update_on();
            }

            EventRemovePodGroup { group_uid } => {
                dp_scheduler!(
                    "{:.3} scheduler EventRemovePodGroup group_uid:{:?}",
                    self.ctx.time(),
                    group_uid
                );

                // Find all pods with current group_uid
                let mut pod2remove: Vec<u64> = Vec::new();
                // Search in pending pods
                for (&pod_uid, pod) in self.pending_pods.iter() {
                    if pod.metadata.group_uid == group_uid {
                        pod2remove.push(pod_uid);
                    }
                }
                // Search in running pods
                for (&pod_uid, pod) in self.running_pods.iter() {
                    if pod.metadata.group_uid == group_uid {
                        pod2remove.push(pod_uid);

                        //Notify kubelet to remove this pod
                        let node_uid = pod.status.node_uid.unwrap();
                        self.send_pod_phase_update(None, pod_uid, None, node_uid, PodPhase::Removed);
                    }
                }

                // Remove all found pods
                for pod_uid in pod2remove {
                    self.process_removed_pod(pod_uid);
                }
                // If we get PodPhase updates for these pods later -> do nothing.
            }

            EventRemovePod { pod_uid } => {
                dp_scheduler!("{:.3} scheduler EventRemovePod pod_uid:{:?}", self.ctx.time(), pod_uid);

                // If pod is running -> notify kubelet to remove this pod
                if self.running_pods.contains_key(&pod_uid) {
                    let node_uid = self.running_pods.get(&pod_uid).unwrap().status.node_uid.unwrap();
                    self.send_pod_phase_update(None, pod_uid, None, node_uid, PodPhase::Removed);
                }

                // Update inner state
                self.process_removed_pod(pod_uid);

                // If we get PodPhase updates for this pod later -> do nothing.
            }

            EventRemovePodGroup { group_uid: _group_uid } => {
                dp_scheduler!(
                    "{:.3} scheduler EventRemovePodGroup group_uid:{:?}",
                    self.ctx.time(),
                    _group_uid
                );
            }

            EventAddNode {
                kubelet_sim_id: _,
                node,
            } => {
                dp_scheduler!(
                    "{:.3} scheduler EventAddNode node_uid:{:?}",
                    self.ctx.time(),
                    node.metadata.uid
                );

                // Update monitoring
                self.monitoring.borrow_mut().scheduler_on_node_added(&node);

                // Update cache
                self.nodes.insert(node.metadata.uid, node.clone());
                self.node_rtree.insert(node);
            }

            EventRemoveNode { node_uid } => {
                dp_scheduler!(
                    "{:.3} scheduler EventRemoveNode node_uid:{:?}",
                    self.ctx.time(),
                    node_uid
                );

                // Update cache if necessary
                match self.nodes.remove(&node_uid) {
                    Some(node) => {
                        // Remove node from RTree
                        self.node_rtree.remove(&node);

                        // Update monitoring
                        self.monitoring.borrow_mut().scheduler_on_node_removed(&node);
                    }
                    None => {} // Nothing to do
                }

                // Here we only delete a node without doing anything with pods on this node.
                // We expect to get PodPhase updates for each pod on this node later.
                // After this update pods will be rescheduled.
            }

            EventSelfUpdate {} => {
                dp_scheduler!("{:.3} scheduler EventSelfUpdate", self.ctx.time());

                // Main scheduling cycle
                self.schedule();

                // If there are pending pods -> continue SelfUpdate
                if self.pending_pods.len() > 0 {
                    self.ctx.emit_self(
                        EventSelfUpdate {},
                        self.init_config.borrow().scheduler.self_update_period,
                    );
                } else {
                    self.self_update_off();
                }
            }

            EventGetCAMetrics {
                used_nodes,
                available_nodes,
            } => {
                dp_scheduler!("{:.3} scheduler EventGetCAMetrics", self.ctx.time());

                self.count_and_send_ca_metrics(&used_nodes, &available_nodes);
            }
        });

        // Update monitoring
        self.monitoring
            .borrow_mut()
            .scheduler_update_pending_pod_count(self.pending_pods.len());
        self.monitoring
            .borrow_mut()
            .scheduler_update_running_pod_count(self.running_pods.len());
    }
}

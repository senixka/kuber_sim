use crate::my_imports::*;


pub struct Experiment {
    // cluster_state_file_path: String,
    // workload_file_path: String,

    sim: dsc::Simulation,
    // seed: u64,

    // cluster_state: Rc<RefCell<ClusterState>>,
    // workload: Rc<RefCell<WorkLoad>>,

    // api: Rc<RefCell<APIServer>>,
    init: Rc<RefCell<Init>>,
    // monitoring: Rc<RefCell<Monitoring>>,

    // api_id: dsc::Id,
    // scheduler_id: dsc::Id,

    is_done: bool,

    ca: Rc<RefCell<CA>>,
    hpa: Rc<RefCell<HPA>>,
}


impl Experiment {
    pub fn new<
        ActiveQCmp: TraitActiveQCmp + 'static,
    > (
        cluster_state_file_path: &str,
        workload_file_path: &str,
        out_path: &str,
        seed: u64,
        active_q: Box<dyn IActiveQ>,
        back_off_q: Box<dyn IBackOffQ>,
        filters: Vec<Box<dyn IFilterPlugin>>,
        post_filters: Vec<Box<dyn IFilterPlugin>>,
        scorers: Vec<Box<dyn IScorePlugin>>,
        normalizers: Vec<Box<dyn IScoreNormalizePlugin>>,
        weights: Vec<i64>,
    ) -> Self {
        // Create components
        let cluster_state = Rc::new(RefCell::new(ClusterState::from_yaml(cluster_state_file_path)));

        let workload = Rc::new(RefCell::new(WorkLoad::from_file(workload_file_path)));

        let mut sim = dsc::Simulation::new(seed);

        let monitoring = Rc::new(RefCell::new(
            Monitoring::new(
                sim.create_context("monitoring"), cluster_state.clone(), out_path.to_string(),
            )
        ));
        let _ = sim.add_handler("monitoring", monitoring.clone());

        let api = Rc::new(RefCell::new(
            APIServer::new(
                sim.create_context("api"), cluster_state.clone()
            )
        ));
        let api_id = sim.add_handler("api", api.clone());

        let scheduler = Rc::new(RefCell::new(
            Scheduler::new(
                sim.create_context("scheduler"),
                cluster_state.clone(),
                monitoring.clone(),
                filters,
                post_filters,
                scorers,
                normalizers,
                weights,
                active_q,
                back_off_q,
            )
        ));
        let scheduler_id = sim.add_handler("scheduler", scheduler.clone());

        let init = Rc::new(RefCell::new(
            Init::new(
                sim.create_context("init"), cluster_state.clone(), workload.clone(), monitoring.clone()
            )
        ));

        let ca = Rc::new(RefCell::new(
            CA::new(
                sim.create_context("ca"),
                cluster_state.clone(),
                monitoring.clone(),
                api_id,
                &mut sim)
        ));
        let ca_id = sim.add_handler("ca", ca.clone());

        let hpa = Rc::new(RefCell::new(
            HPA::new(
                sim.create_context("hpa"),
                cluster_state.clone(),
                workload.clone(),
                api_id)
        ));
        let hpa_id = sim.add_handler("hpa", hpa.clone());

        // Init components
        api.borrow_mut().presimulation_init(scheduler_id, ca_id, hpa_id);
        scheduler.borrow_mut().presimulation_init(api_id);
        init.borrow_mut().presimulation_init(api_id);
        monitoring.borrow_mut().presimulation_init(ca_id, hpa_id);

        // Final check
        api.borrow().presimulation_check();
        scheduler.borrow().presimulation_check();
        init.borrow().presimulation_check();
        monitoring.borrow_mut().presimulation_check();

        Self {
            // cluster_state_file_path: cluster_state_file_path.to_string(),
            // workload_file_path: workload_file_path.to_string(),
            sim,// seed,
            // cluster_state, workload,
            init,// api, monitoring,
            // api_id, scheduler_id,
            is_done: false,
            ca,
            hpa,
        }
    }

    pub fn enable_cluster_autoscaler(&self) {
        self.ca.borrow_mut().turn_on();
    }

    pub fn disable_cluster_autoscaler(&self) {
        self.ca.borrow_mut().turn_off();
    }

    pub fn enable_hpa(&self) {
        self.hpa.borrow_mut().turn_on();
    }

    pub fn disable_hpa(&self) {
        self.hpa.borrow_mut().turn_off();
    }

    pub fn prepare_cluster(&mut self) {
        if self.is_done {
            panic!("prepare_cluster already done!");
        }
        self.is_done = true;

        self.init.borrow().submit_nodes(&mut self.sim);
        self.init.borrow().submit_pods();
    }

    pub fn step_until_no_events(&mut self) {
        if !self.is_done {
            panic!("Cluster is not prepared!");
        }

        self.sim.step_until_no_events();
    }

    pub fn run_for_duration(&mut self, duration: f64) {
        if !self.is_done {
            panic!("Cluster is not prepared!");
        }

        self.sim.step_for_duration(duration);
    }

    pub fn steps(&mut self, steps: u64) {
        if !self.is_done {
            panic!("Cluster is not prepared!");
        }

        self.sim.steps(steps);
    }

    pub fn step_until_time(&mut self, time: f64) {
        if !self.is_done {
            panic!("Cluster is not prepared!");
        }

        self.sim.step_until_time(time);
    }
}

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
}


impl Experiment {
    pub fn new(cluster_state_file_path: &str, workload_file_path: &str, out_path: &str, seed: u64) -> Self {
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
            Scheduler::<ActiveQCmpDefault, BackOffDefault, 3, 1, 1>::new(
                sim.create_context("scheduler"),
                cluster_state.clone(),
                monitoring.clone(),
                [filter_node_selector, filter_requested_resources_available, filter_taints_tolerations],
                [filter_taints_tolerations],
                [score_taints_and_tolerations],
                [skip],
                [1],
                BackOffDefault::default(),
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
            )
        ));
        let ca_id = sim.add_handler("ca", ca.clone());

        // Init components
        api.borrow_mut().presimulation_init(scheduler_id, ca_id);
        scheduler.borrow_mut().presimulation_init(api_id);
        init.borrow_mut().presimulation_init(api_id);
        monitoring.borrow_mut().presimulation_init(ca_id);

        // Final check
        api.borrow().presimulation_check();
        scheduler.borrow().presimulation_check();
        init.borrow().presimulation_check();
        monitoring.borrow_mut().presimulation_check();

        // Start CA
        ca.borrow_mut().turn_on();

        Self {
            // cluster_state_file_path: cluster_state_file_path.to_string(),
            // workload_file_path: workload_file_path.to_string(),
            sim,// seed,
            // cluster_state, workload,
            init,// api, monitoring,
            // api_id, scheduler_id,
            is_done: false,
        }
    }

    pub fn run(&mut self) {
        if self.is_done {
            panic!("Experiment already done!");
        }
        self.is_done = true;

        self.init.borrow().submit_nodes(&mut self.sim);
        self.init.borrow().submit_pods();

        self.sim.step_until_no_events();
    }
}

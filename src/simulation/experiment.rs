use std::cell::RefCell;
use std::rc::Rc;
use crate::init::Init;
use crate::my_imports::{APIServer, dsc, Scheduler};
use crate::scheduler::active_queue::ActiveQCmpUid;
use crate::scheduler::backoff_queue::BackOffQExponential;
use crate::sim_config;

pub struct Experiment {
    cluster_state_file_path: String,
    pods_load_file_path: String,

    sim: dsc::Simulation,
    seed: u64,

    api: Rc<RefCell<APIServer>>,
    scheduler: Rc<RefCell<Scheduler<ActiveQCmpUid, BackOffQExponential>>>,
    init: Rc<RefCell<Init>>,

    api_id: dsc::Id,
    scheduler_id: dsc::Id,
}

impl Experiment {
    pub fn new(cluster_state_file_path: &str, pods_load_file_path: &str, seed: u64) -> Self {
        let mut sim = dsc::Simulation::new(seed);

        let api = Rc::new(RefCell::new(APIServer::new(sim.create_context("api"))));
        let api_id = sim.add_handler("api", api.clone());

        let scheduler = Rc::new(RefCell::new(Scheduler::<ActiveQCmpUid, BackOffQExponential>::new(sim.create_context("scheduler"))));
        let scheduler_id = sim.add_handler("scheduler", scheduler.clone());

        let init = Rc::new(RefCell::new(Init::new(sim.create_context("init"))));

        // Init components

        sim_config::SimConfig::from_yaml(cluster_state_file_path);
        sim_config::NetworkDelays::from_yaml(pods_load_file_path);
        api.borrow_mut().presimulation_init(scheduler_id);
        scheduler.borrow_mut().presimulation_init(api_id);
        init.borrow_mut().presimulation_init(api_id);

        // Final check

        api.borrow().presimulation_check();
        scheduler.borrow().presimulation_check();
        init.borrow().presimulation_check();

        Self {
            cluster_state_file_path: cluster_state_file_path.to_string(),
            pods_load_file_path: pods_load_file_path.to_string(),
            sim, seed,
            api, scheduler, init,
            api_id, scheduler_id,
        }
    }

    pub fn run(&mut self) {
        self.init.borrow().submit_nodes(&mut self.sim);
        self.sim.step_until_no_events();

        self.init.borrow().submit_pods();
        self.sim.step_until_no_events();
    }
}
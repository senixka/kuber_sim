use std::cell::RefCell;
use std::rc::Rc;
use crate::simulation::init::*;
use crate::my_imports::{APIServer, dsc, Scheduler};
use crate::scheduler::active_queue::ActiveQCmpUid;
use crate::scheduler::backoff_queue::BackOffQExponential;
use crate::simulation::config::{ClusterState, WorkLoad};

pub struct Experiment {
    cluster_state_file_path: String,
    workload_file_path: String,

    sim: dsc::Simulation,
    seed: u64,

    cluster_state: Rc<RefCell<ClusterState>>,
    workload: Rc<RefCell<WorkLoad>>,

    api: Rc<RefCell<APIServer>>,
    scheduler: Rc<RefCell<Scheduler<ActiveQCmpUid, BackOffQExponential>>>,
    init: Rc<RefCell<Init>>,

    api_id: dsc::Id,
    scheduler_id: dsc::Id,

    is_done: bool,
}

impl Experiment {
    pub fn new(cluster_state_file_path: &str, workload_file_path: &str, seed: u64) -> Self {
        // Create components
        let cluster_state = Rc::new(RefCell::new(ClusterState::from_yaml(cluster_state_file_path)));
        let workload = Rc::new(RefCell::new(WorkLoad::from_yaml(workload_file_path)));

        let mut sim = dsc::Simulation::new(seed);

        let api = Rc::new(RefCell::new(
            APIServer::new(
                sim.create_context("api"), cluster_state.clone()
            )
        ));
        let api_id = sim.add_handler("api", api.clone());

        let scheduler = Rc::new(RefCell::new(
            Scheduler::<ActiveQCmpUid, BackOffQExponential>::new(
                sim.create_context("scheduler"), cluster_state.clone()
            )
        ));
        let scheduler_id = sim.add_handler("scheduler", scheduler.clone());

        let init = Rc::new(RefCell::new(
            Init::new(
                sim.create_context("init"), cluster_state.clone(), workload.clone()
            )
        ));

        // Init components
        api.borrow_mut().presimulation_init(scheduler_id);
        scheduler.borrow_mut().presimulation_init(api_id);
        init.borrow_mut().presimulation_init(api_id);

        // Final check
        api.borrow().presimulation_check();
        scheduler.borrow().presimulation_check();
        init.borrow().presimulation_check();

        Self {
            cluster_state_file_path: cluster_state_file_path.to_string(),
            workload_file_path: workload_file_path.to_string(),
            sim, seed,
            cluster_state, workload,
            api, scheduler, init,
            api_id, scheduler_id,
            is_done: false,
        }
    }

    pub fn run(&mut self) {
        if self.is_done {
            panic!("Experiment already done!");
        }
        self.is_done = true;

        self.init.borrow().submit_nodes(&mut self.sim);
        self.sim.step_until_no_events();

        self.init.borrow().submit_pods();
        self.sim.step_until_no_events();
    }
}

mod kubelet;
mod init;
mod sim_config;
mod scheduler;
mod load_types;
mod objects;
mod api_server;

pub mod my_imports {
    pub use std::rc::Rc;
    pub use std::cell::RefCell;

    pub use std::collections::HashMap;

    pub mod dsc {
        pub use dslab_core::{cast, Event, EventHandler, Id, Simulation, SimulationContext};
    }

    pub use crate::objects::pod::Pod;
    pub use crate::load_types::types::LoadType;
    pub use crate::load_types::busybox::BusyBox;
    pub use crate::load_types::constant::Constant;
    pub use crate::objects::object_meta::ObjectMeta;

    pub use crate::objects::pod::PodPhase;
    pub use crate::objects::node::Node;
    pub use crate::scheduler::scheduler::*;
    pub use crate::api_server::api::*;
    pub use crate::api_server::events::*;
    pub use crate::kubelet::Kubelet;
    pub use crate::sim_config::*;
    pub use crate::api_server::*;
}
use my_imports::*;
use crate::init::Init;
use crate::scheduler::active_queue::ActiveQCmpUid;
use crate::scheduler::backoff_queue::BackOffQExponential;

fn main() {
    let mut sim = dsc::Simulation::new(179);

    // Create components

    let api = Rc::new(RefCell::new(APIServer::new(sim.create_context("api"))));
    let api_id = sim.add_handler("api", api.clone());

    let scheduler = Rc::new(RefCell::new(Scheduler::<ActiveQCmpUid, BackOffQExponential>::new(sim.create_context("scheduler"))));
    let scheduler_id = sim.add_handler("scheduler", scheduler.clone());

    let init = Rc::new(RefCell::new(Init::new(sim.create_context("init"))));

    // Init components

    sim_config::SimConfig::from_yaml("./data/sim_config.yaml");
    sim_config::NetworkDelays::from_yaml("./data/sim_config.yaml");
    api.borrow_mut().presimulation_init(scheduler_id);
    scheduler.borrow_mut().presimulation_init(api_id);
    init.borrow_mut().presimulation_init(api_id);

    // Final check

    api.borrow().presimulation_check();
    scheduler.borrow().presimulation_check();
    init.borrow().presimulation_check();

    // Simulation

    init.borrow().submit_nodes(&mut sim);

    while sim.step() == true {
        println!("-------------------------------");
    }

    init.borrow().submit_pods();

    while sim.step() == true {
        println!("-------------------------------");
    }
}

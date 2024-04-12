mod kubelet;
mod scheduler;
mod load_types;
mod objects;
mod api_server;
mod simulation;

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
    pub use crate::simulation::config::*;
    pub use crate::api_server::*;
}

use std::io::stdin;
use my_imports::*;
use crate::scheduler::backoff_queue::{BackOffQExponential, TraitBackOffQ};
use crate::simulation::experiment::*;

fn main() {
    {
        // Test
        let mut backoff = BackOffQExponential::new(1.0, 10.0);
        backoff.push(1, 0, 0.0);
        assert_eq!(backoff.try_pop(0.5), None);
        assert_eq!(backoff.try_pop(1.1), Some(1));
    }

    let mut value = String::new();
    stdin().read_line(&mut value).unwrap();
    value = value.trim().to_string();

    if value == "1" {
        let mut test = Experiment::new(
            "./data/cluster_state/test_1.yaml",
            "./data/workload/test_1.yaml",
            179
        );
        test.run();
    }
    if value == "2" {
        let mut test = Experiment::new(
            "./data/cluster_state/state.yaml",
            "./data/workload/pods.yaml",
            179
        );
        test.run();
    }
}

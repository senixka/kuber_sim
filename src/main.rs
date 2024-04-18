mod kubelet;
mod scheduler;
mod load_types;
mod objects;
mod api_server;
mod simulation;
mod test;
mod my_macro;

pub mod my_imports {
    pub use std::rc::Rc;
    pub use std::cell::RefCell;

    pub use std::ops::Neg;
    pub use std::fs;
    pub use std::io::{stdin, BufRead, BufReader};
    pub use serde::{Deserialize, Serialize};
    pub use rstar::{AABB, RTree, RTreeObject};
    pub use std::sync::atomic::{AtomicU64, Ordering};
    pub use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet, BinaryHeap};

    pub mod dsc {
        pub use dslab_core::{cast, Event, EventHandler, Id, Simulation, SimulationContext, EPSILON};
    }

    pub use crate::api_server::api::*;
    pub use crate::api_server::events::*;

    pub use crate::load_types::types::*;
    pub use crate::load_types::constant::*;
    pub use crate::load_types::busybox::*;

    pub use crate::objects::pod::*;
    pub use crate::objects::pod_group::*;
    pub use crate::objects::node::*;
    pub use crate::objects::node_group::*;
    pub use crate::objects::object_meta::*;

    pub use crate::scheduler::active_queue::*;
    pub use crate::scheduler::backoff_queue::*;
    pub use crate::scheduler::filter::*;
    pub use crate::scheduler::node_index::*;
    pub use crate::scheduler::normalize_score::*;
    pub use crate::scheduler::scheduler::*;
    pub use crate::scheduler::score::*;

    pub use crate::simulation::workload::*;
    pub use crate::simulation::cluster_state::*;
    pub use crate::simulation::experiment::*;
    pub use crate::simulation::init::*;
    pub use crate::simulation::monitoring::*;

    pub use crate::kubelet::*;
    pub use crate::test::*;
    pub use crate::debug_print;
}
use my_imports::*;


fn main() {
    // println!("Node: {0}", size_of::<Node>());
    // println!("NodeSpec: {0}", size_of::<NodeSpec>());
    // println!("NodeStatus: {0}", size_of::<NodeStatus>());
    // println!("ObjectMeta: {0}", size_of::<ObjectMeta>());
    // println!("Pod: {0}", size_of::<Pod>());
    // println!("BTreeMap: {0}", size_of::<BTreeMap<String, String>>());
    // println!("HashMap: {0}", size_of::<HashMap<String, String>>());
    // println!("Vec: {0}", size_of::<Vec<(String, String)>>());
    // println!("List: {0}", size_of::<LinkedList<(String, String)>>());

    // WorkLoad::from_csv("./data/cluster_state/state.csv");

    debug_print!("Debug print Enabled");

    Test::test_all();

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
    if value == "3" {
        let mut test = Experiment::new(
            "./data/cluster_state/state.yaml",
            "./data/workload/pods.csv",
            179
        );
        test.run();
    }
}

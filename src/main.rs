mod kubelet;
mod scheduler;
mod load_types;
mod objects;
mod api_server;
mod simulation;
mod test;
mod my_macro;
mod autoscaler;

pub mod my_imports {
    pub use std::rc::Rc;
    pub use std::cell::RefCell;

    pub use std::ops::Neg;
    pub use std::fs;
    pub use std::fs::{File};
    pub use std::io::{stdin, BufRead, BufReader, BufWriter};
    pub use serde::{Deserialize, Serialize};
    pub use rstar::{AABB, RTree, RTreeObject};
    pub use std::sync::atomic::{AtomicU64, Ordering};
    pub use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet, BinaryHeap};

    pub mod dsc {
        pub use dslab_core::{cast, Event, EventHandler, Id, Simulation, SimulationContext, EPSILON};
    }

    pub use crate::api_server::api::*;
    pub use crate::api_server::events::*;

    pub use crate::autoscaler::ca::ca::*;
    pub use crate::autoscaler::hpa::hpa::*;
    pub use crate::autoscaler::hpa::hpa_profile::*;
    pub use crate::autoscaler::hpa::hpa_group_info::*;
    pub use crate::autoscaler::vpa::vpa::*;
    pub use crate::autoscaler::vpa::vpa_profile::*;
    pub use crate::autoscaler::vpa::vpa_pod_info::*;
    pub use crate::autoscaler::vpa::vpa_group_info::*;

    pub use crate::load_types::types::*;
    pub use crate::load_types::constant::*;
    pub use crate::load_types::constant_infinite::*;
    pub use crate::load_types::busybox::*;
    pub use crate::load_types::busybox_infinite::*;

    pub use crate::objects::pod::*;
    pub use crate::objects::pod_group::*;
    pub use crate::objects::node::*;
    pub use crate::objects::node_group::*;
    pub use crate::objects::object_meta::*;

    pub use crate::scheduler::queues::active_queue::*;
    pub use crate::scheduler::queues::active_queue_cmp::*;
    pub use crate::scheduler::queues::backoff_queue::*;
    pub use crate::scheduler::pipeline::filter::*;
    pub use crate::scheduler::node_index::*;
    pub use crate::scheduler::pipeline::score_normalize::*;
    pub use crate::scheduler::scheduler::*;
    pub use crate::scheduler::pipeline::score::*;
    pub use crate::scheduler::features::taints_tolerations::*;
    pub use crate::scheduler::features::node_affinity::*;

    pub use crate::simulation::workload::*;
    pub use crate::simulation::cluster_state::*;
    pub use crate::simulation::experiment::*;
    pub use crate::simulation::init::*;
    pub use crate::simulation::monitoring::*;

    pub use crate::kubelet::kubelet::*;
    pub use crate::kubelet::eviction::*;
    pub use crate::test::*;
    pub use crate::*;
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
    // println!("Some: {0}", std::mem::size_of::<u64>());

    // WorkLoad::from_csv("./data/cluster_state/state.csv");

    debug_print!("Enabled debug print all");
    dp_api_server!("Enabled debug print Api-Server");
    dp_scheduler!("Enabled debug print Scheduler");
    dp_kubelet!("Enabled debug print Kubelet");
    dp_ca!("Enabled debug print Cluster-Autoscaler");
    dp_hpa!("Enabled debug print HPA");

    // Integrity tests
    Test::test_all();

    let mut value = String::new();
    stdin().read_line(&mut value).unwrap();
    value = value.trim().to_string();
    // let value = "perf".to_string();

    // Test pod eviction
    // if value == "evict" {
    //     let mut test = Experiment::new::<ActiveQCmpDefault, BackOffDefault, 1, 1, 1>(
    //         "./data/cluster_state/test_evict.yaml",
    //         "./data/workload/test_evict.yaml",
    //         "./data/out/test_evict.txt",
    //         179,
    //             BackOffDefault::default(),
    //         [filter_node_affinity],
    //         [filter_node_affinity],
    //         [score_tetris],
    //         [skip],
    //         [1],
    //     );
    //     test.prepare_cluster();
    //     test.step_until_no_events();
    // }

    // // Test pod failures
    // if value == "failed" {
    //     let mut test = Experiment::new::<ActiveQCmpDefault, BackOffDefault, 0, 0, 0>(
    //         "./data/cluster_state/test_failed.yaml",
    //         "./data/workload/test_failed.yaml",
    //         "./data/out/test_failed.txt",
    //         179,
    //         BackOffDefault::default(),
    //         [],
    //         [],
    //         [],
    //         [],
    //         [],
    //     );
    //     test.prepare_cluster();
    //     test.step_until_no_events();
    // }

    // Test pod cluster autoscaler
    if value == "ca" {
        let mut test = Experiment::new(
            "./data/cluster_state/test_ca.yaml".to_string(),
            "./data/workload/test_ca.yaml".to_string(),
            "./data/out/test_ca.txt".to_string(),
            179,
        );
        test.add_scheduler(Box::new(ActiveQDefault::default()),
                           Box::new(BackOffQDefault::default()),
                           vec![Box::new(FilterNodeSelector)],
                           vec![Box::new(FilterAlwaysTrue)],
                           vec![Box::new(ScoreTetris)],
                           vec![Box::new(ScoreNormalizeSkip)],
                           vec![2]);
        test.add_ca();

        test.prepare();
        test.run_for_duration(100.0);

        test.enable_ca();
        test.run_for_duration(20.0);

        test.disable_ca();
        test.run_for_duration(100.0);

        test.enable_ca();
        test.run_for_duration(100.0);

    }

    // Test horizontal pod autoscaler
    if value == "vpa" {
        let mut test = Experiment::new(
            "./data/cluster_state/test_vpa.yaml".to_string(),
            "./data/workload/test_vpa.yaml".to_string(),
            "./data/out/test_vpa.txt".to_string(),
            179,
        );

        test.add_scheduler(Box::new(ActiveQDefault::default()),
                           Box::new(BackOffQDefault::default()),
                           vec![Box::new(FilterNodeSelector)],
                           vec![Box::new(FilterAlwaysTrue)],
                           vec![Box::new(ScoreTetris)],
                           vec![Box::new(ScoreNormalizeSkip)],
                           vec![2]);
        test.add_vpa();
        test.enable_dynamic_update();

        test.prepare();
        test.enable_vpa();
        // test.step_until_no_events();

        // test.enable_vpa();
        test.run_for_duration(40.0);
    }

    // Test horizontal pod autoscaler
    if value == "hpa" {
        let mut test = Experiment::new(
            "./data/cluster_state/test_hpa.yaml".to_string(),
            "./data/workload/test_hpa.yaml".to_string(),
            "./data/out/test_hpa.txt".to_string(),
            179,
        );

        test.add_scheduler(Box::new(ActiveQDefault::default()),
                           Box::new(BackOffQDefault::default()),
                           vec![Box::new(FilterNodeSelector)],
                           vec![Box::new(FilterAlwaysTrue)],
                           vec![Box::new(ScoreTetris)],
                           vec![Box::new(ScoreNormalizeSkip)],
                           vec![2]);
        test.add_hpa();
        test.enable_dynamic_update();

        test.prepare();
        test.enable_hpa();
        // test.step_until_no_events();

        // test.enable_hpa();
        test.run_for_duration(200.0);
        //
        // test.disable_hpa();
        // test.run_for_duration(50.0);
        //
        // test.enable_hpa();
        // test.run_for_duration(100.0);
    }


    // // test node affinity
    // if value == "na" {
    //     let mut test = Experiment::new(
    //         "./data/cluster_state/test_node_affinity.yaml",
    //         "./data/workload/test_node_affinity.yaml",
    //         "./data/out/test_node_affinity.txt",
    //         179
    //     );
    //     test.step_until_no_events();
    // }

    // // Test playground
    // if value == "test" {
    //     let mut test = Experiment::new(
    //         "./data/cluster_state/test_1.yaml",
    //         "./data/workload/test_1.yaml",
    //         "./data/out/test_1.txt",
    //         179
    //     );
    //     test.step_until_no_events();
    // }

    // // Test on Google cluster trace with input as yaml (DEPRECATED)
    // if value == "gyaml" {
    //     let mut test = Experiment::new(
    //         "./data/cluster_state/test_gcsv.yaml",
    //         "./data/workload/test_gyaml.yaml",
    //         "./data/out/test_gyaml.txt",
    //         179
    //     );
    //     test.step_until_no_events();
    // }

    // Test on Google cluster trace with input as csv
    if value == "gcsv" {
        let mut test = Experiment::new(
            "./data/cluster_state/test_gcsv.yaml".to_string(),
            "./data/workload/test_gcsv.csv".to_string(),
            "./data/out/test_gcsv.txt".to_string(),
            179,
        );
        test.add_scheduler(Box::new(ActiveQDefault::default()),
                           Box::new(BackOffQDefault::default()),
                           vec![Box::new(FilterNodeSelector)],
                           vec![Box::new(FilterAlwaysTrue)],
                           vec![Box::new(ScoreTetris)],
                           vec![Box::new(ScoreNormalizeSkip)],
                           vec![1]);
        test.prepare();
        test.step_until_time(60.0 * 60.0 * 24.0);
    }

    if value == "perf" {
        let mut test = Experiment::new(
            "./data/cluster_state/test_gcsv.yaml".to_string(),
            "./data/workload/test_perf_small.csv".to_string(),
            "./data/out/test_perf.txt".to_string(),
            179,
        );
        test.add_scheduler(Box::new(ActiveQDefault::default()),
                           Box::new(BackOffQDefault::default()),
                           vec![Box::new(FilterNodeSelector)],
                           vec![Box::new(FilterAlwaysTrue)],
                           vec![Box::new(ScoreTetris)],
                           vec![Box::new(ScoreNormalizeSkip)],
                           vec![2]);
        test.prepare();
        test.step_until_time(60.0 * 60.0 * 60.0);
    }
}

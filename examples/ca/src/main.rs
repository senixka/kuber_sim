use std::time::Duration;
use kuber_sim::my_imports::*;


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

    // let mut value = String::new();
    // stdin().read_line(&mut value).unwrap();
    // value = value.trim().to_string();
    let value = "".to_string();

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
    if value == "" {
        let mut init_config = InitConfig::from_yaml(&"./data/cluster_state/test_ca.yaml".to_string());
        let mut init_nodes = InitNodes::from_yaml(&"./data/cluster_state/test_ca.yaml".to_string());
        let mut init_trace = InitTrace::from_file(&"./data/workload/test_ca.yaml".to_string());

        // Prepare input
        init_config.prepare();
        init_nodes.prepare();
        init_trace.prepare();

        // Prepare pipeline config
        let mut pconf = PipelineConfig::new(
            Box::new(ActiveQDefault::default()),
            Box::new(BackOffQDefault::default()),
            vec![Box::new(FilterNodeSelector)],
            vec![Box::new(FilterAlwaysTrue)],
            vec![Box::new(ScoreTetris)],
            vec![Box::new(ScoreNormalizeSkip)],
            vec![2]);


        let mut exp = Experiment::new();
        exp.add_simulation(
            "./data/out/test_ca_1.txt".to_string(),
            &init_config,
            &init_nodes,
            &init_trace,
            &pconf,
            179,
            |sim: &mut Simulation| {
                sim.prepare();
                sim.step_for_duration(42.0);
            }
        );

        // --------------------------- Change ---------------------------
        init_config.network_delays.scheduler2api = 1.0;
        pconf.post_filters = vec![Box::new(FilterAlwaysFalse)];
        // --------------------------- Change ---------------------------

        exp.add_simulation(
            "./data/out/test_ca_2.txt".to_string(),
            &init_config,
            &init_nodes,
            &init_trace,
            &pconf,
            179,
            |sim: &mut Simulation| {
                sim.prepare();
                sim.step_for_duration(23.0);
            }
        );

        exp.spawn_all();
        thread::sleep(Duration::new(2, 0));
        exp.join_all();
    }

    // // Test pod cluster autoscaler
    // if value == "ca" {
    //     let mut test = Experiment::new(
    //         "./data/cluster_state/test_ca.yaml".to_string(),
    //         "./data/workload/test_ca.yaml".to_string(),
    //         "./data/out/test_ca.txt".to_string(),
    //         179,
    //     );
    //     test.add_scheduler(Box::new(ActiveQDefault::default()),
    //                        Box::new(BackOffQDefault::default()),
    //                        vec![Box::new(FilterNodeSelector)],
    //                        vec![Box::new(FilterAlwaysTrue)],
    //                        vec![Box::new(ScoreTetris)],
    //                        vec![Box::new(ScoreNormalizeSkip)],
    //                        vec![2]);
    //     test.add_ca();
    //
    //     test.prepare();
    //     test.run_for_duration(100.0);
    //
    //     test.enable_ca();
    //     test.run_for_duration(20.0);
    //
    //     test.disable_ca();
    //     test.run_for_duration(100.0);
    //
    //     test.enable_ca();
    //     test.run_for_duration(100.0);
    //
    // }
    //
    // // Test horizontal pod autoscaler
    // if value == "vpa" {
    //     let mut test = Experiment::new(
    //         "./data/cluster_state/test_vpa.yaml".to_string(),
    //         "./data/workload/test_vpa.yaml".to_string(),
    //         "./data/out/test_vpa.txt".to_string(),
    //         179,
    //     );
    //
    //     test.add_scheduler(Box::new(ActiveQDefault::default()),
    //                        Box::new(BackOffQDefault::default()),
    //                        vec![Box::new(FilterNodeSelector)],
    //                        vec![Box::new(FilterAlwaysTrue)],
    //                        vec![Box::new(ScoreTetris)],
    //                        vec![Box::new(ScoreNormalizeSkip)],
    //                        vec![2]);
    //     test.add_vpa();
    //     test.enable_dynamic_update();
    //
    //     test.prepare();
    //     test.enable_vpa();
    //     // test.step_until_no_events();
    //
    //     // test.enable_vpa();
    //     test.run_for_duration(40.0);
    // }
    //
    // // Test horizontal pod autoscaler
    // if value == "hpa" {
    //     let mut test = Experiment::new(
    //         "./data/cluster_state/test_hpa.yaml".to_string(),
    //         "./data/workload/test_hpa.yaml".to_string(),
    //         "./data/out/test_hpa.txt".to_string(),
    //         179,
    //     );
    //
    //     test.add_scheduler(Box::new(ActiveQDefault::default()),
    //                        Box::new(BackOffQDefault::default()),
    //                        vec![Box::new(FilterNodeSelector)],
    //                        vec![Box::new(FilterAlwaysTrue)],
    //                        vec![Box::new(ScoreTetris)],
    //                        vec![Box::new(ScoreNormalizeSkip)],
    //                        vec![2]);
    //     test.add_hpa();
    //     test.enable_dynamic_update();
    //
    //     test.prepare();
    //     test.enable_hpa();
    //     // test.step_until_no_events();
    //
    //     // test.enable_hpa();
    //     test.run_for_duration(200.0);
    //     //
    //     // test.disable_hpa();
    //     // test.run_for_duration(50.0);
    //     //
    //     // test.enable_hpa();
    //     // test.run_for_duration(100.0);
    // }


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

    // // Test on Google cluster trace with input as csv
    // if value == "gcsv" {
    //     let mut test = Experiment::new(
    //         "./data/cluster_state/test_gcsv.yaml".to_string(),
    //         "./data/workload/test_gcsv.csv".to_string(),
    //         "./data/out/test_gcsv.txt".to_string(),
    //         179,
    //     );
    //     test.add_scheduler(Box::new(ActiveQDefault::default()),
    //                        Box::new(BackOffQDefault::default()),
    //                        vec![Box::new(FilterNodeSelector)],
    //                        vec![Box::new(FilterAlwaysTrue)],
    //                        vec![Box::new(ScoreTetris)],
    //                        vec![Box::new(ScoreNormalizeSkip)],
    //                        vec![1]);
    //     test.prepare();
    //     test.step_until_time(60.0 * 60.0 * 24.0);
    // }
    //
    // if value == "perf" {
    //     let mut test = Experiment::new(
    //         "./data/cluster_state/test_gcsv.yaml".to_string(),
    //         "./data/workload/test_perf_small.csv".to_string(),
    //         "./data/out/test_perf.txt".to_string(),
    //         179,
    //     );
    //     test.add_scheduler(Box::new(ActiveQDefault::default()),
    //                        Box::new(BackOffQDefault::default()),
    //                        vec![Box::new(FilterNodeSelector)],
    //                        vec![Box::new(FilterAlwaysTrue)],
    //                        vec![Box::new(ScoreTetris)],
    //                        vec![Box::new(ScoreNormalizeSkip)],
    //                        vec![2]);
    //     test.prepare();
    //     test.step_until_time(60.0 * 60.0 * 60.0);
    // }
}

use kuber_sim::*;

/// This example shows how to run simulation multithreaded
fn main() {
    // Read input
    let mut init_config = InitConfig::from_yaml(&"./in_multithread.yaml".to_string());
    let mut init_nodes = InitNodes::from_yaml(&"./in_multithread.yaml".to_string());
    let mut init_trace = InitTrace::from_file(&"./in_multithread.yaml".to_string());

    // Prepare input
    init_config.prepare();
    init_nodes.prepare();
    init_trace.prepare();

    // Prepare scheduler pipeline config
    let mut pipeline_config = PipelineConfig::new(
        Box::new(ActiveQDefault::default()),
        Box::new(BackOffQConstant::new(1.0)),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
    );

    // Create Experiment
    let mut experiment = Experiment::new();

    // Add first simulation
    experiment.add_simulation(
        "./out_multithread_1".to_string(),
        &init_config,
        &init_nodes,
        &init_trace,
        &pipeline_config,
        123,
        false,
        false,
        false,
        |sim: &mut Simulation| {
            sim.step_for_duration(40.0);
        },
    );

    // You can make arbitrary changes to the configs
    // Add some delay to network
    init_config.network_delays.api2scheduler = 2.0;
    // Change monitoring refresh rate
    init_config.monitoring.self_update_period = 1.5;
    // Change scheduler pipeline
    pipeline_config.scorers.push(ScoreTetris.clone());
    pipeline_config.score_normalizers.push(ScoreNormalizeSkip.clone());
    pipeline_config.scorer_weights.push(1);
    // Change cluster node resources
    init_nodes.nodes[0].node.spec.installed_cpu = 200;

    // After any changes to the init_config, init_nodes or init_trace it is important to prepare changed again
    init_config.prepare();
    init_nodes.prepare();
    // pipeline_config does not need to be prepared again.

    // Add second simulation
    experiment.add_simulation(
        "./out_multithread.txt".to_string(),
        &init_config,
        &init_nodes,
        &init_trace,
        &pipeline_config,
        123,
        false,
        false,
        false,
        |sim: &mut Simulation| {
            sim.step_for_duration(20.0);
        },
    );

    // Now start all simulations
    experiment.spawn_all();
    // And wait until all the simulations are finished
    experiment.join_all();
}

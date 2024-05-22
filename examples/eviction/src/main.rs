use kuber_sim::my_imports::*;

/// This example shows that when pods on the kubelet try to consume more memory than is available on the node, an eviction process occurs.
/// Eviction works only for memory. This mechanism is not required for CPU due to the presence of throttling.
fn main() {
    // Read input
    let mut init_config = InitConfig::from_yaml(&"./in_eviction.yaml".to_string());
    let mut init_nodes = InitNodes::from_yaml(&"./in_eviction.yaml".to_string());
    let mut init_trace = InitTrace::from_file(&"./in_eviction.yaml".to_string());

    // Prepare input
    init_config.prepare();
    init_nodes.prepare();
    init_trace.prepare();

    // Prepare scheduler pipeline config
    let pipeline_config = PipelineConfig::new(
        Box::new(ActiveQDefault::default()),
        Box::new(BackOffQConstant::new(1.0)),
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
    );

    // Create simulation
    let mut sim = Simulation::new(
        "./out_eviction".to_string(),
        &init_config,
        &init_nodes,
        &init_trace,
        &pipeline_config,
        123,
        false,
        false,
        false,
    );

    // Work with simulation
    sim.step_for_duration(50.0);
}

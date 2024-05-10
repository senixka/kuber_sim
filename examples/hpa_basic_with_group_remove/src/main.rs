use kuber_sim::my_imports::*;

/// This example shows that HPA reacts on group remove event.
fn main() {
    // Read input
    let mut init_config = InitConfig::from_yaml(&"./in_hpa_basic_with_group_remove.yaml".to_string());
    let mut init_nodes = InitNodes::from_yaml(&"./in_hpa_basic_with_group_remove.yaml".to_string());
    let mut init_trace = InitTrace::from_file(&"./in_hpa_basic_with_group_remove.yaml".to_string());

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
        "./out_hpa_basic_with_group_remove.txt".to_string(),
        &init_config,
        &init_nodes,
        &init_trace,
        &pipeline_config,
        123,
        false,
        true,
        false,
    );

    // Work with simulation
    sim.step_for_duration(50.0);
}

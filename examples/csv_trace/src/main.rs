use kuber_sim::my_imports::*;

/// This example shows how to init trace from CSV file
fn main() {
    // Read input
    let mut init_config = InitConfig::from_yaml(&"./in_other.yaml".to_string());
    let mut init_nodes = InitNodes::from_yaml(&"./in_other.yaml".to_string());
    let mut init_trace = InitTrace::from_file(&"./in_csv_trace.csv".to_string());

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
        "./out_csv_trace".to_string(),
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

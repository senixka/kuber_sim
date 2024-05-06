use crate::my_imports::*;


#[derive(Clone)]
struct SimConfig {
    pub output_file_path: String,
    pub init_config: InitConfig,
    pub init_nodes: InitNodes,
    pub init_trace: InitTrace,
    pub pipeline_config: PipelineConfig,
    pub seed: u64,
}


pub struct Experiment {
    is_done: bool,
    simulations: LinkedList<(SimConfig, fn(&mut Simulation))>,
    pids: LinkedList<thread::JoinHandle<()>>,
}

impl Experiment {
    pub fn new() -> Self {
        Self {
            is_done: false,
            simulations: LinkedList::new(),
            pids: LinkedList::new(),
        }
    }

    pub fn add_simulation(&mut self,
                          output_file_path: String,
                          init_config: &InitConfig,
                          init_nodes: &InitNodes,
                          init_trace: &InitTrace,
                          pipeline_config: &PipelineConfig,
                          seed: u64,
                          runner: fn(&mut Simulation)) {
        assert!(!self.is_done);

        self.simulations.push_back((SimConfig {
                output_file_path,
                init_config: init_config.clone(),
                init_nodes: init_nodes.clone(),
                init_trace: init_trace.clone(),
                pipeline_config: pipeline_config.clone(),
                seed
            },
            runner.clone()
        ));
    }

    pub fn spawn_all(&mut self) {
        assert!(!self.is_done);
        self.is_done = true;

        while let Some(sim_config) = self.simulations.pop_front() {
            self.pids.push_back(thread::spawn(move || {
                let mut sim = Simulation::new(
                    sim_config.0.output_file_path,
                    sim_config.0.init_config,
                    sim_config.0.init_nodes,
                    sim_config.0.init_trace,
                    sim_config.0.pipeline_config,
                    sim_config.0.seed.clone()
                );

                sim_config.1(&mut sim);
            }));
        }
    }

    pub fn join_all(&mut self) {
        while let Some(pid) = self.pids.pop_front() {
            match pid.join() {
                Result::Ok(_) => {
                    println!("Finished Ok")
                }
                Result::Err(e) => {
                    println!("Finished Err:{:?}", e);
                }
            }
        }
    }
}

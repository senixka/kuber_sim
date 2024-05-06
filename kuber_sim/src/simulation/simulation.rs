use crate::my_imports::*;


pub struct Simulation {
    sim: dsc::Simulation,

    init_config: Rc<RefCell<InitConfig>>,
    init_nodes: InitNodes,
    init_trace: InitTrace,

    monitoring: Rc<RefCell<Monitoring>>,
    api: Rc<RefCell<APIServer>>,

    api_id: dsc::Id,

    scheduler_id: dsc::Id,

    ca: Option<Rc<RefCell<CA>>>,
    ca_id: Option<dsc::Id>,

    hpa: Option<Rc<RefCell<HPA>>>,
    hpa_id: Option<dsc::Id>,

    vpa: Option<Rc<RefCell<VPA>>>,
    vpa_id: Option<dsc::Id>,

    is_preparation_done: bool,
}


impl Simulation {
    pub fn new(output_file_path: String,
               init_config: InitConfig,
               init_nodes: InitNodes,
               init_trace: InitTrace,
               pipeline_config: PipelineConfig,
               seed: u64) -> Self {
        // DSLab core
        let mut sim = dsc::Simulation::new(seed);

        // Init config to shared_ptr
        let init_config_ptr = Rc::new(RefCell::new(init_config));

        // Api-server component
        let api = Rc::new(RefCell::new(
            APIServer::new(
                sim.create_context("api_server"), init_config_ptr.clone()
            )
        ));
        let api_id = sim.add_handler("api_server", api.clone());

        // Monitoring component
        let monitoring = Rc::new(RefCell::new(
            Monitoring::new(
                sim.create_context("monitoring"), init_config_ptr.clone(), &output_file_path,
            )
        ));
        let _ = sim.add_handler("monitoring", monitoring.clone());

        assert_eq!(pipeline_config.scorers.len(), pipeline_config.score_normalizers.len());
        assert_eq!(pipeline_config.scorers.len(), pipeline_config.scorer_weights.len());

        let scheduler = Rc::new(RefCell::new(
            Scheduler::new(
                sim.create_context("scheduler"),
                init_config_ptr.clone(),
                monitoring.clone(),
                api_id,

                pipeline_config.active_queue,
                pipeline_config.backoff_queue,

                pipeline_config.filters,
                pipeline_config.post_filters,
                pipeline_config.scorers,
                pipeline_config.score_normalizers,
                pipeline_config.scorer_weights,
            )
        ));
        let scheduler_id = sim.add_handler("scheduler", scheduler.clone());

        Self {
            sim,
            init_config: init_config_ptr.clone(),
            init_nodes,
            init_trace,
            api_id,
            monitoring,
            api,
            scheduler_id,
            ca: None,
            ca_id: None,
            hpa: None,
            hpa_id: None,
            vpa: None,
            vpa_id: None,
            is_preparation_done: false,
        }
    }

    // pub fn add_ca(&mut self) {
    //     self.ca = Some(Rc::new(RefCell::new(
    //         CA::new(
    //             &mut self.sim,
    //             self.sim.create_context("ca"),
    //             self.init_config.clone(),
    //             self.init_nodes.clone(),
    //             self.monitoring.clone(),
    //             self.api_id)
    //     )));
    //     self.ca_id = Some(self.sim.add_handler("ca", self.ca.clone().unwrap()));
    // }
    //
    // pub fn add_hpa(&mut self) {
    //     self.hpa = Some(Rc::new(RefCell::new(
    //         HPA::new(
    //             self.sim.create_context("hpa"),
    //             self.cluster_state.clone(),
    //             self.api_id)
    //     )));
    //     self.hpa_id = Some(self.sim.add_handler("hpa", self.hpa.clone().unwrap()));
    // }
    //
    // pub fn add_vpa(&mut self) {
    //     self.vpa = Some(Rc::new(RefCell::new(
    //         VPA::new(
    //             self.sim.create_context("vpa"),
    //             self.cluster_state.clone(),
    //             self.api_id)
    //     )));
    //     self.vpa_id = Some(self.sim.add_handler("vpa", self.vpa.clone().unwrap()));
    // }

    pub fn prepare(&mut self) {
        assert_eq!(self.is_preparation_done, false);

        self.api.borrow_mut().prepare(self.scheduler_id, self.ca_id, self.hpa_id, self.vpa_id);
        self.monitoring.borrow_mut().presimulation_init();

        self.init_nodes.submit(&mut self.sim, &self.api.borrow().ctx, self.init_config.clone(), self.monitoring.clone(), self.api_id);
        self.init_trace.submit(&self.api.borrow().ctx, self.api_id);

        // TODO: clear local nodes and trace

        self.is_preparation_done = true;
    }

    pub fn enable_dynamic_update(&self) {
        self.monitoring.borrow_mut().enable_dynamic_update();
    }

    pub fn disable_dynamic_update(&self) {
        self.monitoring.borrow_mut().disable_dynamic_update();
    }


    pub fn enable_ca(&self) {
        self.ca.clone().unwrap().borrow_mut().turn_on();
    }

    pub fn disable_ca(&self) {
        self.ca.clone().unwrap().borrow_mut().turn_off();
    }

    pub fn enable_hpa(&self) {
        self.hpa.clone().unwrap().borrow_mut().turn_on();
    }

    pub fn disable_hpa(&self) {
        self.hpa.clone().unwrap().borrow_mut().turn_off();
    }

    pub fn enable_vpa(&self) {
        self.vpa.clone().unwrap().borrow_mut().turn_on();
    }

    pub fn disable_vpa(&self) {
        self.vpa.clone().unwrap().borrow_mut().turn_off();
    }

    pub fn step_until_no_events(&mut self) {
        assert_eq!(self.is_preparation_done, true);
        self.sim.step_until_no_events();
    }

    pub fn step_for_duration(&mut self, duration: f64) {
        assert_eq!(self.is_preparation_done, true);
        self.sim.step_for_duration(duration);
    }

    pub fn steps(&mut self, steps: u64) {
        assert_eq!(self.is_preparation_done, true);
        self.sim.steps(steps);
    }

    pub fn step_until_time(&mut self, time: f64) {
        assert_eq!(self.is_preparation_done, true);
        self.sim.step_until_time(time);
    }
}

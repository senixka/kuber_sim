use crate::my_imports::*;

pub struct Simulation {
    sim: dsc::Simulation,
    // init_config: Rc<RefCell<InitConfig>>,
    monitoring: Rc<RefCell<Monitoring>>,
    // api: Rc<RefCell<APIServer>>,
    ca: Option<Rc<RefCell<CA>>>,
    hpa: Option<Rc<RefCell<HPA>>>,
    vpa: Option<Rc<RefCell<VPA>>>,
}

impl Simulation {
    pub fn new(
        output_file_path: String,
        init_config: &InitConfig,
        init_nodes: &InitNodes,
        init_trace: &InitTrace,
        pipeline_config: &PipelineConfig,
        seed: u64,
        flag_add_ca: bool,
        flag_add_hpa: bool,
        flag_add_vpa: bool,
    ) -> Self {
        // DSLab core
        let mut sim = dsc::Simulation::new(seed);

        // Init config to shared_ptr
        let init_config_ptr = Rc::new(RefCell::new(init_config.clone()));
        // Init nodes to shared_ptr
        let init_nodes_ptr = Rc::new(RefCell::new(init_nodes.clone()));

        // Api-server component
        let api = Rc::new(RefCell::new(APIServer::new(
            sim.create_context("api_server"),
            init_config_ptr.clone(),
        )));
        let api_id = sim.add_handler("api_server", api.clone());

        // Monitoring component
        let monitoring = Rc::new(RefCell::new(Monitoring::new(
            sim.create_context("monitoring"),
            init_config_ptr.clone(),
            &output_file_path,
        )));
        let _ = sim.add_handler("monitoring", monitoring.clone());

        // Copy scheduler pipeline config
        let pconf = pipeline_config.clone();

        assert_eq!(pconf.scorers.len(), pconf.score_normalizers.len());
        assert_eq!(pconf.scorers.len(), pconf.scorer_weights.len());

        let scheduler = Rc::new(RefCell::new(Scheduler::new(
            sim.create_context("scheduler"),
            init_config_ptr.clone(),
            monitoring.clone(),
            api_id,
            pconf.active_queue,
            pconf.backoff_queue,
            pconf.filters,
            pconf.post_filters,
            pconf.scorers,
            pconf.score_normalizers,
            pconf.scorer_weights,
        )));
        let scheduler_id = sim.add_handler("scheduler", scheduler.clone());

        // Add CA if needed
        let mut ca = None;
        let mut ca_id = None;
        if flag_add_ca {
            let ca_ctx = sim.create_context("ca");
            ca = Some(Rc::new(RefCell::new(CA::new(
                &mut sim,
                ca_ctx,
                init_config_ptr.clone(),
                init_nodes_ptr.clone(),
                monitoring.clone(),
                api_id,
            ))));
            ca_id = Some(sim.add_handler("ca", ca.clone().unwrap()));

            // Turn on CA
            ca.clone().unwrap().borrow_mut().turn_on();
        }

        // Add HPA if needed
        let mut hpa = None;
        let mut hpa_id = None;
        if flag_add_hpa {
            hpa = Some(Rc::new(RefCell::new(HPA::new(
                sim.create_context("hpa"),
                init_config_ptr.clone(),
                api_id,
            ))));
            hpa_id = Some(sim.add_handler("hpa", hpa.clone().unwrap()));

            // Turn on HPA
            hpa.clone().unwrap().borrow_mut().turn_on();
        }

        // Add VPA if needed
        let mut vpa = None;
        let mut vpa_id = None;
        if flag_add_vpa {
            vpa = Some(Rc::new(RefCell::new(VPA::new(
                sim.create_context("vpa"),
                init_config_ptr.clone(),
                api_id,
            ))));
            vpa_id = Some(sim.add_handler("vpa", vpa.clone().unwrap()));

            // Turn on VPA
            vpa.clone().unwrap().borrow_mut().turn_on();
        }

        // Prepare components
        api.borrow_mut().prepare(scheduler_id, ca_id, hpa_id, vpa_id);
        monitoring.borrow_mut().prepare();

        // Prepare cluster with nodes
        init_nodes.submit(
            &mut sim,
            &api.borrow().ctx,
            init_config_ptr.clone(),
            monitoring.clone(),
            api_id,
        );
        // Prepare cluster with trace
        init_trace.submit(&api.borrow().ctx, api_id);

        Self {
            sim,
            // init_config: init_config_ptr.clone(),
            monitoring,
            // api,
            ca,
            hpa,
            vpa,
        }
    }

    pub fn dump_stats(&self) {
        self.monitoring.borrow().dump_statistics();
    }

    pub fn enable_dynamic_update(&self) {
        self.monitoring.borrow_mut().enable_dynamic_update();
    }

    pub fn disable_dynamic_update(&self) {
        self.monitoring.borrow_mut().disable_dynamic_update();
    }

    pub fn enable_print(&mut self) {
        self.monitoring.borrow_mut().print_enabled = true;
    }

    pub fn disable_print(&mut self) {
        self.monitoring.borrow_mut().print_enabled = false;
    }

    pub fn clear_records(&mut self) {
        self.monitoring.borrow_mut().clear_records();
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
        self.sim.step_until_no_events();
    }

    pub fn step_for_duration(&mut self, duration: f64) {
        self.sim.step_for_duration(duration);
    }

    pub fn steps(&mut self, steps: u64) {
        self.sim.steps(steps);
    }

    pub fn step_until_time(&mut self, time: f64) {
        self.sim.step_until_time(time);
    }
}

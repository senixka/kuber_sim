use crate::my_imports::*;


pub struct Experiment {
    cluster_state_file_path: String,
    workload_file_path: String,
    output_file_path: String,

    sim: dsc::Simulation,
    seed: u64,

    api: Rc<RefCell<APIServer>>,
    cluster_state: Rc<RefCell<ClusterState>>,
    workload: Rc<RefCell<WorkLoad>>,
    init: Rc<RefCell<Init>>,
    monitoring: Rc<RefCell<Monitoring>>,

    api_id: dsc::Id,
    monitoring_id: dsc::Id,

    scheduler: Option<Rc<RefCell<Scheduler>>>,
    scheduler_id: Option<dsc::Id>,

    ca: Option<Rc<RefCell<CA>>>,
    ca_id: Option<dsc::Id>,

    hpa: Option<Rc<RefCell<HPA>>>,
    hpa_id: Option<dsc::Id>,

    is_preparation_done: bool,
}


impl Experiment {
    pub fn new(cluster_state_file_path: String,
               workload_file_path: String,
               output_file_path: String,
               seed: u64) -> Self {
        // DSLab core
        let mut sim = dsc::Simulation::new(seed);

        // State objects
        let cluster_state = Rc::new(RefCell::new(ClusterState::from_yaml(&cluster_state_file_path)));
        let workload = Rc::new(RefCell::new(WorkLoad::from_file(&workload_file_path)));

        // Api-server component
        let api = Rc::new(RefCell::new(
            APIServer::new(
                sim.create_context("api_server"), cluster_state.clone()
            )
        ));
        let api_id = sim.add_handler("api_server", api.clone());

        // Monitoring component
        let monitoring = Rc::new(RefCell::new(
            Monitoring::new(
                sim.create_context("monitoring"), cluster_state.clone(), &output_file_path,
            )
        ));
        let monitoring_id = sim.add_handler("monitoring", monitoring.clone());

        // Init component
        let init = Rc::new(RefCell::new(
            Init::new(
                sim.create_context("init"), cluster_state.clone(), workload.clone(), monitoring.clone(), api_id,
            )
        ));

        Self {
            cluster_state_file_path, workload_file_path, output_file_path,
            sim, seed,
            api, cluster_state, workload, init, monitoring,
            api_id, monitoring_id,
            scheduler: None,
            scheduler_id: None,
            ca: None,
            ca_id: None,
            hpa: None,
            hpa_id: None,
            is_preparation_done: false,
        }
    }

    pub fn add_scheduler(&mut self,
                         active_queue: Box<dyn IActiveQ>,
                         backoff_queue: Box<dyn IBackOffQ>,

                         filters: Vec<Box<dyn IFilterPlugin>>,
                         post_filters: Vec<Box<dyn IFilterPlugin>>,
                         scorers: Vec<Box<dyn IScorePlugin>>,
                         score_normalizers: Vec<Box<dyn IScoreNormalizePlugin>>,
                         scorer_weights: Vec<i64>) {
        assert_eq!(scorers.len(), score_normalizers.len());
        assert_eq!(scorers.len(), scorer_weights.len());

        self.scheduler = Some(Rc::new(RefCell::new(
            Scheduler::new(
                self.sim.create_context("scheduler"),
                self.api_id,
                self.cluster_state.clone(),
                self.monitoring.clone(),

                active_queue,
                backoff_queue,

                filters,
                post_filters,
                scorers,
                score_normalizers,
                scorer_weights,
            )
        )));
        self.scheduler_id = Some(self.sim.add_handler("scheduler", self.scheduler.clone().unwrap()));
    }

    pub fn add_ca(&mut self) {
        self.ca = Some(Rc::new(RefCell::new(
            CA::new(
                self.sim.create_context("ca"),
                self.cluster_state.clone(),
                self.monitoring.clone(),
                self.api_id,
                &mut self.sim)
        )));
        self.ca_id = Some(self.sim.add_handler("ca", self.ca.clone().unwrap()));
    }

    pub fn add_hpa(&mut self) {
        self.hpa = Some(Rc::new(RefCell::new(
            HPA::new(
                self.sim.create_context("hpa"),
                self.cluster_state.clone(),
                self.workload.clone(),
                self.api_id)
        )));
        self.hpa_id = Some(self.sim.add_handler("hpa", self.hpa.clone().unwrap()));
    }

    pub fn prepare(&mut self) {
        assert_eq!(self.is_preparation_done, false);

        let scheduler_id = self.scheduler_id.unwrap();
        let ca_id = self.ca_id.unwrap_or(dsc::Id::MAX);
        let hpa_id = self.ca_id.unwrap_or(dsc::Id::MAX);

        self.api.borrow_mut().prepare(scheduler_id, ca_id, hpa_id);
        self.monitoring.borrow_mut().presimulation_init(ca_id, hpa_id);

        self.init.borrow().submit_nodes(&mut self.sim);
        self.init.borrow().submit_pods();

        self.is_preparation_done = true;
    }

    pub fn enable_cluster_autoscaler(&self) {
        self.ca.clone().unwrap().borrow_mut().turn_on();
    }

    pub fn disable_cluster_autoscaler(&self) {
        self.ca.clone().unwrap().borrow_mut().turn_off();
    }

    pub fn enable_hpa(&self) {
        self.hpa.clone().unwrap().borrow_mut().turn_on();
    }

    pub fn disable_hpa(&self) {
        self.hpa.clone().unwrap().borrow_mut().turn_off();
    }

    pub fn step_until_no_events(&mut self) {
        assert_eq!(self.is_preparation_done, true);
        self.sim.step_until_no_events();
    }

    pub fn run_for_duration(&mut self, duration: f64) {
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

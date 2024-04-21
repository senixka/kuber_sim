use crate::my_imports::*;


pub struct CA {
    ctx: dsc::SimulationContext,
    cluster_state: Rc<RefCell<ClusterState>>,
    api_sim_id: dsc::Id,
    monitoring: Rc<RefCell<Monitoring>>,

    is_turned_on: bool,

    // kubelet_pool: Vec<(dsc::Id, Rc<RefCell<Kubelet>>)>,
    // free_nodes: Vec<NodeGroup>,
    // in_use: Vec<NodeGroup>,
}


impl CA {
    pub fn new(ctx: dsc::SimulationContext,
               cluster_state: Rc<RefCell<ClusterState>>,
               monitoring: Rc<RefCell<Monitoring>>,
               api_sim_id: dsc::Id) -> Self {
        Self {
            ctx,
            cluster_state,
            api_sim_id,
            monitoring,
            is_turned_on: false,
        }
    }

    pub fn turn_on(&mut self) {
        if !self.is_turned_on {
            self.is_turned_on = true;
            self.ctx.emit_self(APICASelfUpdate {}, self.cluster_state.borrow().constants.ca_self_update_period);
        }
    }

    pub fn turn_off(&mut self) {
        if self.is_turned_on {
            self.is_turned_on = false;
            self.ctx.cancel_heap_events(|x| x.src == self.ctx.id() && x.dst == self.ctx.id());
        }
    }

    pub fn process(&mut self, insufficient_resources_pending: u64, max_insufficient_resources_request: (u64, u64)) {

    }
}

impl dsc::EventHandler for CA {
    fn on(&mut self, event: dsc::Event) {
        dsc::cast!(match event.data {
            APICATurnOn {} => {
                dp_ca!("{:.12} ca APICATurnOn", self.ctx.time());

                self.turn_on();
            }
            APICATurnOff {} => {
                dp_ca!("{:.12} ca APICATurnOff", self.ctx.time());

                self.turn_off();
            }
            APICASelfUpdate {} => {
                dp_ca!("{:.12} ca APICASelfUpdate", self.ctx.time());

                if !self.is_turned_on {
                    panic!("Bad logic. Self update should be canceled.");
                }

                self.ctx.emit(APIGetCAMetrics {}, self.api_sim_id, self.cluster_state.borrow().network_delays.ca2api);
                self.ctx.emit_self(APICASelfUpdate {}, self.cluster_state.borrow().constants.ca_self_update_period);
            }
            APIPostCAMetrics { insufficient_resources_pending, max_insufficient_resources_request } => {
                dp_ca!("{:.12} ca APICASelfUpdate insufficient_resources_pending:{:?} max_insufficient_resources_request:{:?}", self.ctx.time(), insufficient_resources_pending, max_insufficient_resources_request);

                self.process(insufficient_resources_pending, max_insufficient_resources_request);
            }
        });
    }
}

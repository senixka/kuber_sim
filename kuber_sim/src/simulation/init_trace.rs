use crate::my_imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceEvent {
    AddPodGroup(PodGroup),
    RemovePodGroup(EventRemovePodGroup),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEventWrapper {
    submit_time: f64,
    event: TraceEvent,
}

impl PartialOrd for TraceEventWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TraceEventWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.submit_time.total_cmp(&other.submit_time)
    }
}

impl PartialEq for TraceEventWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.submit_time == other.submit_time
    }
}

impl Eq for TraceEventWrapper {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InitTrace {
    #[serde(default)]
    pub trace: Vec<TraceEventWrapper>,
}

impl InitTrace {
    pub fn from_file(path: &String) -> Self {
        if path.ends_with(".yaml") {
            return InitTrace::from_yaml(path);
        // } else if path.ends_with(".csv") {
        //     return InitTrace::from_csv(path);
        } else {
            panic!("File format and extension must be '.yaml' or '.csv'")
        }
    }

    pub fn from_yaml(path: &String) -> Self {
        let fin = std::fs::File::open(path).unwrap();
        let mut trace: InitTrace = serde_yaml::from_reader(fin).unwrap();

        // Prepare trace
        trace.prepare();
        return trace;
    }
    //
    // // TODO: update format, prepare for hpa, add taints and other...
    // pub fn from_csv(path: &String) -> Self {
    //     let file = std::fs::File::open(path).unwrap();
    //     let reader = BufReader::new(file);
    //
    //     let mut workload = InitTrace::default();
    //
    //     for line in reader.lines() {
    //         let s = line.unwrap().trim().to_string();
    //         let data: Vec<&str> = s.split(",").collect();
    //
    //         let mut pod = Pod::default();
    //
    //         pod.spec.request_cpu = data[2].parse().unwrap();
    //         pod.spec.request_memory = data[3].parse().unwrap();
    //         pod.spec.limit_cpu = data[4].parse().unwrap_or(i64::MAX);
    //         pod.spec.limit_memory = data[5].parse().unwrap_or(i64::MAX);
    //         pod.spec.priority = data[6].parse().unwrap_or(0);
    //
    //         let pod_labels: Vec<&str> = data[7].split(";").collect();
    //         for label in pod_labels {
    //             if label.is_empty() {
    //                 continue;
    //             }
    //
    //             let key_value: Vec<&str> = label.split(":").collect();
    //             assert_eq!(key_value.len(), 2);
    //
    //             pod.metadata.labels.insert(key_value[0].to_string(), key_value[1].to_string());
    //         }
    //
    //         let node_selector_data: Vec<&str> = data[8].split(";").collect();
    //         for selector in node_selector_data {
    //             if selector.is_empty() {
    //                 continue;
    //             }
    //
    //             let key_value: Vec<&str> = selector.split(":").collect();
    //             assert_eq!(key_value.len(), 2);
    //
    //             pod.spec.node_selector.insert(key_value[0].to_string(), key_value[1].to_string());
    //         }
    //
    //         let load_data: Vec<&str> = data[9].split(";").collect();
    //         match load_data[0] {
    //             "constant" => {
    //                 assert_eq!(load_data.len(), 4);
    //
    //                 let mut constant = Constant::default();
    //                 constant.cpu = load_data[1].parse().unwrap();
    //                 constant.memory = load_data[2].parse().unwrap();
    //                 constant.duration = load_data[3].parse().unwrap();
    //
    //                 pod.spec.load = LoadType::Constant(constant);
    //             }
    //             "busybox" => {
    //                 assert_eq!(load_data.len(), 7);
    //
    //                 let mut busybox = BusyBox::default();
    //                 busybox.cpu_down = load_data[1].parse().unwrap();
    //                 busybox.memory_down = load_data[2].parse().unwrap();
    //                 busybox.cpu_up = load_data[3].parse().unwrap();
    //                 busybox.memory_up = load_data[4].parse().unwrap();
    //                 busybox.duration = load_data[5].parse().unwrap();
    //                 busybox.shift_time = load_data[6].parse().unwrap();
    //
    //                 pod.spec.load = LoadType::BusyBox(busybox);
    //             }
    //             _ => {
    //                 panic!("Unknown load type");
    //             }
    //         }
    //
    //         let mut pod_group = PodGroup::default();
    //         pod_group.pod_count = data[0].parse().unwrap();
    //         pod_group.submit_time = data[1].parse().unwrap();
    //         pod_group.pod = pod;
    //
    //         pod_group.prepare();
    //         workload.pods.push(pod_group);
    //     }
    //
    //     return workload;
    // }

    pub fn prepare(&mut self) {
        // Prepare trace events
        for wrapper in self.trace.iter_mut() {
            match &mut wrapper.event {
                TraceEvent::AddPodGroup(pod_group) => {
                    pod_group.prepare();

                    // Check HPA invariants
                    if pod_group.hpa_profile.is_some() {
                        let profile = pod_group.hpa_profile.clone().unwrap();
                        assert!(
                            profile.min_size <= profile.max_size,
                            "ConfigHPA.min_size must be <= ConfigHPA.max_size"
                        );
                        assert!(
                            profile.scale_down_mean_cpu_fraction >= 0.0,
                            "ConfigHPA.scale_down_mean_cpu_fraction must be >= 0"
                        );
                        assert!(
                            profile.scale_down_mean_memory_fraction >= 0.0,
                            "ConfigHPA.scale_down_mean_memory_fraction must be >= 0"
                        );
                        assert!(
                            profile.scale_down_mean_cpu_fraction <= profile.scale_up_mean_cpu_fraction,
                            "ConfigHPA.scale_down_mean_cpu_fraction must be <= ConfigHPA.scale_up_mean_cpu_fraction"
                        );
                        assert!(profile.scale_down_mean_memory_fraction <= profile.scale_up_mean_memory_fraction, "ConfigHPA.scale_down_mean_memory_fraction must be <= ConfigHPA.scale_up_mean_memory_fraction");
                    }

                    // Check VPA invariants
                    if pod_group.vpa_profile.is_some() {
                        let profile = pod_group.vpa_profile.clone().unwrap();
                        assert!(
                            profile.min_allowed_cpu <= profile.max_allowed_cpu,
                            "ConfigVPA.min_allowed_cpu must be <= ConfigVPA.max_allowed_cpu"
                        );
                        assert!(
                            profile.min_allowed_memory <= profile.max_allowed_memory,
                            "ConfigVPA.min_allowed_memory must be <= ConfigVPA.max_allowed_memory"
                        );
                        assert!(profile.min_allowed_cpu > 0, "ConfigVPA.min_allowed_cpu must be > 0");
                        assert!(
                            profile.min_allowed_memory > 0,
                            "ConfigVPA.min_allowed_memory must be > 0"
                        );
                    }
                }
                TraceEvent::RemovePodGroup(_) => {
                    // Do nothing
                }
            }
        }
    }

    pub fn submit(&self, emitter: &dsc::SimulationContext, api_sim_id: dsc::Id) {
        let mut last_time: f64 = 0.0;
        let mut delayed_events: BTreeSet<TraceEventWrapper> = BTreeSet::new();

        // Process trace events
        for wrapper in self.trace.iter() {
            // Try to submit delayed events
            while !delayed_events.is_empty() && delayed_events.first().unwrap().submit_time <= wrapper.submit_time {
                let delayed = delayed_events.pop_first().unwrap();
                match delayed.event {
                    TraceEvent::RemovePodGroup(inner_event) => {
                        // Emit inner event
                        assert!(last_time <= delayed.submit_time);
                        emitter.emit_ordered(inner_event, api_sim_id, delayed.submit_time);
                        last_time = delayed.submit_time;
                    }
                    TraceEvent::AddPodGroup(_) => {
                        panic!("Unexpected TraceEvent.")
                    }
                }
            }

            match &wrapper.event {
                TraceEvent::AddPodGroup(pod_group) => {
                    // Emit AddPodGroup event
                    emitter.emit_ordered(
                        EvenAddPodGroup {
                            pod_group: pod_group.clone(),
                        },
                        api_sim_id,
                        wrapper.submit_time,
                    );

                    // Add RemovePodGroup event to delayed if duration != 0
                    if pod_group.group_duration != 0.0 {
                        delayed_events.insert(TraceEventWrapper {
                            submit_time: wrapper.submit_time + pod_group.group_duration,
                            event: TraceEvent::RemovePodGroup(EventRemovePodGroup {
                                group_uid: pod_group.group_uid,
                            }),
                        });
                    }
                }
                TraceEvent::RemovePodGroup(_) => {
                    panic!("Unexpected TraceEvent.");
                }
            }
        }

        // Try to submit delayed events
        while !delayed_events.is_empty() {
            let delayed = delayed_events.pop_first().unwrap();
            match delayed.event {
                TraceEvent::RemovePodGroup(inner_event) => {
                    // Emit inner event
                    assert!(last_time <= delayed.submit_time);
                    emitter.emit_ordered(inner_event, api_sim_id, delayed.submit_time);
                    last_time = delayed.submit_time;
                }
                TraceEvent::AddPodGroup(_) => {
                    panic!("Unexpected TraceEvent.")
                }
            }
        }
    }
}

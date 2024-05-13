use crate::my_imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceEvent {
    AddPodGroup(PodGroup),
    RemovePodGroup(EventRemovePodGroup),
}

impl FromStr for TraceEvent {
    type Err = ();

    /// Expects "<enum_index: u8>;<enum_payload: { PodGroup }>"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (enum_index, enum_inner) = s.split_once(';').unwrap();
        let enum_inner = enum_inner.trim();

        match enum_index {
            "0" => Ok(Self::AddPodGroup(str::parse(enum_inner).unwrap())),
            _ => panic!("Unexpected enum_index: '{:?}'", enum_index),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEventWrapper {
    submit_time: f64,
    event: TraceEvent,
}

impl FromStr for TraceEventWrapper {
    type Err = ();

    /// Expects "<submit_time: f64>;<TraceEvent>"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (submit_time, event) = s.split_once(';').unwrap();
        let event = event.trim();

        Ok(Self {
            submit_time: str::parse(submit_time).unwrap(),
            event: str::parse(event).unwrap(),
        })
    }
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
        } else if path.ends_with(".csv") {
            return InitTrace::from_csv(path);
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

    pub fn from_csv(path: &String) -> Self {
        // Open file
        let file = std::fs::File::open(path).unwrap();
        // Create file reader
        let reader = BufReader::new(file);
        // Create empty trace
        let mut init_trace = InitTrace::default();

        // Read trace
        for line in reader.lines() {
            let s = line.unwrap().trim().to_string();
            if s.is_empty() {
                continue;
            }

            init_trace.trace.push(str::parse::<TraceEventWrapper>(&s).unwrap());
        }

        return init_trace;
    }

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
        let mut delayed_events: BTreeSet<TraceEventWrapper> = BTreeSet::new();
        let submit_delayed_up_to_time = |delayed: &mut BTreeSet<TraceEventWrapper>, current_time: f64| {
            while !delayed.is_empty() && delayed.first().unwrap().submit_time <= current_time {
                let delayed = delayed.pop_first().unwrap();
                match delayed.event {
                    TraceEvent::RemovePodGroup(inner_event) => {
                        emitter.emit_ordered(inner_event, api_sim_id, delayed.submit_time);
                    }
                    TraceEvent::AddPodGroup(_) => {
                        panic!("Unexpected TraceEvent.")
                    }
                }
            }
        };

        // Process trace events
        for wrapper in self.trace.iter() {
            // Try to submit delayed events
            submit_delayed_up_to_time(&mut delayed_events, wrapper.submit_time);

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
        // Submit all delayed events
        submit_delayed_up_to_time(&mut delayed_events, f64::MAX);
    }

    pub fn find_matching_bracket(s: &str, start_index: usize) -> Option<usize> {
        let mut count = 0;
        for (i, c) in s[start_index..].char_indices() {
            match c {
                '{' => count += 1,
                '}' => {
                    count -= 1;
                    if count == 0 {
                        return Some(start_index + i);
                    }
                }
                _ => {}
            }
        }
        None
    }
}

///////////////////////////////////////////// Test /////////////////////////////////////////////////

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_bracket() {
        assert_eq!(InitTrace::find_matching_bracket("{}", 0), Some(1));
        assert_eq!(InitTrace::find_matching_bracket("{1}", 0), Some(2));
        assert_eq!(InitTrace::find_matching_bracket("{1,{2},3}", 0), Some(8));
        assert_eq!(InitTrace::find_matching_bracket("{1,{2},3}", 3), Some(5));
        assert_eq!(InitTrace::find_matching_bracket("{{{}{}}{}}", 0), Some(9));
        assert_eq!(InitTrace::find_matching_bracket("{{{}{}}{}}", 1), Some(6));
        assert_eq!(InitTrace::find_matching_bracket("{{{}{}}{}}", 2), Some(3));
    }

    #[test]
    fn test_csv() {
        println!("{:?}", str::parse::<TraceEventWrapper>("1;0;5;30;{{};{10;10;20;20;1;{1;5;15};{};{};{}}};{};{}\n"));
        println!("{:?}", str::parse::<TraceEventWrapper>("1;0;5;30;{{};{10;10;20;20;1;{1;5;15};{};{};{}}};{1;2;3;4;5;6};{1;2;3;4}\n"));
        println!("{:?}", str::parse::<TraceEventWrapper>("1;0;5;;{{gpu:amd,env:test};{1;2;3;4;5;{0;5;15;30};{};{};{}}};{};{}\n"));
        println!("{:?}", str::parse::<TraceEventWrapper>("1;0;5;;{{};{10;10;20;20;1;{2;15;16;20;21;5;45};{gpu:amd,env:test};{gpu,amd,0,1;test,,1,0};{}}};{};{}\n"));
    }
}

use crate::my_imports::*;


#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WorkLoad {
    #[serde(default)]
    pub hpa_pods: Vec<HPAPodGroup>,

    #[serde(default)]
    pub pods: Vec<PodGroup>,
}


impl WorkLoad {
    pub fn from_file(path: &str) -> Self {
        if path.ends_with(".yaml") {
            return WorkLoad::from_yaml(path);
        } else if path.ends_with(".csv") {
            return WorkLoad::from_csv(path);
        } else {
            panic!("Unknown workload file format")
        }
    }

    pub fn from_yaml(path: &str) -> Self {
        let fin = std::fs::File::open(path).unwrap();
        let mut workload: WorkLoad = serde_yaml::from_reader(fin).unwrap();

        for pod_group in &mut workload.pods {
            pod_group.prepare();
        }

        for hpa_pod_group in &mut workload.hpa_pods {
            hpa_pod_group.pod_group.prepare();
        }

        return workload;
    }

    // TODO: update format, prepare for hpa, add taints and other...
    pub fn from_csv(path: &str) -> Self {
        let file = std::fs::File::open(path).unwrap();
        let reader = BufReader::new(file);

        let mut workload = WorkLoad::default();

        for line in reader.lines() {
            let s = line.unwrap().trim().to_string();
            let data: Vec<&str> = s.split(",").collect();

            let mut pod = Pod::default();

            pod.spec.arrival_time = data[1].parse().unwrap();
            pod.spec.request_cpu = data[2].parse().unwrap();
            pod.spec.request_memory = data[3].parse().unwrap();
            pod.spec.limit_cpu = data[4].parse().unwrap_or(u64::MAX);
            pod.spec.limit_memory = data[5].parse().unwrap_or(u64::MAX);
            pod.spec.priority = data[6].parse().unwrap_or(0);

            let pod_labels: Vec<&str> = data[7].split(";").collect();
            for label in pod_labels {
                if label.is_empty() {
                    continue;
                }

                let key_value: Vec<&str> = label.split(":").collect();
                assert_eq!(key_value.len(), 2);

                pod.metadata.labels.insert(key_value[0].to_string(), key_value[1].to_string());
            }

            let node_selector_data: Vec<&str> = data[8].split(";").collect();
            for selector in node_selector_data {
                if selector.is_empty() {
                    continue;
                }

                let key_value: Vec<&str> = selector.split(":").collect();
                assert_eq!(key_value.len(), 2);

                pod.spec.node_selector.insert(key_value[0].to_string(), key_value[1].to_string());
            }

            let load_data: Vec<&str> = data[9].split(";").collect();
            match load_data[0] {
                "constant" => {
                    assert_eq!(load_data.len(), 4);

                    let mut constant = Constant::default();
                    constant.cpu = load_data[1].parse().unwrap();
                    constant.memory = load_data[2].parse().unwrap();
                    constant.duration = load_data[3].parse().unwrap();

                    pod.spec.load = LoadType::Constant(constant);
                }
                "busybox" => {
                    assert_eq!(load_data.len(), 7);

                    let mut busybox = BusyBox::default();
                    busybox.cpu_down = load_data[1].parse().unwrap();
                    busybox.memory_down = load_data[2].parse().unwrap();
                    busybox.cpu_up = load_data[3].parse().unwrap();
                    busybox.memory_up = load_data[4].parse().unwrap();
                    busybox.duration = load_data[5].parse().unwrap();
                    busybox.shift_time = load_data[6].parse().unwrap();

                    pod.spec.load = LoadType::BusyBox(busybox);
                }
                _ => {
                    panic!("Unknown load type");
                }
            }

            let mut pod_group = PodGroup::default();
            pod_group.amount = data[0].parse().unwrap();
            pod_group.pod = pod;

            pod_group.prepare();
            workload.pods.push(pod_group);
        }

        return workload;
    }
}

use crate::my_imports::*;


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HPAPodGroup {
    pub pod_group: PodGroup,

    pub min_size: u64,
    pub max_size: u64,

    pub min_group_cpu_percent: f64,
    pub min_group_memory_percent: f64,

    pub max_group_cpu_percent: f64,
    pub max_group_memory_percent: f64,
}

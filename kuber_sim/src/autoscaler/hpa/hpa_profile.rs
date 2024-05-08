use crate::my_imports::*;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct HPAProfile {
    /// Min allowed group size
    pub min_size: u64,
    /// Max allowed group size
    pub max_size: u64,

    /// When average group cpu AND group memory goes down below these parameters, scale down occurs (if min_size allows)
    pub scale_down_mean_cpu_fraction: f64,
    pub scale_down_mean_memory_fraction: f64,

    /// When average group cpu ОК group memory goes up above these parameters, scale up occurs (if max_size allows)
    pub scale_up_mean_cpu_fraction: f64,
    pub scale_up_mean_memory_fraction: f64,
}

impl Eq for HPAProfile {}

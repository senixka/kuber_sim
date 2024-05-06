use crate::my_imports::*;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct VPAProfile {
    pub min_allowed_cpu: i64,
    pub min_allowed_memory: i64,

    pub max_allowed_cpu: i64,
    pub max_allowed_memory: i64,

    pub reschedule_delay: f64,
    pub histogram_update_frequency: f64,

    pub gap_cpu: f64,
    pub gap_memory: f64,

    pub recommendation_margin_fraction: f64,
    pub limit_margin_fraction: f64,
}

impl Eq for VPAProfile {}

use crate::my_imports::*;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct VPAProfile {
    /// Min allowed cpu value when down-scale pod cpu request
    pub min_allowed_cpu: i64,
    /// Min allowed memory value when down-scale pod memory request
    pub min_allowed_memory: i64,

    /// Max allowed cpu value when up-scale pod cpu request
    pub max_allowed_cpu: i64,
    /// Max allowed memory value when up-scale pod memory request
    pub max_allowed_memory: i64,
}

impl Eq for VPAProfile {}

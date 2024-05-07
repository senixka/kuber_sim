use crate::my_imports::*;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct VPAProfile {
    pub min_allowed_cpu: i64,
    pub min_allowed_memory: i64,

    pub max_allowed_cpu: i64,
    pub max_allowed_memory: i64,
}

impl Eq for VPAProfile {}

use std::collections::HashMap;
use serde::{Deserialize, Serialize};


// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#objectmeta-v1-meta
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectMeta {
    pub labels: HashMap<String, String>,

    #[serde(skip_deserializing)]
    pub uid: u64,
}

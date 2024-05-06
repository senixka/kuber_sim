use crate::my_imports::*;

// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#objectmeta-v1-meta
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObjectMeta {
    pub labels: BTreeMap<String, String>,

    #[serde(skip)]
    pub uid: u64,
    #[serde(skip)]
    pub group_uid: u64,
}

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

impl FromStr for ObjectMeta {
    type Err = ();

    /// Expects "key_1:value_1,key_2:value_2,..."
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split(',').collect();
        let mut labels: BTreeMap<String, String> = BTreeMap::new();

        for key_value in data {
            let (key, value) = key_value.split_once(':').unwrap();
            labels.insert(key.to_string(), value.to_string());
        }

        Ok(Self {
            labels,
            uid: 0,
            group_uid: 0,
        })
    }
}

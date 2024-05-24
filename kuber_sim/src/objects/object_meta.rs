// https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.29/#objectmeta-v1-meta
#[derive(Debug, Clone, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ObjectMeta {
    pub labels: std::collections::BTreeMap<String, String>,

    #[serde(skip)]
    pub uid: u64,
    #[serde(skip)]
    pub group_uid: u64,
}

impl std::str::FromStr for ObjectMeta {
    type Err = ();

    /// Expects "key_1:value_1,key_2:value_2,..."
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split(',').collect();
        let mut labels = std::collections::BTreeMap::<String, String>::new();

        for key_value in data {
            let (key, value) = sim_some!(key_value.split_once(':'), "ObjectMeta. Invalid format.");
            labels.insert(key.to_string(), value.to_string());
        }

        Ok(Self {
            labels,
            uid: 0,
            group_uid: 0,
        })
    }
}

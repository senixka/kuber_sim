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

impl FromStr for VPAProfile {
    type Err = ();

    /// Expects "<i64>,<i64>,<i64>,<i64>"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split(';').collect();
        assert_eq!(data.len(), 4);

        Ok(Self {
            min_allowed_cpu: str::parse(data[0]).unwrap(),
            min_allowed_memory: str::parse(data[1]).unwrap(),
            max_allowed_cpu: str::parse(data[2]).unwrap(),
            max_allowed_memory: str::parse(data[3]).unwrap(),
        })
    }
}

impl Eq for VPAProfile {}

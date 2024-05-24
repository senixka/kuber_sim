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

    /// Expects "<i64>;<i64>;<i64>;<i64>"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split(';').collect();
        sim_assert!(data.len() == 4, "Invalid number of arguments in VPAProfile.");

        Ok(Self {
            min_allowed_cpu: sim_ok!(str::parse(data[0]), "VPAProfile[0]. Invalid value for min_allowed_cpu."),
            min_allowed_memory: sim_ok!(
                str::parse(data[1]),
                "VPAProfile[1]. Invalid value for min_allowed_memory."
            ),
            max_allowed_cpu: sim_ok!(str::parse(data[2]), "VPAProfile[2]. Invalid value for max_allowed_cpu."),
            max_allowed_memory: sim_ok!(
                str::parse(data[3]),
                "VPAProfile[3]. Invalid value for max_allowed_memory."
            ),
        })
    }
}

impl Eq for VPAProfile {}

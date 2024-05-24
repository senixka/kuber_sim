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

impl FromStr for HPAProfile {
    type Err = ();

    /// Expects "<u64>;<u64>;<f64>;<f64>;<f64>;<f64>"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split(';').collect();
        sim_assert!(data.len() == 6, "Invalid number of arguments in HPAProfile.");

        Ok(Self {
            min_size: sim_ok!(str::parse(data[0]), "HPAProfile[0]. Invalid value for min_size."),
            max_size: sim_ok!(str::parse(data[1]), "HPAProfile[1]. Invalid value for max_size."),
            scale_down_mean_cpu_fraction: sim_ok!(
                str::parse(data[2]),
                "HPAProfile[2]. Invalid value for scale_down_mean_cpu_fraction."
            ),
            scale_down_mean_memory_fraction: sim_ok!(
                str::parse(data[3]),
                "HPAProfile[3]. Invalid value for scale_down_mean_memory_fraction."
            ),
            scale_up_mean_cpu_fraction: sim_ok!(
                str::parse(data[4]),
                "HPAProfile[4]. Invalid value for scale_up_mean_cpu_fraction."
            ),
            scale_up_mean_memory_fraction: sim_ok!(
                str::parse(data[5]),
                "HPAProfile[5]. Invalid value for scale_up_mean_memory_fraction."
            ),
        })
    }
}

impl Eq for HPAProfile {}

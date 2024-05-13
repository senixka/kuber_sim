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

    /// Expects "<u64>,<u64>,<f64>,<f64>,<f64>,<f64>"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let data: Vec<&str> = s.split(';').collect();
        assert_eq!(data.len(), 6);

        Ok(Self {
            min_size: str::parse(data[0]).unwrap(),
            max_size: str::parse(data[1]).unwrap(),
            scale_down_mean_cpu_fraction: str::parse(data[2]).unwrap(),
            scale_down_mean_memory_fraction: str::parse(data[3]).unwrap(),
            scale_up_mean_cpu_fraction: str::parse(data[4]).unwrap(),
            scale_up_mean_memory_fraction: str::parse(data[5]).unwrap(),
        })
    }
}

impl Eq for HPAProfile {}

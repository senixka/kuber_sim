use crate::autoscaler::hpa::hpa_profile::HPAProfile;
use crate::autoscaler::vpa::vpa_profile::VPAProfile;
use crate::objects::pod::Pod;
use crate::simulation::init_trace::InitTrace;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct PodGroup {
    pub pod_count: u64,
    #[serde(default)]
    pub group_duration: f64,

    pub pod: Pod,

    #[serde(default)]
    pub hpa_profile: Option<HPAProfile>,
    #[serde(default)]
    pub vpa_profile: Option<VPAProfile>,

    #[serde(skip)]
    pub group_uid: u64,
}

impl std::str::FromStr for PodGroup {
    type Err = ();

    /// Expects "<pod_count: u64>;<group_duration: f64>;{<Pod>};{<HPAProfile>};{<VPAProfile>}"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (pod_count_str, other) = s.split_once(';').unwrap();
        let (group_duration_str, other) = other.trim().split_once(";").unwrap();
        let other = other.trim();

        let pod_end = InitTrace::find_matching_bracket(other, 0).unwrap();
        let hpa_end = InitTrace::find_matching_bracket(other, pod_end + 2).unwrap();

        let pod_str = &other[1..pod_end];
        let hpa_profile_str = &other[pod_end + 3..hpa_end];
        let vpa_profile_str = &other[hpa_end + 3..other.len() - 1];

        let mut group_duration: f64 = 0.0;
        if !group_duration_str.is_empty() {
            group_duration = str::parse(group_duration_str).unwrap();
        }

        let mut hpa_profile: Option<HPAProfile> = None;
        if !hpa_profile_str.is_empty() {
            hpa_profile = Some(str::parse(hpa_profile_str).unwrap());
        }

        let mut vpa_profile: Option<VPAProfile> = None;
        if !vpa_profile_str.is_empty() {
            vpa_profile = Some(str::parse(vpa_profile_str).unwrap());
        }

        Ok(Self {
            pod_count: str::parse(pod_count_str).unwrap(),
            group_duration,
            pod: str::parse(pod_str).unwrap(),
            hpa_profile,
            vpa_profile,
            group_uid: 0,
        })
    }
}

impl PodGroup {
    pub fn prepare(&mut self) {
        static UID_COUNTER: AtomicU64 = AtomicU64::new(1);

        self.group_uid = UID_COUNTER.load(Ordering::Relaxed);
        UID_COUNTER.fetch_add(1, Ordering::Relaxed);

        sim_assert!(self.group_duration >= 0.0, "PodGroup. group_duration must be >= 0.");
    }
}

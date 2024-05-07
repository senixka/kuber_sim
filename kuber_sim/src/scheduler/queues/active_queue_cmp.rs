use crate::my_imports::*;

pub trait TraitActiveQCmp: Ord {
    fn wrap(pod: Pod) -> Self;
    fn inner(&self) -> Pod;
}

pub type ActiveQCmpDefault = ActiveQCmpPriority;

/////////////////////////////////////////// Cmp Uid ////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ActiveQCmpUid(pub Pod);

impl TraitActiveQCmp for ActiveQCmpUid {
    #[inline]
    fn wrap(pod: Pod) -> Self {
        Self { 0: pod }
    }

    #[inline]
    fn inner(&self) -> Pod {
        self.0.clone()
    }
}

impl PartialOrd for ActiveQCmpUid {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ActiveQCmpUid {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.metadata.uid.cmp(&other.0.metadata.uid)
    }
}

impl PartialEq for ActiveQCmpUid {
    fn eq(&self, other: &Self) -> bool {
        self.0.metadata.uid == other.0.metadata.uid
    }
}

impl Eq for ActiveQCmpUid {}

//////////////////////////////////////// Cmp Priority //////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ActiveQCmpPriority(pub Pod);

impl TraitActiveQCmp for ActiveQCmpPriority {
    #[inline]
    fn wrap(pod: Pod) -> Self {
        Self { 0: pod }
    }

    #[inline]
    fn inner(&self) -> Pod {
        self.0.clone()
    }
}

impl PartialOrd for ActiveQCmpPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ActiveQCmpPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0.spec.priority, self.0.metadata.uid).cmp(&(other.0.spec.priority, other.0.metadata.uid))
    }
}

impl PartialEq for ActiveQCmpPriority {
    fn eq(&self, other: &Self) -> bool {
        self.0.metadata.uid == other.0.metadata.uid
    }
}

impl Eq for ActiveQCmpPriority {}

///////////////////////////////////////////// Test /////////////////////////////////////////////////

#[rustfmt::skip]
#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_active_queue_many_times() {
        for _ in 0..10 {
            test_active_min_queue_cmp_uid();
            test_active_max_queue_cmp_priority();
        }
    }

    #[test]
    fn test_active_min_queue_cmp_uid() {
        let mut q = ActiveMinQ::<ActiveQCmpUid>::new();

        let mut p1 = Pod::default(); p1.metadata.uid = 4;
        let mut p2 = Pod::default(); p2.metadata.uid = 1;
        let mut p3 = Pod::default(); p3.metadata.uid = 3;
        let mut p4 = Pod::default(); p4.metadata.uid = 2;

        q.push(p4.clone());
        q.push(p2.clone());
        q.push(p1.clone());
        q.push(p3.clone());

        assert_eq!(q.try_pop().unwrap(), p2);
        assert_eq!(q.try_pop().unwrap(), p4);
        assert_eq!(q.try_pop().unwrap(), p3);
        assert_eq!(q.try_pop().unwrap(), p1);
        assert_eq!(q.try_pop(), None);

        q.push(p4.clone());
        q.push(p2.clone());
        q.push(p1.clone());
        q.push(p3.clone());

        assert_eq!(q.try_remove(p2.clone()), true);
        assert_eq!(q.try_remove(p2.clone()), false);
        assert_eq!(q.try_pop().unwrap(), p4);
        assert_eq!(q.try_pop().unwrap(), p3);
        assert_eq!(q.try_pop().unwrap(), p1);
        assert_eq!(q.try_pop(), None);
        assert_eq!(q.try_remove(p1.clone()), false);

        // Insert many pods with uids in [1..500]
        let mut rng = rand::thread_rng();
        for _ in 1..2000 {
            let mut pod = Pod::default();
            pod.metadata.uid = rng.gen_range(1..=500);
            q.push(pod);
        }

        // Remove random pods
        let len_before = q.len();
        let mut rm_cnt = 0;
        for _ in 0..200 {
            let uid = rng.gen_range(1..=500);
            let mut pod = Pod::default();
            pod.metadata.uid = uid;

            rm_cnt += q.try_remove(pod) as usize;
        }
        assert_eq!(q.len() + rm_cnt, len_before);

        // Pop remaining pods
        let mut last_priority = i64::MAX;
        let mut uids: HashSet<u64> = HashSet::new();
        while let Some(pod) = q.try_pop() {
            assert!(last_priority >= pod.spec.priority);
            last_priority = pod.spec.priority;

            let newly_inserted = uids.insert(pod.metadata.uid);
            assert!(newly_inserted);
        }

        assert_eq!(q.try_pop(), None);
        assert_eq!(q.len(), 0);
    }

    #[test]
    pub fn test_active_max_queue_cmp_priority() {
        let mut q = ActiveMaxQ::<ActiveQCmpPriority>::new();

        let mut p1 = Pod::default(); p1.spec.priority = 1; p1.metadata.uid = 4;
        let mut p2 = Pod::default(); p2.spec.priority = 3; p2.metadata.uid = 1;
        let mut p3 = Pod::default(); p3.spec.priority = 3; p3.metadata.uid = 3;
        let mut p4 = Pod::default(); p4.spec.priority = 2; p4.metadata.uid = 2;

        q.push(p4.clone());
        q.push(p2.clone());
        q.push(p1.clone());
        q.push(p3.clone());

        assert_eq!(q.try_pop().unwrap(), p3);
        assert_eq!(q.try_pop().unwrap(), p2);
        assert_eq!(q.try_pop().unwrap(), p4);
        assert_eq!(q.try_pop().unwrap(), p1);
        assert_eq!(q.try_pop(), None);

        q.push(p4.clone());
        q.push(p2.clone());
        q.push(p1.clone());
        q.push(p3.clone());

        assert_eq!(q.try_remove(p2.clone()), true);
        assert_eq!(q.try_remove(p2.clone()), false);
        assert_eq!(q.try_pop().unwrap(), p3);
        assert_eq!(q.try_pop().unwrap(), p4);
        assert_eq!(q.try_pop().unwrap(), p1);
        assert_eq!(q.try_pop(), None);
        assert_eq!(q.try_remove(p1.clone()), false);

        // Insert many pods with priority in [-25..25]
        let mut rng = rand::thread_rng();
        let mut pod_spec: Vec<(u64, i64)> = Vec::new();
        for i in 1..2000 {
            assert_eq!(q.len(), i as usize - 1);

            let mut pod = Pod::default();
            pod.metadata.uid = i;
            pod.spec.priority = rng.gen_range(-25..=25);
            pod_spec.push((i, pod.spec.priority));
            q.push(pod);
        }

        // Remove random pods
        let len_before = q.len();
        let mut rm_cnt = 0;
        for _ in 0..200 {
            let (uid, priority) = pod_spec.remove(rng.gen_range(0..pod_spec.len()));
            let mut pod = Pod::default();
            pod.metadata.uid = uid;
            pod.spec.priority = priority;

            rm_cnt += q.try_remove(pod) as usize;
        }
        assert_eq!(q.len() + rm_cnt, len_before);

        // Pop remaining pods
        let mut last_priority = i64::MAX;
        let mut uids: HashSet<u64> = HashSet::new();
        while let Some(pod) = q.try_pop() {
            assert!(last_priority >= pod.spec.priority);
            last_priority = pod.spec.priority;

            let newly_inserted = uids.insert(pod.metadata.uid);
            assert!(newly_inserted);
        }

        assert_eq!(q.try_pop(), None);
        assert_eq!(q.len(), 0);
    }
}

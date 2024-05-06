use crate::my_imports::*;

pub trait TraitActiveQCmp: Ord {
    fn wrap(pod: Pod) -> Self;
    fn inner(&self) -> Pod;
}

pub type ActiveQCmpDefault = ActiveQCmpPriority;

////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////

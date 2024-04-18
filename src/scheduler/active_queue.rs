use std::cmp::Ordering;
use crate::my_imports::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait TraitActiveQCmp: Ord {
    fn wrap(pod: Pod) -> Self;

    fn inner(&self) -> Pod;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ActiveQCmpDefault = ActiveQCmpPriority;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ActiveQCmpMinUid(pub Pod);

impl TraitActiveQCmp for ActiveQCmpMinUid {
    #[inline]
    fn wrap(pod: Pod) -> Self {
        Self { 0: pod }
    }

    #[inline]
    fn inner(&self) -> Pod {
        self.0.clone()
    }
}

impl PartialOrd for ActiveQCmpMinUid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ActiveQCmpMinUid {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.metadata.uid.cmp(&self.0.metadata.uid)
    }
}

impl PartialEq for ActiveQCmpMinUid {
    fn eq(&self, other: &Self) -> bool {
        self.0.metadata.uid == other.0.metadata.uid
    }
}

impl Eq for ActiveQCmpMinUid {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct ActiveQCmpPriority(pub Pod);

impl TraitActiveQCmp for crate::scheduler::active_queue::ActiveQCmpPriority {
    #[inline]
    fn wrap(pod: Pod) -> Self {
        Self { 0: pod }
    }

    #[inline]
    fn inner(&self) -> Pod {
        self.0.clone()
    }
}

impl PartialOrd for crate::scheduler::active_queue::ActiveQCmpPriority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for crate::scheduler::active_queue::ActiveQCmpPriority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.spec.priority.cmp(&other.0.spec.priority)
    }
}

impl PartialEq for crate::scheduler::active_queue::ActiveQCmpPriority {
    fn eq(&self, other: &Self) -> bool {
        self.0.spec.priority == other.0.spec.priority
    }
}

impl Eq for crate::scheduler::active_queue::ActiveQCmpPriority {}

////////////////////////////////////////////////////////////////////////////////////////////////////

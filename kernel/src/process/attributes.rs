use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

use derive_more::Display;

macro_rules! int_type {
    ($name:ident, $underlying:ty) => {
        #[derive(Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name($underlying);

        impl From<$underlying> for $name {
            fn from(value: $underlying) -> Self {
                Self(value)
            }
        }

        impl ::core::ops::Deref for $name {
            type Target = $underlying;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl ::core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProcessId(pub(in crate::process) u64);

impl !Default for ProcessId {}

impl ProcessId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Relaxed))
    }
}

int_type!(ProcessGroupId, u64);
int_type!(EffectiveUserId, u32);
int_type!(EffectiveGroupId, u32);
int_type!(RealUserId, u32);
int_type!(RealGroupId, u32);
int_type!(SavedSetUserId, u32);
int_type!(SavedSetGroupId, u32);
int_type!(FileModeCreationMask, u16); // TODO: use a permission type once we have one

macro_rules! attributes {
    ($($name:ident : $typ:ty),*,) => {
        #[derive(Debug)]
        pub struct Attributes {
            $(pub $name: $typ,)*
        }
    };
}

attributes! {
    // TODO: controlling terminal
    // TODO: current working directory
    // TODO: root directory
    pgid: ProcessGroupId,
    euid: EffectiveUserId,
    egid: EffectiveGroupId,
    uid: RealUserId,
    gid: RealGroupId,
    suid: SavedSetUserId,
    sgid: SavedSetGroupId,
    // TODO: session membership
    // TODO: supplementary group ids
}

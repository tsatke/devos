use alloc::collections::BTreeMap;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;

use spin::RwLock;

use crate::process::fd::FilenoAllocator;
use crate::process::fd::{FileDescriptor, Fileno};

macro_rules! int_type {
    ($name:ident, $underlying:ty) => {
        #[derive(::derive_more::Display, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name($underlying);

        impl From<$underlying> for $name {
            fn from(value: $underlying) -> Self {
                Self(value)
            }
        }

        impl Deref for $name {
            type Target = $underlying;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

int_type!(ProcessId, u64);

impl !Default for ProcessId {}

impl ProcessId {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Relaxed).into()
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

        impl Attributes {
            pub fn builder() -> AttributeBuilder {
                AttributeBuilder::default()
            }
        }

        #[derive(Default)]
        pub struct AttributeBuilder {
            $($name: Option<$typ>,)*
        }

        impl AttributeBuilder {
            pub fn build(self) -> Attributes {
                Attributes {
                    $($name: self.$name.expect(concat!(stringify!($name), "must be set")),)*
                }
            }

            $(
                pub fn $name<I: ::core::convert::Into<$typ>>(&mut self, $name: I) -> &mut Self {
                    self.$name = Some(::core::convert::Into::<$typ>::into($name));
                    self
                }
            )*
        }
    };
}

attributes! {
    // TODO: controlling terminal
    // TODO: current working directory
    // TODO: root directory
    pid: ProcessId,
    euid: EffectiveUserId,
    egid: EffectiveGroupId,
    uid: RealUserId,
    gid: RealGroupId,
    suid: SavedSetUserId,
    sgid: SavedSetGroupId,
    next_fd: FilenoAllocator,
    open_fds: RwLock<BTreeMap<Fileno, FileDescriptor>>,
    // TODO: session membership
    // TODO: supplementary group ids
}

impl Attributes {
    pub fn create<F>(f: F) -> Self
    where
        F: FnOnce(&mut AttributeBuilder) -> &mut AttributeBuilder,
    {
        let mut builder = AttributeBuilder::default();
        f(&mut builder);
        builder.build()
    }
}

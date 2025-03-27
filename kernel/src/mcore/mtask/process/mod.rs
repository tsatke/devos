use crate::mem::address_space::AddressSpace;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
pub use id::*;

mod id;

static ROOT_PROCESS: OnceCell<Arc<Process>> = OnceCell::uninit();

pub struct Process {
    _pid: ProcessId,
    _name: String,

    address_space: Option<AddressSpace>,
}

impl Process {
    pub fn root() -> &'static Arc<Process> {
        ROOT_PROCESS.get_or_init(|| {
            Arc::new(Process {
                _pid: ProcessId::new(),
                _name: "root".to_string(),
                address_space: None,
            })
        })
    }

    pub fn address_space(&self) -> &AddressSpace {
        self.address_space
            .as_ref()
            .unwrap_or(AddressSpace::kernel())
    }
}

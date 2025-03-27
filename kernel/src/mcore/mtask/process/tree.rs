use crate::mcore::mtask::process::{Process, ProcessId};
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use spin::RwLock;

static PROCESS_TREE: OnceCell<RwLock<ProcessTree>> = OnceCell::uninit();

pub fn process_tree() -> &'static RwLock<ProcessTree> {
    PROCESS_TREE.get_or_init(|| {
        RwLock::new(ProcessTree {
            children: BTreeMap::default(),
            processes: BTreeMap::default(),
        })
    })
}

pub struct ProcessTree {
    pub children: BTreeMap<ProcessId, Vec<Arc<Process>>>,
    pub processes: BTreeMap<ProcessId, Arc<Process>>,
}

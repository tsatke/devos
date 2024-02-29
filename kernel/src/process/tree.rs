use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;

use spin::RwLock;
use x86_64::structures::paging::PageTableFlags;

use crate::process::attributes::ProcessId;
use crate::process::thread::ThreadId;
use crate::process::{current, Process};
use crate::serial_println;

static PROCESS_TREE: RwLock<ProcessTree> = RwLock::new(ProcessTree::new());

pub fn process_tree() -> &'static RwLock<ProcessTree> {
    &PROCESS_TREE
}

pub struct ProcessTree {
    root_pid: Option<ProcessId>,
    processes_by_id: BTreeMap<ProcessId, Process>,
    parents: BTreeMap<ProcessId, ProcessId>,
    children: BTreeMap<ProcessId, BTreeSet<ProcessId>>,
    threads: BTreeMap<ProcessId, BTreeSet<ThreadId>>,
}

impl Default for ProcessTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessTree {
    pub const fn new() -> Self {
        Self {
            root_pid: None,
            processes_by_id: BTreeMap::new(),
            parents: BTreeMap::new(),
            children: BTreeMap::new(),
            threads: BTreeMap::new(),
        }
    }

    pub fn process_by_id(&self, process_id: &ProcessId) -> Option<&Process> {
        self.processes_by_id.get(process_id)
    }

    pub fn set_root(&mut self, process: Process) {
        if self.root_pid.is_some() {
            panic!("root process already set");
        }

        let process_id = *process.pid();
        self.root_pid = Some(process_id);
        self.processes_by_id.insert(process_id, process);
    }

    pub fn insert_process(&mut self, parent: Process, process: Process) {
        let parent_process_id = *parent.pid();
        let child_process_id = *process.pid();
        self.processes_by_id
            .insert(child_process_id, process.clone());
        self.children
            .entry(parent_process_id)
            .or_default()
            .insert(child_process_id);
        self.parents.insert(child_process_id, parent_process_id);
    }

    pub fn remove_process(&mut self, process_id: &ProcessId) -> Option<Process> {
        let p = self.processes_by_id.remove(process_id);
        self.parents.remove(process_id);
        if let Some(_children) = self.children.remove(process_id) {
            todo!("change parent of orphan, old parent pid={}", process_id);
        };
        self.children.values_mut().for_each(|children| {
            children.remove(process_id);
        });
        self.threads.remove(process_id);
        p
    }

    pub fn add_thread(&mut self, process_id: &ProcessId, thread_id: &ThreadId) {
        self.threads
            .entry(*process_id)
            .or_default()
            .insert(*thread_id);
    }

    pub fn remove_thread(&mut self, process_id: &ProcessId, thread_id: &ThreadId) {
        self.threads
            .entry(*process_id)
            .or_default()
            .remove(thread_id);
    }

    pub fn threads(&self, process_id: &ProcessId) -> Option<impl Iterator<Item = &ThreadId>> {
        self.threads.get(process_id).map(|threads| threads.iter())
    }

    pub fn has_threads(&self, process_id: &ProcessId) -> bool {
        self.threads(process_id)
            .map_or(false, |threads| threads.count() > 0)
    }

    pub fn dump(&self) {
        serial_println!("=== start process tree dump");
        self.dump_process_and_children(&self.root_pid.expect("no root process set"), 4);
        serial_println!("=== end process tree dump");
    }

    pub fn dump_current(&self) {
        serial_println!("=== start process dump");
        self.dump_process_no_children(current().pid(), 4);
        serial_println!("=== end process dump");
    }

    fn dump_process_and_children(&self, process_id: &ProcessId, indent: usize) {
        self.dump_process_no_children(process_id, indent);

        if let Some(children) = self.children.get(process_id) {
            for child in children {
                self.dump_process_and_children(child, indent + 4);
            }
        }
    }

    fn dump_process_no_children(&self, process_id: &ProcessId, indent: usize) {
        let process = self.processes_by_id.get(process_id).unwrap();
        let process_name = process.name();
        let process_id = process.pid();
        let threads = self
            .threads(process_id)
            .map_or(0, |threads| threads.count());
        let children = self
            .children
            .get(process_id)
            .map_or(0, |children| children.len());
        let vm_objects = process.vmm().vm_objects().read().len();
        let open_fds = process.open_fds().read().len();

        serial_println!(
            "{:indent$}{} (pid={}, threads={}, children={}, vm_objects={}, open_fds={})",
            "",
            process_name,
            process_id,
            threads,
            children,
            vm_objects,
            open_fds,
            indent = indent
        );

        for (_, vm_object) in process.vmm().vm_objects().read().iter() {
            serial_println!(
                "{:indent$}*vm_object: {:#p}-{:#p} {:#016x} {} {}",
                "",
                vm_object.addr(),
                vm_object.addr() + vm_object.size(),
                vm_object.size(),
                page_table_flags_to_string(vm_object.flags()),
                vm_object.name(),
                indent = indent + 4
            )
        }

        for (fileno, fd) in process.open_fds().read().iter() {
            serial_println!(
                "{:indent$}*open_fd: {} (fileno={})",
                "",
                fd.node().path(),
                fileno,
                indent = indent + 4
            )
        }
    }
}

fn page_table_flags_to_string(flags: PageTableFlags) -> String {
    macro_rules! flag {
        ($buf:expr, $flag:expr, $char:literal) => {
            if flags.contains($flag) {
                $buf.push($char);
            } else {
                $buf.push('-');
            }
        };
        ($buf:expr, not $flag:expr, $char:literal) => {
            if !flags.contains($flag) {
                $buf.push($char);
            } else {
                $buf.push('-');
            }
        };
    }
    let mut s = String::new();
    flag!(s, PageTableFlags::USER_ACCESSIBLE, 'u');
    flag!(s, PageTableFlags::PRESENT, 'r');
    flag!(s, PageTableFlags::WRITABLE, 'w');
    flag!(s, not PageTableFlags::NO_EXECUTE, 'x');
    s
}

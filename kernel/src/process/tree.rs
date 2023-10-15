use crate::process::task::TaskId;
use crate::process::{Process, ProcessId};
use crate::serial_println;
use alloc::collections::{BTreeMap, BTreeSet};

pub struct ProcessTree {
    root_pid: ProcessId,
    processes_by_id: BTreeMap<ProcessId, Process>,
    parents: BTreeMap<ProcessId, ProcessId>,
    children: BTreeMap<ProcessId, BTreeSet<ProcessId>>,
    tasks: BTreeMap<ProcessId, BTreeSet<TaskId>>,
}

impl ProcessTree {
    pub fn new(root_process: Process, root_process_task: &TaskId) -> Self {
        let root_pid = *root_process.process_id();
        let mut s = Self {
            root_pid,
            processes_by_id: BTreeMap::new(),
            parents: BTreeMap::new(),
            children: BTreeMap::new(),
            tasks: BTreeMap::new(),
        };
        s.processes_by_id.insert(root_pid, root_process);
        s.add_task(&root_pid, root_process_task);
        s
    }

    pub fn process_by_id(&self, process_id: &ProcessId) -> Option<&Process> {
        self.processes_by_id.get(process_id)
    }

    pub fn insert_process(&mut self, parent: Process, process: Process) {
        let parent_process_id = *parent.process_id();
        let child_process_id = *process.process_id();
        self.processes_by_id
            .insert(child_process_id, process.clone());
        self.children
            .entry(parent_process_id)
            .or_insert_with(BTreeSet::new)
            .insert(child_process_id);
        self.parents.insert(child_process_id, parent_process_id);
    }

    pub fn remove_process(&mut self, process_id: &ProcessId) -> Option<Process> {
        let p = self.processes_by_id.remove(process_id);
        self.parents.remove(process_id);
        if let Some(_children) = self.children.remove(process_id) {
            todo!("change parent of orphan, old parent pid={}", process_id);
        };
        self.tasks.remove(process_id);
        p
    }

    pub fn add_task(&mut self, process_id: &ProcessId, task_id: &TaskId) {
        self.tasks
            .entry(*process_id)
            .or_insert_with(BTreeSet::new)
            .insert(*task_id);
    }

    pub fn remove_task(&mut self, process_id: &ProcessId, task_id: &TaskId) {
        self.tasks
            .entry(*process_id)
            .or_insert_with(BTreeSet::new)
            .remove(task_id);
    }

    pub fn tasks(&self, process_id: &ProcessId) -> Option<impl Iterator<Item = &TaskId>> {
        self.tasks.get(process_id).map(|tasks| tasks.iter())
    }

    pub fn has_tasks(&self, process_id: &ProcessId) -> bool {
        self.tasks(process_id)
            .map_or(false, |tasks| tasks.count() > 0)
    }

    pub fn dump(&self) {
        serial_println!("=== start process tree dump");
        self.dump_process(&self.root_pid, 4);
        serial_println!("=== end process tree dump");
    }

    fn dump_process(&self, process_id: &ProcessId, indent: usize) {
        let process = self.processes_by_id.get(process_id).unwrap();
        let process_name = process.name();
        let process_id = process.process_id();
        let tasks = self.tasks(process_id).map_or(0, |tasks| tasks.count());
        let children = self
            .children
            .get(process_id)
            .map_or(0, |children| children.len());
        serial_println!(
            "{:indent$}{} (pid={}, tasks={}, children={})",
            "",
            process_name,
            process_id,
            tasks,
            children,
            indent = indent
        );
        if let Some(children) = self.children.get(process_id) {
            for child in children {
                self.dump_process(child, indent + 4);
            }
        }
    }
}

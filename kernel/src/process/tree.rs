use crate::process::task::TaskId;
use crate::process::{Process, ProcessId};
use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::vec::Vec;
use core::iter::FusedIterator;
use core::ops::{Deref, DerefMut};

pub struct ProcessTree {
    root: ProcessTreeNode,
    processes: BTreeMap<ProcessId, Process>,
}

pub struct ProcessTreeNode {
    process: Process,
    children: Vec<ProcessTreeNode>,
    tasks: BTreeSet<TaskId>,
}

impl Deref for ProcessTreeNode {
    type Target = Process;

    fn deref(&self) -> &Self::Target {
        &self.process
    }
}

impl DerefMut for ProcessTreeNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.process
    }
}

impl ProcessTreeNode {
    fn new(process: Process) -> Self {
        ProcessTreeNode {
            process,
            children: Vec::new(),
            tasks: BTreeSet::new(),
        }
    }
}

impl ProcessTree {
    pub fn new(root_process: Process) -> Self {
        let mut map = BTreeMap::new();
        map.insert(*root_process.process_id(), root_process.clone());
        Self {
            root: ProcessTreeNode::new(root_process),
            processes: map,
        }
    }

    pub fn process(&self, process_id: &ProcessId) -> Option<&Process> {
        self.processes.get(process_id)
    }

    pub fn iter(&self) -> BfsIterator<'_> {
        let mut queue = VecDeque::new();
        queue.push_back(&self.root);
        BfsIterator { queue }
    }

    pub fn find_parent(&self, child_id: &ProcessId) -> Option<&ProcessTreeNode> {
        self.iter().find(|node| {
            node.children
                .iter()
                .find(|child| child.process_id() == child_id)
                .is_some()
        })
    }

    pub fn find_process(&self, task_id: &TaskId) -> Option<&ProcessTreeNode> {
        self.iter().find(|node| node.tasks.contains(task_id))
    }
}

pub struct BfsIterator<'a> {
    queue: VecDeque<&'a ProcessTreeNode>,
}

impl<'a> Iterator for BfsIterator<'a> {
    type Item = &'a ProcessTreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_front()?;
        for child in &node.children {
            self.queue.push_back(child);
        }
        Some(node)
    }
}

impl FusedIterator for BfsIterator<'_> {}

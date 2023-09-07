use crate::io::vfs::IoError::Unsupported;
use crate::io::vfs::LookupError::NoSuchEntry;
use crate::io::vfs::MountError::ExistsButShouldNot;
use crate::io::vfs::{
    CreateError, CreateNodeType, IDir, INode, INodeBase, INodeNum, LookupError, MountError,
    Permission, Stat,
};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub struct RootDir {
    name: String,
    stat: Stat,
    children: BTreeMap<String, INode>,
}

impl RootDir {
    pub fn new(name: String, stat: Stat) -> Self {
        Self {
            name,
            stat,
            children: BTreeMap::new(),
        }
    }
}

impl INodeBase for RootDir {
    fn num(&self) -> INodeNum {
        self.stat.inode
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn stat(&self) -> Stat {
        self.stat
    }
}

impl IDir for RootDir {
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<INode, LookupError> {
        if let Some(inode) = self.children.get(name.as_ref()) {
            return Ok(inode.clone());
        }
        Err(NoSuchEntry)
    }

    fn create(
        &mut self,
        _name: &dyn AsRef<str>,
        _typ: CreateNodeType,
        _permission: Permission,
    ) -> Result<INode, CreateError> {
        Err(CreateError::IoError(Unsupported))
    }

    fn children(&self) -> Result<Vec<INode>, LookupError> {
        Ok(self.children.values().cloned().collect())
    }

    fn mount(&mut self, node: INode) -> Result<(), MountError> {
        if self.children.get(&node.name()).is_some() {
            return Err(ExistsButShouldNot);
        }
        self.children.insert(node.name(), node);
        Ok(())
    }
}

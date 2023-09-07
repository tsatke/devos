use crate::io::vfs::IoError::Unsupported;
use crate::io::vfs::LookupError::NoSuchEntry;
use crate::io::vfs::{
    CreateError, CreateNodeType, Dir, Inode, InodeBase, InodeNum, LookupError, Permission, Stat,
};
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

pub struct RootDir {
    name: String,
    stat: Stat,
    children: BTreeMap<String, Inode>,
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

impl InodeBase for RootDir {
    fn num(&self) -> InodeNum {
        self.stat.inode
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn stat(&self) -> Stat {
        self.stat
    }
}

impl Dir for RootDir {
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<Inode, LookupError> {
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
    ) -> Result<Inode, CreateError> {
        Err(CreateError::IoError(Unsupported))
    }

    fn children(&self) -> Result<Vec<Inode>, LookupError> {
        Ok(self.children.values().cloned().collect())
    }
}

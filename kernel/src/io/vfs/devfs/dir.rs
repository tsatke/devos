use crate::io::vfs::devfs::CharDev;
use crate::io::vfs::{
    CreateError, CreateNodeType, Dir, Inode, InodeBase, LookupError, Permission, Stat,
};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub struct DevFsDir {
    name: String,
    fsid: u64,
    children: Vec<Inode>,
}

impl DevFsDir {
    pub fn new(fsid: u64, name: String) -> Self {
        Self {
            name,
            fsid,
            children: vec![
                Inode::new_character_device_file(CharDev::new(
                    fsid,
                    1_u64.into(),
                    "null".to_string(),
                    |buf: &mut dyn AsMut<[u8]>| {
                        let buf = buf.as_mut();
                        buf.fill(0);
                        Ok(buf.len())
                    },
                    |buf: &dyn AsRef<[u8]>| Ok(buf.as_ref().len()),
                )),
                Inode::new_character_device_file(CharDev::new(
                    fsid,
                    2_u64.into(),
                    "zero".to_string(),
                    |buf: &mut dyn AsMut<[u8]>| {
                        let buf = buf.as_mut();
                        buf.fill(0);
                        Ok(buf.len())
                    },
                    |buf: &dyn AsRef<[u8]>| Ok(buf.as_ref().len()),
                )),
            ],
        }
    }
}

impl InodeBase for DevFsDir {
    fn name(&self) -> String {
        self.name.clone() // TODO: remove clone
    }

    fn stat(&self) -> Stat {
        Stat {
            dev: self.fsid,
            inode: 0_u64.into(),
            ..Default::default()
        }
    }
}

impl Dir for DevFsDir {
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<Inode, LookupError> {
        todo!()
    }

    fn create(
        &mut self,
        name: &dyn AsRef<str>,
        typ: CreateNodeType,
        permission: Permission,
    ) -> Result<Inode, CreateError> {
        todo!()
    }

    fn children(&self) -> Result<Vec<Inode>, LookupError> {
        Ok(self.children.clone())
    }
}

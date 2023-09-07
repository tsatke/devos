use crate::io::vfs::LookupError::NoSuchEntry;
use crate::io::vfs::{
    CreateError, CreateNodeType, Dir, File, Fs, Inode, InodeBase, InodeNum, LookupError,
    Permission, ReadError, Stat, WriteError,
};
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;
use spin::RwLock;

pub struct MemFs {
    #[allow(dead_code)] // TODO: inner is read by tests, but remove it anyways
    inner: InnerHandle,
    root: Inode,
}

type InnerHandle = Arc<RwLock<Inner>>;

struct Inner {
    nodes: BTreeMap<InodeNum, Inode>,
    inode_counter: AtomicU64,
    fsid: u64,
}

impl Inner {
    fn get_unused_inode_num(&self) -> InodeNum {
        self.inode_counter.fetch_add(1, Relaxed).into()
    }
}

impl MemFs {
    pub fn new(fsid: u64) -> Self {
        let inner = Arc::new(RwLock::new(Inner {
            nodes: BTreeMap::new(),
            inode_counter: AtomicU64::new(1),
            fsid,
        }));

        let root_inode_num = 0_u64.into();
        let root_dir = MemDir::new(inner.clone(), "/".to_string(), root_inode_num);
        let root = Inode::new_dir(root_dir);
        inner.write().nodes.insert(root_inode_num, root.clone());

        Self { inner, root }
    }
}

impl Fs for MemFs {
    fn root_inode(&self) -> Inode {
        self.root.clone()
    }
}

struct MemNodeBase {
    fs: Arc<RwLock<Inner>>,
    stat: Stat,
    name: String,
}

impl MemNodeBase {
    fn new(fs: InnerHandle, name: String, stat: Stat) -> Self {
        Self { fs, name, stat }
    }
}

impl InodeBase for MemNodeBase {
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

pub struct MemFile {
    base: MemNodeBase,
    data: Vec<u8>,
}

impl MemFile {
    fn new(fs: InnerHandle, name: String, inode_num: InodeNum, data: Vec<u8>) -> Self {
        Self {
            base: MemNodeBase::new(
                fs,
                name,
                Stat {
                    inode: inode_num,
                    size: data.len() as u64,
                    ..Default::default()
                },
            ),
            data,
        }
    }
}

impl InodeBase for MemFile {
    fn num(&self) -> InodeNum {
        self.base.num()
    }

    fn name(&self) -> String {
        self.base.name()
    }

    fn stat(&self) -> Stat {
        self.base.stat()
    }
}

impl File for MemFile {
    fn size(&self) -> u64 {
        self.stat().size
    }

    fn truncate(&mut self, size: u64) -> Result<(), WriteError> {
        let new_size = TryInto::<usize>::try_into(size).unwrap(); // u64 -> usize is valid on x86_64
        self.data.resize(new_size, 0);
        self.base.stat.size = self.data.len() as u64;
        Ok(())
    }

    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize, ReadError> {
        let buffer = buf.as_mut();
        let length = buffer.len();
        if offset as usize + length > self.data.len() {
            return Err(ReadError::InvalidOffset(offset as usize));
        }
        buffer.copy_from_slice(&self.data[offset as usize..offset as usize + length]);
        Ok(length)
    }

    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> Result<usize, WriteError> {
        let buffer = buf.as_ref();
        let length = buffer.len();
        if offset as usize + length > self.data.len() {
            return Err(WriteError::InvalidOffset(offset as usize));
        }
        self.data[offset as usize..offset as usize + length].copy_from_slice(buffer);
        Ok(length)
    }
}

pub struct MemDir {
    base: MemNodeBase,
    children: Vec<InodeNum>,
}

impl MemDir {
    fn new(fs: InnerHandle, name: String, inode_num: InodeNum) -> Self {
        Self {
            base: MemNodeBase::new(
                fs,
                name,
                Stat {
                    inode: inode_num,
                    ..Default::default()
                },
            ),
            children: Vec::new(),
        }
    }
}

impl InodeBase for MemDir {
    fn num(&self) -> InodeNum {
        self.base.num()
    }

    fn name(&self) -> String {
        self.base.name()
    }

    fn stat(&self) -> Stat {
        self.base.stat()
    }
}

impl Dir for MemDir {
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<Inode, LookupError> {
        let needle = name.as_ref();
        let guard = self.base.fs.read();
        match self
            .children
            .iter()
            .filter_map(|n| guard.nodes.get(n))
            .find(|n| n.name() == needle)
        {
            None => Err(NoSuchEntry),
            Some(n) => Ok(n.clone()),
        }
    }

    fn create(
        &mut self,
        name: &dyn AsRef<str>,
        typ: CreateNodeType,
        _permission: Permission,
    ) -> Result<Inode, CreateError> {
        let name = name.as_ref().to_string();
        let inode_num = self.base.fs.read().get_unused_inode_num();
        let inode = match typ {
            CreateNodeType::File => {
                let f = MemFile::new(self.base.fs.clone(), name, inode_num, vec![]);
                Inode::new_file(f)
            }
            CreateNodeType::Dir => {
                let d = MemDir::new(self.base.fs.clone(), name, inode_num);
                Inode::new_dir(d)
            }
        };
        self.children.push(inode_num);
        Ok(inode)
    }

    fn children(&self) -> Result<Vec<Inode>, LookupError> {
        let guard = self.base.fs.read();
        Ok(self
            .children
            .iter()
            .filter_map(|n| guard.nodes.get(n))
            .cloned()
            .collect())
    }
}

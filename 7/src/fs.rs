use std::collections::BTreeMap;

use nonempty::{nonempty, NonEmpty};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct BlockRef(usize);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct INode {
    size: usize,
}

impl INode {
    fn new(size: usize) -> Self {
        Self { size }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DNode {
    files: BTreeMap<String, BlockRef>,
    children: BTreeMap<String, BlockRef>,
    pub size: usize,
}

impl DNode {
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            children: BTreeMap::new(),
            size: 0,
        }
    }

    fn child_size(&self, fs: &Filesystem) -> usize {
        self.children
            .values()
            .filter_map(|br| fs.get_dir(br).map(|f| f.size))
            .sum()
    }

    fn add_file(&mut self, path: String, ptr: BlockRef, size: usize) {
        self.files.insert(path, ptr);
        self.size += size;
    }

    fn add_directory(&mut self, path: String, ptr: BlockRef) {
        self.children.insert(path, ptr);
    }
}

#[derive(Debug)]
pub(crate) enum FsItem {
    INode(INode),
    DNode(DNode),
}

impl FsItem {
    pub fn is_dir(&self) -> bool {
        match self {
            Self::INode(_) => false,
            Self::DNode(_) => true,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::INode(i) => i.size,
            Self::DNode(i) => i.size,
        }
    }
}

impl From<DNode> for FsItem {
    fn from(d: DNode) -> Self {
        FsItem::DNode(d)
    }
}

impl From<INode> for FsItem {
    fn from(i: INode) -> Self {
        FsItem::INode(i)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Path {
    components: NonEmpty<(String, BlockRef)>,
}

impl Path {
    fn new(root: BlockRef) -> Self {
        Self {
            components: nonempty![("".to_owned(), root)],
        }
    }

    pub fn pop_up(&mut self) {
        self.components.pop();
    }

    pub fn cd<S: Into<String>>(&mut self, path: S, fs: &Filesystem) -> anyhow::Result<()> {
        let last_block = self.components.last().1;
        let path = path.into();
        let new_block = fs.get_child(last_block, path.as_str())?;
        self.components.push((path, new_block));
        Ok(())
    }

    fn with<S: Into<String>, R: AsBlockRef>(&self, path: S, block: &R) -> Self {
        let mut new = self.clone();
        new.components.push((path.into(), block.as_block_ref()));
        new
    }
}

pub(crate) trait AsBlockRef {
    fn as_block_ref(&self) -> BlockRef;
}

impl AsBlockRef for BlockRef {
    fn as_block_ref(&self) -> BlockRef {
        *self
    }
}

impl AsBlockRef for Path {
    fn as_block_ref(&self) -> BlockRef {
        self.components.last().1
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.components.len() == 1 {
            return write!(f, "/");
        }
        for item in
            itertools::Itertools::intersperse(self.components.iter().map(|c| c.0.as_str()), "/")
        {
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Blocks {
    blocks: Vec<FsItem>,
}

impl Blocks {
    fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    fn alloc_file(&mut self, size: usize) -> BlockRef {
        let inode = INode::new(size);
        let index = self.blocks.len();
        self.blocks.push(inode.into());
        BlockRef(index)
    }
    fn alloc_directory(&mut self) -> BlockRef {
        let dnode = DNode::new();
        let index = self.blocks.len();
        self.blocks.push(dnode.into());
        BlockRef(index)
    }

    fn get(&self, index: BlockRef) -> Option<&FsItem> {
        self.blocks.get(index.0)
    }

    fn get_mut(&mut self, index: BlockRef) -> Option<&mut FsItem> {
        self.blocks.get_mut(index.0)
    }
}

#[derive(Debug)]
pub(crate) struct Filesystem {
    blocks: Blocks,
    root: BlockRef,
}

impl Filesystem {
    pub fn new() -> Self {
        let mut blocks = Blocks::new();
        let root = blocks.alloc_directory();
        Self { root, blocks }
    }

    pub fn get_item<R: AsBlockRef>(&self, block: &R) -> Option<&FsItem> {
        self.blocks.get(block.as_block_ref())
    }

    pub fn get_dir<R: AsBlockRef>(&self, block: &R) -> Option<&DNode> {
        if let Some(FsItem::DNode(d)) = self.blocks.get(block.as_block_ref()) {
            Some(d)
        } else {
            None
        }
    }

    pub fn get_mut_dir(&mut self, block: BlockRef) -> Option<&mut DNode> {
        if let Some(FsItem::DNode(d)) = self.blocks.get_mut(block) {
            Some(d)
        } else {
            None
        }
    }

    pub fn add_directory<S: Into<String>, R: AsBlockRef>(
        &mut self,
        parent: &R,
        name: S,
    ) -> anyhow::Result<BlockRef> {
        let dir = self.blocks.alloc_directory();
        if let Some(parent) = self.get_mut_dir(parent.as_block_ref()) {
            parent.add_directory(name.into(), dir);
            Ok(dir)
        } else {
            anyhow::bail!("could not find parent directory");
        }
    }

    pub fn add_file<S: Into<String>, R: AsBlockRef>(
        &mut self,
        parent: &R,
        name: S,
        size: usize,
    ) -> anyhow::Result<BlockRef> {
        let file = self.blocks.alloc_file(size);
        if let Some(parent) = self.get_mut_dir(parent.as_block_ref()) {
            parent.add_file(name.into(), file, size);
            Ok(file)
        } else {
            anyhow::bail!("could not find parent directory");
        }
    }

    pub fn get_root_dir(&self) -> &DNode {
        self.get_dir(&self.get_root()).unwrap()
    }

    pub fn get_root(&self) -> BlockRef {
        self.root
    }

    pub fn get_root_path(&self) -> Path {
        Path::new(self.root)
    }

    fn get_child(&self, cwd: BlockRef, name: &str) -> anyhow::Result<BlockRef> {
        self.get_dir(&cwd)
            .and_then(|parent| parent.children.get(name))
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("could not find parent!"))
    }

    pub fn cache_directory_sizes(&mut self) -> anyhow::Result<()> {
        let mut stack = vec![self.get_root()];
        let mut traversal = vec![];
        while let Some(item) = stack.pop() {
            if let Some(d) = self.get_dir(&item) {
                for child in d.children.values() {
                    stack.push(*child);
                }
                traversal.push(item);
            }
        }
        for block_ref in traversal.into_iter().rev() {
            let size = self
                .get_dir(&block_ref)
                .map(|d| d.child_size(self))
                .ok_or_else(|| anyhow::anyhow!("failed to compute size!"))?;
            if let Some(d) = self.get_mut_dir(block_ref) {
                d.size += size;
            }
        }
        Ok(())
    }

    pub fn walk<F, T>(&self, mut f: F)
    where
        F: FnMut(Path, &FsItem) -> T,
    {
        let mut stack = vec![self.get_root_path()];
        while let Some(path) = stack.pop() {
            let item = self.get_item(&path).unwrap();
            if let FsItem::DNode(d) = item {
                for (name, child) in d.children.iter() {
                    stack.push(path.with(name, child));
                }
                for (name, child) in d.files.iter() {
                    stack.push(path.with(name, child));
                }
            }
            f(path, item);
        }
    }
}

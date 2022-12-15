use std::io::BufRead;

use clap::{Parser, ValueEnum};

mod fs {
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
}

#[derive(ValueEnum, Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Part1,
    Part2,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_enum)]
    mode: Mode,
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Ls,
}

fn populate_filesystem_from_commands<R: BufRead>(reader: R) -> anyhow::Result<fs::Filesystem> {
    let mut fs = fs::Filesystem::new();
    let mut cwd = fs.get_root_path();
    let mut command = None;
    for line in reader.lines() {
        let line = line?;
        if line.starts_with('$') {
            let mut parts = line.split(' ');
            let command_run = parts
                .nth(1)
                .ok_or_else(|| anyhow::anyhow!("invalid command line"))?;
            match command_run {
                "cd" => {
                    let target_path = parts
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("missing path for cd"))?;
                    if target_path == "/" {
                        cwd = fs.get_root_path();
                    } else if target_path == ".." {
                        cwd.pop_up();
                    } else {
                        cwd.cd(target_path, &fs)?;
                    }
                }
                "ls" => command = Some(Command::Ls),
                c => anyhow::bail!("unhandled command {}", c),
            }
        } else {
            match command {
                Some(Command::Ls) => {
                    if let Some((stat, label)) = line.split_once(' ') {
                        if stat == "dir" {
                            fs.add_directory(&cwd, label)?;
                        } else {
                            let size = stat.parse()?;
                            fs.add_file(&cwd, label, size)?;
                        }
                    } else {
                        anyhow::bail!("invalid output line {:?}", line);
                    }
                }
                None => anyhow::bail!("output without a running command"),
            }
        }
    }
    Ok(fs)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut fs = populate_filesystem_from_commands(&mut handle)?;
    fs.cache_directory_sizes()?;
    match args.mode {
        Mode::Part1 => {
            let mut total_size = 0;
            fs.walk(|_, item| {
                if item.is_dir() && item.size() < 100000 {
                    total_size += item.size();
                }
            });
            println!("total_size = {}", total_size);
        }
        Mode::Part2 => {
            let mut best_candidate = None;
            let root_size = fs.get_root_dir().size;
            if root_size > 70000000 {
                anyhow::bail!("FS is too big!");
            }
            let free = 70000000 - root_size;
            if free > 30000000 {
                anyhow::bail!("FS already has 30000000B free");
            }
            let needed = 30000000 - free;
            fs.walk(|path, item| {
                if item.is_dir() && item.size() > needed {
                    match best_candidate {
                        None => best_candidate = Some((path, item.size())),
                        Some((_, c)) if c > item.size() => {
                            best_candidate = Some((path, item.size()))
                        }
                        _ => {}
                    }
                }
            });
            if let Some((path, size)) = best_candidate {
                println!("{} is {}B", path, size);
            }
        }
    }
    Ok(())
}

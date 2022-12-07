use std::collections::BTreeMap;
use std::io::BufRead;

use clap::{Parser, ValueEnum};

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
struct File {
    size: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct Directory {
    files: BTreeMap<String, File>,
    children: BTreeMap<String, Directory>,
    size: usize,
}

impl Directory {
    fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            children: BTreeMap::new(),
            size: 0,
        }
    }

    fn cache_directory_sizes(&mut self) {
        self.size = self.files.values().map(|f| f.size).sum();
        if self.children.len() > 0 {
            for child in self.children.values_mut() {
                child.cache_directory_sizes();
            }
            self.size += self.children.values().map(|f| f.size).sum::<usize>();
        }
    }

    fn walk<F, T>(&self, start_path: Path, f: &mut F)
    where
        F: FnMut(Path, &Directory) -> T,
    {
        f(start_path.clone(), &self);
        for (name, child) in self.children.iter() {
            let path = start_path.with(name);
            child.walk(path, f);
        }
    }

    fn get_mut(&mut self, components: &Path) -> anyhow::Result<&mut Directory> {
        let mut top: &mut Directory = self;
        for component in components.components.iter() {
            top = top
                .children
                .get_mut(component)
                .ok_or_else(|| anyhow::anyhow!("invalid path!"))?;
        }
        Ok(top)
    }

    fn add_file(&mut self, path: &str, size: usize) {
        self.files.insert(path.to_owned(), File { size });
    }

    fn add_directory(&mut self, path: &str) {
        self.children.insert(path.to_owned(), Directory::new());
    }
}

#[derive(Debug)]
struct Filesystem {
    root: Directory,
}

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Ls,
}

#[derive(Debug, Clone)]
struct Path {
    components: Vec<String>,
}

impl Path {
    fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    fn pop_up(&mut self) {
        self.components.pop();
    }

    fn push<S: Into<String>>(&mut self, component: S) {
        self.components.push(component.into());
    }

    fn with<S: Into<String>>(&self, component: S) -> Self {
        let mut new = self.clone();
        new.push(component);
        new
    }
}

impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "/{}", self.components.join("/"))
    }
}

impl Filesystem {
    fn parse<R: BufRead>(reader: R) -> anyhow::Result<Self> {
        let mut root = Directory::new();
        let mut cwd = Path::new();
        let mut command = None;
        for line in reader.lines() {
            let line = line?;
            if line.chars().next() == Some('$') {
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
                            cwd = Path::new();
                        } else if target_path == ".." {
                            cwd.pop_up();
                        } else {
                            cwd.push(target_path);
                        }
                    }
                    "ls" => command = Some(Command::Ls),
                    c => anyhow::bail!("unhandled command {}", c),
                }
                // command
            } else {
                match command {
                    Some(Command::Ls) => {
                        if let Some((stat, label)) = line.split_once(' ') {
                            if stat == "dir" {
                                root.get_mut(&cwd)?.add_directory(label);
                            } else {
                                let size = stat.parse()?;
                                root.get_mut(&cwd)?.add_file(label, size);
                            }
                        } else {
                            anyhow::bail!("invalid output line {:?}", line);
                        }
                    }
                    None => anyhow::bail!("output without a running command"),
                }
            }
        }
        Ok(Self { root })
    }

    fn cache_directory_sizes(&mut self) {
        self.root.cache_directory_sizes();
    }

    fn walk<F, T>(&self, mut f: F)
    where
        F: FnMut(Path, &Directory) -> T,
    {
        let path = Path::new();
        self.root.walk(path, &mut f);
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let mut fs = Filesystem::parse(handle)?;
    let mut total_size = 0;
    fs.cache_directory_sizes();
    match args.mode {
        Mode::Part1 => {
            fs.walk(|_path, dir| {
                if dir.size < 100000 {
                    total_size += dir.size;
                }
            });
            println!("total_size = {}", total_size);
        }
        Mode::Part2 => {
            let mut best_candidate = None;
            let root_size = fs.root.size;
            if root_size > 70000000 {
                anyhow::bail!("FS is too big!");
            }
            let free = 70000000 - root_size;
            if free > 30000000 {
                anyhow::bail!("FS already has 30000000B free");
            }
            let needed = 30000000 - free;
            fs.walk(|path, dir| {
                if dir.size > needed {
                    match best_candidate {
                        None => best_candidate = Some((path, dir.size)),
                        Some((_, c)) if c > dir.size => best_candidate = Some((path, dir.size)),
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

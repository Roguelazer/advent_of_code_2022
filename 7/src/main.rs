use std::io::BufRead;

use clap::{Parser, ValueEnum};

mod fs;

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

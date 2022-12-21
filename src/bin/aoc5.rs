use std::io::BufRead;

use clap::{Parser, ValueEnum};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Crate(char);

impl Crate {
    fn new(ch: char) -> Self {
        Self(ch)
    }
}

type Stack = Vec<Crate>;

#[derive(Debug)]
struct Command {
    num_crates: u32,
    source_stack: usize,
    dest_stack: usize,
}

#[derive(Debug)]
struct Scene {
    stacks: Vec<Stack>,
    commands: Vec<Command>,
}

impl Scene {
    fn parse<I: Iterator<Item = String>>(lines: I) -> anyhow::Result<Self> {
        let mut stacks: Vec<Stack> = Vec::new();
        let mut commands = Vec::new();
        for line in lines {
            if line.trim_start().starts_with('[') {
                for (index, _) in line.match_indices('[') {
                    let column = index / 4;
                    if column >= stacks.len() {
                        for _ in stacks.len()..=column {
                            stacks.push(Stack::new());
                        }
                    }
                    let item = line
                        .chars()
                        .nth(index + 1)
                        .map(Crate::new)
                        .ok_or_else(|| anyhow::anyhow!("invalid column definition"))?;
                    stacks[column].push(item);
                }
            } else if line.starts_with("move") {
                let mut parts = line.split(' ');
                let num_crates = parts
                    .nth(1)
                    .ok_or_else(|| anyhow::anyhow!("invalid move line {}", line))
                    .and_then(|p| p.parse().map_err(anyhow::Error::new))?;
                let source_stack = parts
                    .nth(1)
                    .ok_or_else(|| anyhow::anyhow!("invalid move line {}", line))
                    .and_then(|p| p.parse::<usize>().map_err(anyhow::Error::new))?;
                let dest_stack = parts
                    .nth(1)
                    .ok_or_else(|| anyhow::anyhow!("invalid move line {}", line))
                    .and_then(|p| p.parse::<usize>().map_err(anyhow::Error::new))?;
                commands.push(Command {
                    num_crates,
                    source_stack,
                    dest_stack,
                });
            }
        }
        for stack in stacks.iter_mut() {
            stack.reverse();
        }
        Ok(Self { stacks, commands })
    }

    fn move_between(
        &mut self,
        count: usize,
        source_stack: usize,
        dest_stack: usize,
    ) -> anyhow::Result<()> {
        let source_stack = self
            .stacks
            .get_mut(source_stack)
            .ok_or_else(|| anyhow::anyhow!("invalid command source stack"))?;
        let mut items = source_stack.split_off(source_stack.len() - count);
        let dest_stack = self
            .stacks
            .get_mut(dest_stack)
            .ok_or_else(|| anyhow::anyhow!("invalid command dest stack"))?;
        dest_stack.append(&mut items);
        Ok(())
    }

    fn run(&mut self, mode: Mode) -> anyhow::Result<()> {
        let mut commands = Vec::new();
        std::mem::swap(&mut self.commands, &mut commands);
        for command in commands.iter() {
            if mode == Mode::Part1 {
                for _ in 0..command.num_crates {
                    self.move_between(1, command.source_stack - 1, command.dest_stack - 1)?;
                }
            } else {
                self.move_between(
                    command.num_crates as usize,
                    command.source_stack - 1,
                    command.dest_stack - 1,
                )?;
            }
        }
        Ok(())
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

fn main() {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let mut scene = Scene::parse(handle.lines().filter_map(Result::ok)).unwrap();
    scene.run(args.mode).unwrap();
    let empty_crate = Crate::new(' ');
    let output: String = scene
        .stacks
        .iter()
        .map(|s| s.last().cloned().unwrap_or(empty_crate))
        .map(|c| c.0)
        .collect();
    println!("{}", output);
}

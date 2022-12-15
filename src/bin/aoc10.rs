use std::collections::{BTreeMap, VecDeque};
use std::io::BufRead;
use std::str::FromStr;

use clap::{Parser, ValueEnum};

type Clock = u32;

#[derive(ValueEnum, Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Part1,
    Part2,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_enum)]
    mode: Mode,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Register {
    X,
}

#[derive(Debug)]
enum Op {
    Noop,
    Add(Register, i32),
}

impl FromStr for Op {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut words = s.split(' ');
        let command = words
            .next()
            .ok_or_else(|| anyhow::anyhow!("invalid command {:?}", s))?;
        Ok(match command {
            "noop" => Op::Noop,
            "addx" => {
                let value = words
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or_else(|| anyhow::anyhow!("invalid arg to addx"))?;
                Op::Add(Register::X, value)
            }
            other => anyhow::bail!("invalid command {}", other),
        })
    }
}

impl Op {
    fn cycles(&self) -> Clock {
        match self {
            Self::Noop => 1,
            Self::Add(_, _) => 2,
        }
    }
}

#[derive(Debug)]
struct RunningInstruction {
    op: Op,
    until: Clock,
}

#[derive(Debug)]
struct Cpu {
    clock: Clock,
    running: Option<RunningInstruction>,
    registers: BTreeMap<Register, i32>,
}

impl Cpu {
    fn new() -> Self {
        let mut registers = BTreeMap::new();
        registers.insert(Register::X, 1);
        Self {
            clock: 0,
            running: None,
            registers,
        }
    }

    fn is_ready(&self) -> bool {
        self.running
            .as_ref()
            .map(|r| self.clock >= r.until)
            .unwrap_or(true)
    }

    fn regval(&self, r: Register) -> i32 {
        *self.registers.get(&r).unwrap()
    }

    fn start(&mut self, command: Op) {
        self.running = Some(RunningInstruction {
            until: self.clock + command.cycles(),
            op: command,
        });
    }

    fn tick(&mut self) {
        self.clock += 1;
        // retire any running instruction
        if self.is_ready() {
            if let Some(command) = self.running.take() {
                match command.op {
                    Op::Add(reg, val) => *self.registers.get_mut(&reg).unwrap() += val,
                    Op::Noop => {}
                }
            }
        }
    }
}

#[derive(Debug)]
struct CrtDisplay {
    framebuffer: Vec<Vec<bool>>,
    width: u16,
    height: u16,
    current_x: u16,
    current_y: u16,
}

impl CrtDisplay {
    fn new(width: u16, height: u16) -> Self {
        CrtDisplay {
            width,
            height,
            framebuffer: vec![vec![false; width.into()]; height.into()],
            current_x: 0,
            current_y: 0,
        }
    }

    fn tick(&mut self, sprite_x: i32) {
        let y = self.current_y as usize;
        let x = self.current_x as usize;
        if ((self.current_x as i32) - sprite_x).abs() <= 1 {
            self.framebuffer[y][x] = true;
        } else {
            self.framebuffer[y][x] = false;
        }
        if self.current_x == self.width - 1 {
            self.current_x = 0;
            self.current_y = (self.current_y + 1) % self.height;
        } else {
            self.current_x += 1;
        }
    }

    fn draw(&self) {
        for row in &self.framebuffer {
            println!(
                "{}",
                row.iter()
                    .map(|col| if *col { "#" } else { " " })
                    .collect::<String>()
            );
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin_r = std::io::stdin();
    let stdin = stdin_r.lock();
    let mut cpu = Cpu::new();
    let mut display = CrtDisplay::new(40, 6);
    let mut commands = stdin
        .lines()
        .map(|line| line?.parse())
        .collect::<anyhow::Result<VecDeque<_>>>()?;
    let mut next_sample = 20;
    let mut samples = vec![];
    while !commands.is_empty() {
        if cpu.clock == next_sample {
            let xval = cpu.regval(Register::X);
            samples.push(xval * (next_sample as i32));
            next_sample += 40;
        }
        cpu.tick();
        let sprite_x = cpu.regval(Register::X);
        display.tick(sprite_x);
        if cpu.is_ready() {
            let command = commands.pop_front().unwrap();
            cpu.start(command);
        }
    }
    if args.mode == Mode::Part1 {
        println!("{}", samples.into_iter().sum::<i32>());
    } else {
        display.draw();
    }
    Ok(())
}

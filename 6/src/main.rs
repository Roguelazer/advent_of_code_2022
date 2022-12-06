use std::io::Read;

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

fn is_unique_bytes(s: &[u8]) -> bool {
    let mut set = bit_set::BitSet::with_capacity(256);
    for c in s {
        if !set.insert(*c as usize) {
            return false;
        }
    }
    true
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let sequence_size = match args.mode {
        Mode::Part1 => 4,
        Mode::Part2 => 14,
    };
    let mut input = Vec::new();
    handle.read_to_end(&mut input)?;
    let res = input
        .as_slice()
        .windows(sequence_size)
        .enumerate()
        .find_map(|(i, window)| {
            if is_unique_bytes(window) {
                Some(i + sequence_size)
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("no marker found!"))?;
    println!("{:?}", res);
    Ok(())
}

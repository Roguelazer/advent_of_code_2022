use std::io::BufRead;
use std::ops::RangeInclusive;
use std::str::FromStr;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Assignment(RangeInclusive<i32>);

impl FromStr for Assignment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (lhs, rhs) = s.split_once('-').ok_or_else(|| anyhow::anyhow!("no -"))?;
        Ok(Self(lhs.parse()?..=rhs.parse()?))
    }
}

impl From<RangeInclusive<i32>> for Assignment {
    fn from(r: RangeInclusive<i32>) -> Self {
        Self(r)
    }
}

impl Assignment {
    fn fully_contains(&self, other: &Self) -> bool {
        self.0.contains(other.0.start()) && self.0.contains(other.0.end())
    }

    fn overlaps(&self, other: &Self) -> bool {
        if self.0.contains(other.0.start()) || self.0.contains(other.0.end()) {
            true
        } else if other.0.start() <= self.0.start() && other.0.end() >= self.0.start() {
            true
        } else {
            false
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

fn main() {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let rv = handle
        .lines()
        .filter_map(Result::ok)
        .map(|line| {
            let (first, second) = line.split_once(',').unwrap();
            let first = first.parse::<Assignment>().unwrap();
            let second = second.parse::<Assignment>().unwrap();
            (first, second)
        })
        .filter(|(first, second)| match args.mode {
            Mode::Part1 => first.fully_contains(second) || second.fully_contains(first),
            Mode::Part2 => first.overlaps(second),
        })
        .count();
    println!("{}", rv);
}

#[cfg(test)]
mod tests {
    use super::Assignment;

    #[test]
    fn test_assignment_overlaps() {
        assert!(Assignment::from(0..=5).overlaps(&Assignment::from(5..=10)));
        assert!(Assignment::from(10..=10).overlaps(&Assignment::from(0..=20)));
        assert!(Assignment::from(10..=10).overlaps(&Assignment::from(0..=10)));
    }
}

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

fn is_unique_bytes(s: &[u8], set: &mut bit_set::BitSet) -> bool {
    for c in s {
        if !set.insert(*c as usize) {
            return false;
        }
    }
    true
}

struct RingyBuf<T: Default + Copy, const N: usize> {
    storage: [T; N],
    full: bool,
    i: usize,
}

impl<T: Default + Copy, const N: usize> RingyBuf<T, N> {
    fn new() -> Self {
        Self {
            storage: [T::default(); N],
            full: false,
            i: 0,
        }
    }

    fn push(&mut self, value: T) {
        self.storage[self.i] = value;
        self.i = (self.i + 1) % N;
        if self.i == 0 {
            self.full = true;
        }
    }

    fn is_full(&self) -> bool {
        self.full
    }

    fn bytes(&self) -> &[T] {
        if self.full {
            &self.storage
        } else {
            &self.storage[0..self.i]
        }
    }
}

fn run<const N: usize, R: Read>(io: R) -> Option<usize> {
    let mut r = RingyBuf::<u8, N>::new();
    let mut s = bit_set::BitSet::with_capacity(256);
    for (i, b) in io.bytes().filter_map(Result::ok).enumerate() {
        r.push(b);
        if r.is_full() {
            s.clear();
            if is_unique_bytes(r.bytes(), &mut s) {
                return Some(i + 1);
            }
        }
    }
    None
}

fn main() {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    if let Some(found) = match args.mode {
        Mode::Part1 => run::<4, _>(handle),
        Mode::Part2 => run::<14, _>(handle),
    } {
        println!("{:?}", found);
    }
}

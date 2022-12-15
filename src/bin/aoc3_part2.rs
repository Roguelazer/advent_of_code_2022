use std::collections::HashSet;
use std::io::BufRead;

use itertools::Itertools;

trait Priority {
    fn priority(&self) -> u32;
}

impl Priority for char {
    fn priority(&self) -> u32 {
        if ('a'..='z').contains(self) {
            (*self as u32) - ('a' as u32) + 1
        } else if ('A'..='Z').contains(self) {
            (*self as u32) - ('A' as u32) + 27
        } else {
            panic!("what is {:?}", self);
        }
    }
}

fn find_duplicate(elves: &[&str]) -> char {
    let mut elves_iter = elves.iter().map(|e| e.chars().collect::<HashSet<_>>());
    let first = elves_iter.next().unwrap();
    let intersection = elves_iter.fold(first, |a, b| {
        a.intersection(&b).cloned().collect::<HashSet<char>>()
    });
    *(intersection.iter().next().unwrap())
}

fn main() {
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let res: u32 = handle
        .lines()
        .filter_map(Result::ok)
        .tuples()
        .map(|(elf1, elf2, elf3)| {
            let duplicated = find_duplicate(&[&elf1, &elf2, &elf3]);
            duplicated.priority()
        })
        .sum();
    println!("{}", res);
}

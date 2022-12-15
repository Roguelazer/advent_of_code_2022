use std::collections::HashSet;
use std::io::BufRead;

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

fn find_duplicate(lhs: &str, rhs: &str) -> char {
    let lhs_chars = lhs.chars().collect::<HashSet<_>>();
    let rhs_chars = rhs.chars().collect::<HashSet<_>>();
    let mut intersection = lhs_chars.intersection(&rhs_chars);
    *(intersection.next().unwrap())
}

fn main() {
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let res: u32 = handle
        .lines()
        .filter_map(Result::ok)
        .map(|line| {
            let midpoint = line.len() / 2;
            let (cpt1, cpt2) = line.split_at(midpoint);
            let duplicated = find_duplicate(cpt1, cpt2);
            duplicated.priority()
        })
        .sum();
    println!("{}", res);
}

use std::io::BufRead;

use derive_more::Display;

#[derive(Debug, Display, PartialEq, Eq, Clone, Copy)]
enum Outcome {
    Win,
    Tie,
    Loss,
}

impl Outcome {
    fn score(&self) -> u32 {
        match self {
            Outcome::Win => 6,
            Outcome::Tie => 3,
            Outcome::Loss => 0,
        }
    }
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Copy)]
enum Rps {
    Rock,
    Paper,
    Scissors,
}

impl Rps {
    fn from_they_play(c: char) -> Self {
        match c {
            'A' => Rps::Rock,
            'B' => Rps::Paper,
            'C' => Rps::Scissors,
            other => panic!("unexpected input {:?}", other),
        }
    }

    fn from_you_play(c: char) -> Self {
        match c {
            'X' => Rps::Rock,
            'Y' => Rps::Paper,
            'Z' => Rps::Scissors,
            other => panic!("unexpected you-play input {:?}", other),
        }
    }

    fn score(&self) -> u32 {
        match self {
            Rps::Rock => 1,
            Rps::Paper => 2,
            Rps::Scissors => 3,
        }
    }

    fn beats(&self) -> Rps {
        match self {
            Rps::Rock => Rps::Scissors,
            Rps::Paper => Rps::Rock,
            Rps::Scissors => Rps::Paper,
        }
    }

    fn play(&self, other: &Rps) -> Outcome {
        if self == other {
            Outcome::Tie
        } else if self.beats() == *other {
            Outcome::Win
        } else {
            Outcome::Loss
        }
    }
}

fn score_round(you_play: Rps, they_play: Rps) -> u32 {
    let outcome = you_play.play(&they_play);
    you_play.score() + outcome.score()
}

fn main() {
    let stdin = std::io::stdin();
    let handle = stdin.lock();
    let total_score: u32 = handle
        .lines()
        .filter_map(|l| l.ok())
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let mut chars = line.chars();
            let they_play = Rps::from_they_play(chars.next().unwrap());
            let you_play = Rps::from_you_play(chars.nth(1).unwrap());
            let score = score_round(you_play, they_play);
            Some(score)
        })
        .sum();
    println!("{}", total_score);
}

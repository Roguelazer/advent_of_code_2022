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

    fn from_you_should(c: char) -> Self {
        match c {
            'X' => Outcome::Loss,
            'Y' => Outcome::Tie,
            'Z' => Outcome::Win,
            other => panic!("unhandled input {}", other),
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

    fn is_beaten_by(&self) -> Self {
        match self {
            Rps::Rock => Rps::Paper,
            Rps::Paper => Rps::Scissors,
            Rps::Scissors => Rps::Rock,
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

fn score_round(they_play: Rps, you_should: Outcome) -> u32 {
    let you_play = match you_should {
        Outcome::Win => they_play.is_beaten_by(),
        Outcome::Tie => they_play,
        Outcome::Loss => they_play.beats(),
    };
    let score = you_play.score() + you_should.score();
    println!(
        "they play {}, you play {}, outcome: {}; score: {}",
        they_play, you_play, you_should, score
    );
    debug_assert!(you_play.play(&they_play) == you_should);
    score
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
            let you_should = Outcome::from_you_should(chars.nth(1).unwrap());
            let score = score_round(they_play, you_should);
            Some(score)
        })
        .sum();
    println!("{}", total_score);
}

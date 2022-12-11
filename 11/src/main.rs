use std::str::FromStr;

use clap::{Parser, ValueEnum};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;

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
    #[clap(short, long, value_parser)]
    rounds: Option<usize>,
    #[clap(short, long, value_parser)]
    verbose: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum Operand {
    Old,
    Literal(i64),
}

impl Operand {
    fn value(&self, item: i64) -> i64 {
        match self {
            Self::Old => item,
            Self::Literal(i) => *i,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Op {
    Add,
    Multiply,
}

impl Op {
    fn apply(&self, item: i64, operand: &Operand, modulus: i64) -> i64 {
        let lhs = operand.value(item) % modulus;
        let rhs = item % modulus;
        match self {
            Op::Add => lhs.checked_add(rhs).unwrap() % modulus,
            Op::Multiply => lhs.checked_mul(rhs).unwrap() % modulus,
        }
    }
}

impl FromStr for Operand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "old" {
            Ok(Operand::Old)
        } else {
            Ok(Operand::Literal(s.parse()?))
        }
    }
}

#[derive(Debug)]
struct Test {
    modulus: i64,
    true_target: usize,
    false_target: usize,
}

#[derive(Debug)]
struct Action {
    item: i64,
    target: usize,
}

#[derive(Debug)]
struct Monkey {
    id: usize,
    inspections: usize,
    items: Vec<i64>,
    operation: Op,
    operand: Operand,
    test: Test,
}

impl Monkey {
    fn simulate(&mut self, common_modulus: i64, div_level: bool) -> Vec<Action> {
        self.items
            .drain(0..)
            .map(|item| {
                self.inspections += 1;
                let mut new_cost = self.operation.apply(item, &self.operand, common_modulus);
                if div_level {
                    new_cost /= 3;
                }
                if new_cost % self.test.modulus == 0 {
                    Action {
                        item: new_cost,
                        target: self.test.true_target,
                    }
                } else {
                    Action {
                        item: new_cost,
                        target: self.test.false_target,
                    }
                }
            })
            .collect()
    }
}

static MONKEY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?xm)
        ^ Monkey \s (?P<monkey_id>\d+): \n
        ^ \s+ Starting \s  items: \s (?P<items>(?: \d+ ,\s )* \d+) \n
        ^ \s+ Operation: \s+ new \s+ = \s+ old \s+ (?P<op> [+*])\s+(?P<operand> \S+ ) \n
        ^ \s+ Test: \s+ divisible\ by\  (?P<modulus>\d+) \n
        ^ \s+ If\ true:\ throw\ to\ monkey\  (?P<true_target>\d+) \n
        ^ \s+ If\ false:\ throw\ to\ monkey\  (?P<false_target>\d+)
    "#,
    )
    .unwrap()
});

fn parse_monkey<'a>(s: &'a str) -> anyhow::Result<Monkey> {
    let c = MONKEY_RE
        .captures(s)
        .ok_or_else(|| anyhow::anyhow!("Invalid monke {}", s))?;
    let monkey_id = c.name("monkey_id").unwrap().as_str().parse()?;
    let items = c
        .name("items")
        .unwrap()
        .as_str()
        .split(',')
        .map(|i| i.trim().parse::<i64>())
        .collect::<Result<Vec<i64>, _>>()?;
    let op = match c.name("op").unwrap().as_str() {
        "+" => Op::Add,
        "*" => Op::Multiply,
        other => anyhow::bail!("Unhandled operation {}", other),
    };
    let operand = c.name("operand").unwrap().as_str().parse()?;
    let modulus = c.name("modulus").unwrap().as_str().parse()?;
    let true_target = c.name("true_target").unwrap().as_str().parse()?;
    let false_target = c.name("false_target").unwrap().as_str().parse()?;
    Ok(Monkey {
        id: monkey_id,
        inspections: 0,
        items,
        operation: op,
        operand,
        test: Test {
            modulus,
            true_target,
            false_target,
        },
    })
}

fn parse_monkeys(s: &str) -> anyhow::Result<Vec<Monkey>> {
    s.split("\n\n")
        .map(|monkey| parse_monkey(monkey))
        .collect::<anyhow::Result<Vec<Monkey>>>()
}

fn simulate_round(monkeys: &mut Vec<Monkey>, common_modulus: i64, div_level: bool) {
    for index in 0..monkeys.len() {
        let actions = monkeys
            .get_mut(index)
            .unwrap()
            .simulate(common_modulus, div_level);
        for action in actions {
            monkeys
                .get_mut(action.target)
                .unwrap()
                .items
                .push(action.item);
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin_r = std::io::stdin();
    let input = std::io::read_to_string(stdin_r)?;
    let mut monkeys = parse_monkeys(input.as_str())?;
    let common_modulus = monkeys.iter().fold(1, |a, m| a * m.test.modulus);
    let rounds = match args.rounds {
        Some(r) => r,
        None => match args.mode {
            Mode::Part1 => 20,
            Mode::Part2 => 10000,
        },
    };
    for round in 0..rounds {
        simulate_round(&mut monkeys, common_modulus, args.mode == Mode::Part1);
        if args.verbose {
            println!("== After round {} ==", round);
            for monkey in monkeys.iter() {
                println!(
                    "Monkey {} inspected items {} times",
                    monkey.id, monkey.inspections
                );
            }
        }
    }
    let too_much = monkeys
        .iter()
        .map(|m| m.inspections)
        .sorted()
        .rev()
        .take(2)
        .fold(1, |a, b| a * b);
    println!("{}", too_much);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_monkey, Op, Operand};

    #[test]
    fn test_parse_monkey() {
        let res = parse_monkey(
            r#"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3
"#,
        );
        let monkey = res.unwrap();
        assert_eq!(monkey.id, 0);
        assert_eq!(monkey.items, vec![79, 98]);
        assert_eq!(monkey.operation, Op::Multiply);
        assert_eq!(monkey.operand, Operand::Literal(19));
        assert_eq!(monkey.test.modulus, 23);
        assert_eq!(monkey.test.true_target, 2);
        assert_eq!(monkey.test.false_target, 3);
    }
}

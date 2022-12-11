use std::str::FromStr;

use clap::{Parser, ValueEnum};
use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, newline, one_of, space1};
use nom::combinator::{map, map_res};
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, preceded, separated_pair};
use nom::IResult;

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

fn parse_monkey<'a>(s: &'a str) -> IResult<&'a str, Monkey> {
    let (s, monkey_id) = delimited(
        tag("Monkey "),
        map(nom::character::complete::u32, |r: u32| r as usize),
        tag(":\n"),
    )(s)?;
    let (s, items) = delimited(
        preceded(space1, tag("Starting items: ")),
        separated_list1(tag(", "), nom::character::complete::i64),
        newline,
    )(s)?;
    let (s, (op, operand)) = delimited(
        pair(space1, tag("Operation: new = old ")),
        separated_pair(
            map_res(one_of("+*"), |s: char| {
                Ok(match s {
                    '+' => Op::Add,
                    '*' => Op::Multiply,
                    other => anyhow::bail!("invalid operation {}", other),
                })
            }),
            tag(" "),
            map_res(alt((tag("old"), digit1)), |s: &str| s.parse::<Operand>()),
        ),
        newline,
    )(s)?;
    let (s, modulus) = delimited(
        pair(space1, tag("Test: divisible by ")),
        nom::character::complete::i64,
        newline,
    )(s)?;
    let (s, true_target) = delimited(
        pair(space1, tag("If true: throw to monkey ")),
        map(nom::character::complete::u32, |r: u32| r as usize),
        newline,
    )(s)?;
    let (s, false_target) = delimited(
        pair(space1, tag("If false: throw to monkey ")),
        map(nom::character::complete::u32, |r: u32| r as usize),
        newline,
    )(s)?;
    let monkey = Monkey {
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
    };
    Ok((s, monkey))
}

fn parse_monkeys(s: &str) -> anyhow::Result<Vec<Monkey>> {
    let (remaining, monkeys) = separated_list1(tag("\n"), parse_monkey)(s)
        .map_err(|e| anyhow::anyhow!("Parsing error: {:?}", e))?;
    if remaining.len() > 0 {
        anyhow::bail!("unconsumed input {:?}", remaining);
    }
    Ok(monkeys)
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
        let (remaining, monkey) = res.unwrap();
        assert_eq!(remaining.len(), 0);
        assert_eq!(monkey.id, 0);
        assert_eq!(monkey.items, vec![79, 98]);
        assert_eq!(monkey.operation, Op::Multiply);
        assert_eq!(monkey.operand, Operand::Literal(19));
        assert_eq!(monkey.test.modulus, 23);
        assert_eq!(monkey.test.true_target, 2);
        assert_eq!(monkey.test.false_target, 3);
    }
}

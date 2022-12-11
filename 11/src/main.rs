use std::io::BufRead;
use std::str::FromStr;

use clap::{Parser, ValueEnum};
use itertools::Itertools;

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
    fn apply(&self, item: i64, operand: &Operand) -> i64 {
        match self {
            Op::Add => operand.value(item) + item,
            Op::Multiply => operand.value(item).wrapping_mul(item),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum TestOp {
    Divisible,
}

impl TestOp {
    fn matches(&self, item: i64, operand: &Operand) -> bool {
        match self {
            Self::Divisible => {
                let value = operand.value(item);
                item % value == 0
            }
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
    operation: TestOp,
    operand: Operand,
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
    #[allow(dead_code)]
    id: usize,
    inspections: usize,
    items: Vec<i64>,
    operation: Op,
    operand: Operand,
    test: Test,
}

impl Monkey {
    fn simulate(&mut self, div_level: bool) -> Vec<Action> {
        self.items
            .drain(0..)
            .map(|item| {
                self.inspections += 1;
                println!("applying {:?} to {}", self.operation, item);
                let mut new_cost = self.operation.apply(item, &self.operand);
                if div_level {
                    new_cost /= 3;
                }
                if self.test.operation.matches(new_cost, &self.test.operand) {
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

fn parse_monkeys<R: BufRead>(io: &mut R) -> anyhow::Result<Vec<Monkey>> {
    let mut monkeys = vec![];
    for mut monkey_chunk in &io.lines().filter_map(Result::ok).chunks(7) {
        let monkey_id: usize = monkey_chunk
            .next()
            .unwrap()
            .trim_end_matches(':')
            .split(' ')
            .nth(1)
            .unwrap()
            .parse()?;
        let items = monkey_chunk
            .next()
            .unwrap()
            .split(':')
            .nth(1)
            .unwrap()
            .split(',')
            .map(|s| {
                s.trim()
                    .to_owned()
                    .parse::<i64>()
                    .map_err(|e| anyhow::Error::new(e))
            })
            .collect::<anyhow::Result<Vec<i64>>>()?;
        let op_s = monkey_chunk
            .next()
            .unwrap()
            .split("= old ")
            .nth(1)
            .unwrap()
            .to_owned();
        let (op, operand) = {
            let op = match &op_s[0..1] {
                "+" => Op::Add,
                "*" => Op::Multiply,
                o => anyhow::bail!("invalid op {}", o),
            };
            let val = Operand::from_str(&op_s[2..])?;
            (op, val)
        };
        let test_s = monkey_chunk.next().unwrap();
        let test_s = test_s.split(':').nth(1);
        let (test_op, test_operand) =
            if let Some(rest) = test_s.and_then(|s| s.strip_prefix(" divisible by ")) {
                (TestOp::Divisible, rest.parse::<Operand>()?)
            } else {
                anyhow::bail!("unknown operation {:?}", test_s)
            };
        let true_value = monkey_chunk.next().unwrap();
        let true_target = if let Some((_, id)) = true_value
            .trim()
            .strip_prefix("If true: ")
            .and_then(|r| r.rsplit_once(' '))
        {
            id.parse::<usize>()?
        } else {
            anyhow::bail!("unknown action {:?}", true_value);
        };
        let false_value = monkey_chunk.next().unwrap();
        let false_target = if let Some((_, id)) = false_value
            .trim()
            .strip_prefix("If false: ")
            .and_then(|r| r.rsplit_once(' '))
        {
            id.parse::<usize>()?
        } else {
            anyhow::bail!("unknown action {:?}", true_value);
        };
        let test = Test {
            operation: test_op,
            operand: test_operand,
            true_target,
            false_target,
        };
        monkeys.push(Monkey {
            id: monkey_id,
            inspections: 0,
            items,
            operation: op,
            operand,
            test,
        });
    }
    Ok(monkeys)
}

fn simulate_round(monkeys: &mut Vec<Monkey>, div_level: bool) {
    for index in 0..monkeys.len() {
        let actions = monkeys.get_mut(index).unwrap().simulate(div_level);
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
    let mut stdin = stdin_r.lock();
    let mut monkeys = parse_monkeys(&mut stdin)?;
    let rounds = match args.mode {
        Mode::Part1 => 20,
        Mode::Part2 => 10000,
    };
    for _ in 0..rounds {
        simulate_round(&mut monkeys, args.mode == Mode::Part1);
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

use clap::{Parser, ValueEnum};
use std::io::BufRead;
use std::mem;

#[derive(ValueEnum, Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Part1,
    Part2,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    verbose: bool,
    #[clap(short, long, value_enum)]
    mode: Mode,
}

#[derive(Debug, Copy, Clone)]
struct Item {
    value: i32,
    original_index: usize,
}

#[derive(Debug)]
enum Direction {
    Left,
    Right,
}

fn mix(input: &Vec<i32>) -> Vec<i32> {
    let mut output = input
        .iter()
        .enumerate()
        .map(|(i, v)| Item {
            value: *v,
            original_index: i,
        })
        .collect::<Vec<Item>>();
    let len = input.len();
    log::debug!("{:?}", input);
    for index in 0..input.len() {
        log::debug!("examining index {:?}", index);
        let position = output
            .iter()
            .position(|p| p.original_index == index)
            .unwrap();
        let value = output[position];
        let count = value.value.abs();
        let direction = match value.value.signum() {
            0 => continue,
            -1 => Direction::Left,
            1 => Direction::Right,
            _ => unreachable!(),
        };
        let mut index = position;
        for _ in 0..count {
            match direction {
                Direction::Left => {
                    let next_index = (index as i32 - 1).rem_euclid(len as i32) as usize;
                    output.swap(next_index, index);
                    index = next_index;
                }
                Direction::Right => {
                    let next_index = (index as i32 + 1).rem_euclid(len as i32) as usize;
                    output.swap(next_index, index);
                    index = next_index;
                }
            }
        }
        log::debug!("{:?}\n", output.iter().map(|m| m.value).collect::<Vec<_>>());
    }
    output.into_iter().map(|m| m.value).collect()
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let log_level = if args.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };
    env_logger::builder()
        .format_module_path(false)
        .format_timestamp_millis()
        .filter_level(log_level)
        .init();
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let numbers = input
        .lines()
        .map(|l| l.parse::<i32>())
        .collect::<Result<Vec<_>, _>>()?;
    let result = mix(&numbers);
    if args.mode == Mode::Part1 {
        let zero_index = result.iter().position(|i| *i == 0).unwrap();
        let sum: i32 = [1000, 2000, 3000]
            .into_iter()
            .map(|i| {
                let index = (zero_index + i) % result.len();
                result[index]
            })
            .sum();
        println!("{}", sum);
    }
    println!("{:?}", numbers);
    Ok(())
}

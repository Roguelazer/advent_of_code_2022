use clap::{Parser, ValueEnum};

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
    value: i64,
    original_index: usize,
}

fn mix(input: &Vec<i64>, decryption_key: i64, rounds: usize) -> Vec<i64> {
    let mut output = input
        .iter()
        .enumerate()
        .map(|(i, v)| Item {
            value: *v * decryption_key,
            original_index: i,
        })
        .collect::<Vec<Item>>();
    let len = input.len();
    for _ in 0..rounds {
        log::debug!("{:?}", input);
        for index in 0..input.len() {
            let position = output
                .iter()
                .position(|p| p.original_index == index)
                .unwrap();
            let v = output[position].value;
            log::debug!("examining original index {}, value {}", index, v);
            let new_position = (position as i64 + v).rem_euclid(len as i64 - 1) as usize;
            let value = output.remove(position);
            output.insert(new_position, value);
            log::debug!("{} -> {} (by {})", position, new_position, v);
            log::debug!("{:?}\n", output.iter().map(|m| m.value).collect::<Vec<_>>());
        }
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
        .map(|l| l.parse::<i64>())
        .collect::<Result<Vec<_>, _>>()?;
    let result = match args.mode {
        Mode::Part1 => mix(&numbers, 1, 1),
        Mode::Part2 => mix(&numbers, 811589153, 10),
    };
    let zero_index = result.iter().position(|i| *i == 0).unwrap();
    let sum: i64 = [1000, 2000, 3000]
        .into_iter()
        .map(|i| {
            let index = (zero_index + i) % result.len();
            result[index]
        })
        .sum();
    println!("{}", sum);
    Ok(())
}

use clap::{Parser, ValueEnum};
use std::io::BufRead;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    verbose: bool,
}

fn parse_snafu(s: &str) -> i64 {
    let mut val = 0;
    let mut mul = 1;
    for chr in s.chars().rev() {
        let b = match chr {
            '2' => 2,
            '1' => 1,
            '0' => 0,
            '-' => -1,
            '=' => -2,
            _ => unreachable!(),
        };
        val += b * mul;
        mul *= 5;
    }
    val
}

fn to_snafu(mut i: i64) -> String {
    let mut c = vec![];
    while i > 0 {
        let (this_place, next) = match i % 5 {
            0 => ('0', i / 5),
            1 => ('1', i / 5),
            2 => ('2', i / 5),
            3 => ('=', (i / 5) + 1),
            4 => ('-', (i / 5) + 1),
            _ => unreachable!(),
        };
        c.push(this_place);
        i = next;
    }
    c.reverse();
    c.into_iter().collect()
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
    let stdin_r = std::io::stdin();
    let stdin = stdin_r.lock();
    let lines = stdin
        .lines()
        .filter_map(Result::ok)
        .filter_map(|s| {
            let s = s.trim();
            if s.is_empty() {
                None
            } else {
                Some(parse_snafu(s))
            }
        })
        .collect::<Vec<i64>>();
    log::debug!("{:?}", lines);
    let s = lines.iter().sum::<i64>();
    log::debug!("sum: {}", s);
    println!("{}", to_snafu(s));
    Ok(())
}

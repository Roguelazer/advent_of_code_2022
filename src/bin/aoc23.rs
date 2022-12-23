use clap::{Parser, ValueEnum};
use itertools::{Itertools, MinMaxResult};
use std::collections::HashSet;

use aoclib::DenseGrid;
use aoclib::Point;

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct Elf {
    id: usize,
    position: Point,
    proposed_position: Option<Point>,
}

impl Elf {
    fn new(id: usize, position: Point) -> Self {
        Elf {
            id,
            position,
            proposed_position: None,
        }
    }
}

fn parse_positions(s: &str) -> Vec<Elf> {
    let mut i = 0;
    let mut res = vec![];
    for (y, line) in s.lines().enumerate() {
        for (x, chr) in line.bytes().enumerate() {
            let position = Point::new(x as i64, y as i64);
            if chr == b'#' {
                i += 1;
                res.push(Elf::new(i, position))
            }
        }
    }
    res
}

const NORTH: Point = Point::new(0, -1);
const WEST: Point = Point::new(-1, 0);
const EAST: Point = Point::new(1, 0);
const SOUTH: Point = Point::new(0, 1);

fn step(elves: &mut Vec<Elf>, directions: &[Point]) -> bool {
    let mut moved = false;
    let current_positions = elves.iter().map(|e| e.position).collect::<HashSet<_>>();
    // phase 1: proposal
    for elf in elves.iter_mut() {
        if [-1i64, 0, 1]
            .into_iter()
            .flat_map(|x| {
                [-1i64, 0, 1].into_iter().filter_map(move |y| {
                    if x == 0 && y == 0 {
                        None
                    } else {
                        Some(Point::new(x, y))
                    }
                })
            })
            .map(|p| elf.position + p)
            .all(|p| !current_positions.contains(&p))
        {
            log::debug!("elf {} is sitting this round out", elf.id);
            continue;
        }
        for direction in directions.iter() {
            if [-1i64, 0, 1].iter().all(|other_dim| {
                let offset = if direction.x == 0 {
                    Point::new(*other_dim, 0)
                } else {
                    Point::new(0, *other_dim)
                };
                let offset = *direction + offset;
                let check_point = elf.position + offset;
                !current_positions.contains(&check_point)
            }) {
                elf.proposed_position = Some(elf.position + *direction);
                break;
            }
        }
    }
    // phase 2: motion
    let proposed_count = elves.iter().filter_map(|e| e.proposed_position).counts();
    for elf in elves.iter_mut() {
        if let Some(proposal) = elf.proposed_position.take() {
            if proposed_count.get(&proposal) == Some(&1) {
                log::debug!("elf {} moves {} -> {}", elf.id, elf.position, proposal);
                elf.position = proposal;
                moved = true;
            } else {
                log::debug!(
                    "elf {} wants to move {} -> {} but cannot",
                    elf.id,
                    elf.position,
                    proposal
                );
            }
        }
    }
    moved
}

fn bounding_box(elves: &[Elf]) -> (Point, Point) {
    let (min_x, max_x) = match elves.iter().map(|e| e.position.x).minmax() {
        MinMaxResult::MinMax(a, b) => (a, b),
        _ => unreachable!(),
    };
    let (min_y, max_y) = match elves.iter().map(|e| e.position.y).minmax() {
        MinMaxResult::MinMax(a, b) => (a, b),
        _ => unreachable!(),
    };
    (Point::new(min_x, min_y), Point::new(max_x, max_y))
}

fn render(elves: &[Elf]) {
    let (mut min_bb, mut max_bb) = bounding_box(elves);
    min_bb.x -= 1;
    min_bb.y -= 1;
    max_bb.x += 1;
    max_bb.y += 1;
    let mut grid = DenseGrid::new_with(min_bb, max_bb, '.');
    for elf in elves {
        grid.set(elf.position, '#');
    }
    grid.dump_with(|c| *c);
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
    let mut elves = parse_positions(&input);

    let mut directions = vec![NORTH, SOUTH, WEST, EAST];

    if args.mode == Mode::Part1 {
        if args.verbose {
            println!("=== Initial condition ===");
            render(elves.as_slice());
        }
        for round in 0..10 {
            step(&mut elves, &directions);
            directions.rotate_left(1);
            if args.verbose {
                println!("=== After round {} ===", round + 1);
                render(elves.as_slice());
            }
        }

        let (min_bb, max_bb) = bounding_box(elves.as_slice());
        let width = max_bb.x.abs_diff(min_bb.x) + 1;
        let height = max_bb.y.abs_diff(min_bb.y) + 1;
        println!("{}", width * height - elves.len() as u64);
    } else {
        let start = std::time::Instant::now();
        let mut round = 1;
        loop {
            if !step(&mut elves, &directions) {
                break;
            } else {
                round += 1;
                directions.rotate_left(1);
            }
        }
        println!("{} (in {:?})", round, start.elapsed())
    }
    Ok(())
}

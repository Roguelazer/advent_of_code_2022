use clap::{Parser, ValueEnum};
use itertools::Itertools;
use std::collections::{BTreeMap, HashSet, VecDeque};

use aoclib::{DenseGrid, Point};

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
    #[clap(short, long)]
    dump_path: bool,
    #[clap(short, long, value_enum)]
    mode: Mode,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Blizzard {
    position: Point,
    direction: Point,
}

impl Blizzard {
    fn new(position: Point, direction: Point) -> Self {
        Self {
            position,
            direction,
        }
    }
}

#[derive(Debug, Clone)]
struct Map {
    width: i64,
    height: i64,
    blizzards: Vec<Blizzard>,
}

impl Map {
    fn dump(&self, pos: Point) {
        let mut g = DenseGrid::new_with(
            Point::new(0, 0),
            Point::new(self.width - 1, self.height - 1),
            b'.',
        );
        let counts = self.blizzards.iter().map(|b| b.position).counts();
        for b in &self.blizzards {
            if counts[&b.position] > 1 {
                g.set(b.position, b'N');
            } else {
                let c = if b.direction == NORTH {
                    b'^'
                } else if b.direction == EAST {
                    b'>'
                } else if b.direction == WEST {
                    b'<'
                } else if b.direction == SOUTH {
                    b'v'
                } else {
                    unreachable!();
                };
                g.set(b.position, c);
            }
        }
        if pos.y >= 0 {
            match g.get(pos) {
                Some(b'.') => {}
                None => {}
                other => panic!("{} = {:?}, expected .", pos, other),
            }
            g.set(pos, b'E');
        }
        g.dump_with(|c| *c as char);
    }

    fn step(&self) -> Map {
        let mut next = self.clone();
        next.blizzards = self
            .blizzards
            .iter()
            .map(|b| {
                let mut next_coord = b.position + b.direction;
                if next_coord.x == self.width {
                    next_coord.x = 0;
                } else if next_coord.x == -1 {
                    next_coord.x = self.width - 1;
                } else if next_coord.y == self.height {
                    next_coord.y = 0;
                } else if next_coord.y == -1 {
                    next_coord.y = self.height - 1;
                }
                Blizzard {
                    position: next_coord,
                    direction: b.direction,
                }
            })
            .collect();
        next
    }

    fn can_move(&self, position: Point) -> bool {
        if position.x < 0 || position.x >= self.width || position.y < 0 || position.y >= self.height
        {
            false
        } else {
            !self.blizzards.iter().any(|b| b.position == position)
        }
    }
}

fn parse_map(s: &str) -> (Map, Point, Point) {
    let s = s.trim();
    let width = s.split('\n').next().unwrap().len() - 2;
    let height = s.split('\n').count() - 2;
    let start_x = s
        .split('\n')
        .next()
        .unwrap()
        .chars()
        .position(|p| p == '.')
        .unwrap()
        - 1;
    let end_x = s
        .split('\n')
        .last()
        .unwrap()
        .chars()
        .position(|p| p == '.')
        .unwrap()
        - 1;
    let start_coordinate = Point::new(start_x as i64, -1);
    let end_coordinate = Point::new(end_x as i64, height as i64);
    let blizzards = s
        .split('\n')
        .skip(1)
        .take(height)
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars()
                .skip(1)
                .enumerate()
                .filter_map(move |(x, chr)| {
                    let position = Point::new(x as i64, y as i64);
                    if chr == '>' {
                        Some(Blizzard::new(position, EAST))
                    } else if chr == '<' {
                        Some(Blizzard::new(position, WEST))
                    } else if chr == '^' {
                        Some(Blizzard::new(position, NORTH))
                    } else if chr == 'v' {
                        Some(Blizzard::new(position, SOUTH))
                    } else if chr == '.' || chr == '#' {
                        None
                    } else {
                        panic!("unhandled input {}", chr)
                    }
                })
        })
        .collect::<Vec<_>>();
    (
        Map {
            width: width as i64,
            height: height as i64,
            blizzards,
        },
        start_coordinate,
        end_coordinate,
    )
}

#[derive(Debug)]
struct Memo {
    maps_by_step: BTreeMap<usize, Map>,
    seen: HashSet<(Point, usize)>,
}

impl Memo {
    fn ensure_map(&mut self, timestamp: usize) {
        if self.maps_by_step.contains_key(&timestamp) {
            return;
        }
        let (max_ts, max_map) = self.maps_by_step.last_key_value().unwrap();
        let max_ts = *max_ts;
        let mut max_map = max_map.clone();
        for ts in (max_ts + 1)..=timestamp {
            max_map = max_map.step();
            self.maps_by_step.insert(ts, max_map.clone());
        }
    }

    fn dump_with_path(&mut self, path: &[(usize, Point)]) {
        for (ts, position) in path.iter() {
            self.ensure_map(*ts);
            let map = self.maps_by_step.get(&ts).unwrap();
            println!("TS={}, POS={}", ts, position);
            map.dump(*position);
            println!();
        }
    }
}

const SOUTH: Point = Point::new(0, 1);
const EAST: Point = Point::new(1, 0);
const WEST: Point = Point::new(-1, 0);
const NORTH: Point = Point::new(0, -1);

trait MaybePath: std::fmt::Debug {
    fn empty() -> Self;
    fn with(&self, ts: usize, position: Point) -> Self;
    fn dump_with(&self, _memo: &mut Memo) {}
    fn end_ts(&self) -> usize;
}

impl MaybePath for Vec<(usize, Point)> {
    fn empty() -> Self {
        vec![]
    }

    fn with(&self, ts: usize, position: Point) -> Self {
        let mut new = self.clone();
        new.push((ts, position));
        new
    }

    fn dump_with(&self, memo: &mut Memo) {
        memo.dump_with_path(self)
    }

    fn end_ts(&self) -> usize {
        self.iter().last().map(|p| p.0).unwrap_or(0)
    }
}

impl MaybePath for usize {
    fn empty() -> Self {
        0
    }

    fn with(&self, ts: usize, _position: Point) -> Self {
        ts
    }

    fn end_ts(&self) -> usize {
        *self
    }
}

fn simulate<P: MaybePath>(
    memo: &mut Memo,
    start_coordinate: Point,
    start_ts: usize,
    end_coordinate: Point,
    empty_path: &P,
) -> P {
    let mut queue = VecDeque::new();
    let mut max_ts = 0;
    queue.push_back((
        start_coordinate,
        empty_path.with(start_ts, start_coordinate),
    ));
    while let Some((position, path)) = queue.pop_front() {
        let timestamp = path.end_ts();
        log::debug!("considering {} at {}", position, timestamp);
        max_ts = std::cmp::max(max_ts, timestamp);
        if !memo.seen.insert((position, timestamp)) {
            continue;
        }
        memo.ensure_map(timestamp + 1);
        let map = memo.maps_by_step.get(&(timestamp + 1)).unwrap();
        for offset in &[SOUTH, NORTH, WEST, EAST] {
            let candidate = position + *offset;
            if candidate == end_coordinate {
                return path.with(timestamp + 1, end_coordinate);
            }
            if map.can_move(candidate) {
                queue.push_back((candidate, path.with(timestamp + 1, candidate)));
            }
        }
        if map.can_move(position) || position == start_coordinate {
            queue.push_back((position, path.with(timestamp + 1, position)));
        }
    }
    panic!("ran out of moves at {}", max_ts);
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
    let (first_map, start_coordinate, end_coordinate) = parse_map(&input);
    let mut maps_by_step = BTreeMap::new();
    maps_by_step.insert(0, first_map);
    let memo = Memo {
        maps_by_step,
        seen: HashSet::new(),
    };
    if args.verbose {
        run_rest(start_coordinate, end_coordinate, memo, vec![], args);
    } else {
        run_rest(start_coordinate, end_coordinate, memo, 0, args);
    }
    Ok(())
}

fn run_rest<P: MaybePath>(
    start_coordinate: Point,
    end_coordinate: Point,
    mut memo: Memo,
    empty_path: P,
    args: Args,
) {
    let start = std::time::Instant::now();
    let best = if args.mode == Mode::Part1 {
        let path = simulate(&mut memo, start_coordinate, 0, end_coordinate, &empty_path);
        if args.dump_path {
            path.dump_with(&mut memo);
        }
        path
    } else {
        let first = simulate(&mut memo, start_coordinate, 0, end_coordinate, &empty_path);
        if args.dump_path {
            first.dump_with(&mut memo);
        }
        let second = simulate(
            &mut memo,
            end_coordinate,
            first.end_ts(),
            start_coordinate,
            &empty_path,
        );
        if args.dump_path {
            second.dump_with(&mut memo);
        }
        let third = simulate(
            &mut memo,
            start_coordinate,
            second.end_ts(),
            end_coordinate,
            &empty_path,
        );
        if args.dump_path {
            third.dump_with(&mut memo);
        }
        third
    };
    println!("{} (in {:?})", best.end_ts(), start.elapsed());
}

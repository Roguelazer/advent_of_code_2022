use std::cmp::min;

use clap::{Parser, ValueEnum};
use itertools::{EitherOrBoth, Itertools};

const WIDTH: usize = 7;
const TALLEST_SHAPE: usize = 4;

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
    #[clap(short, long)]
    verbose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Stuck,
    Moving,
}

impl Cell {
    fn as_char(&self) -> char {
        match self {
            Cell::Empty => '.',
            Cell::Stuck => '#',
            Cell::Moving => '@',
        }
    }

    fn frozen(&self) -> Self {
        match self {
            Self::Moving => Self::Stuck,
            other => *other,
        }
    }
}

const E: Cell = Cell::Empty;
const M: Cell = Cell::Moving;

const SHAPES: &[&[[Cell; WIDTH]]] = &[
    &[[E, E, M, M, M, M, E]],
    &[
        [E, E, E, M, E, E, E],
        [E, E, M, M, M, E, E],
        [E, E, E, M, E, E, E],
    ],
    &[
        [E, E, E, E, M, E, E],
        [E, E, E, E, M, E, E],
        [E, E, M, M, M, E, E],
    ],
    &[
        [E, E, M, E, E, E, E],
        [E, E, M, E, E, E, E],
        [E, E, M, E, E, E, E],
        [E, E, M, E, E, E, E],
    ],
    &[[E, E, M, M, E, E, E], [E, E, M, M, E, E, E]],
];

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Motion {
    Left,
    Right,
    Down,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Tick {
    Lr,
    Ud,
}

impl Tick {
    fn next(&self) -> Self {
        match self {
            Tick::Lr => Tick::Ud,
            Tick::Ud => Tick::Lr,
        }
    }
}

fn move_down(current: &mut [Cell], lower: &mut [Cell]) {
    for index in 0..current.len() {
        if current[index] == Cell::Moving {
            assert_eq!(lower[index], Cell::Empty);
            lower[index] = Cell::Moving;
            current[index] = Cell::Empty;
        }
    }
}

fn move_dir(motion: Motion, row: &mut [Cell]) {
    if motion == Motion::Right {
        for index in (0..WIDTH).rev() {
            if row[index] == Cell::Moving {
                row.swap(index, index + 1)
            }
        }
    } else if motion == Motion::Left {
        for index in 0..WIDTH {
            if row[index] == Cell::Moving {
                row.swap(index, index - 1)
            }
        }
    }
}

#[derive(Debug)]
struct Scene {
    floor_offset: usize,
    rows: Vec<[Cell; WIDTH]>,
    next_shape: usize,
    next_motion: usize,
    next_tick: Tick,
    motions: Vec<Motion>,
    shape_bottom_row: Option<usize>,
    shapes_added: usize,
}

impl Scene {
    fn new(motions: Vec<Motion>) -> Self {
        Self {
            floor_offset: 0,
            next_shape: 0,
            shapes_added: 0,
            motions,
            next_motion: 0,
            next_tick: Tick::Lr,
            shape_bottom_row: None,
            rows: vec![],
        }
    }

    fn empty_row() -> [Cell; WIDTH] {
        [Cell::Empty; WIDTH]
    }

    fn add_shape(&mut self) {
        let shape = SHAPES[self.next_shape];
        self.next_shape = (self.next_shape + 1) % SHAPES.len();
        for _ in 0..3 {
            self.rows.push(Self::empty_row())
        }
        let brow = self.rows.len();
        for row in shape.iter().rev() {
            self.rows.push(*row);
        }
        self.shapes_added += 1;
        self.shape_bottom_row = Some(brow);
    }

    fn can_move_laterally(&self, motion: Motion, index: usize) -> bool {
        if index >= self.rows.len() {
            return true;
        }
        let row = self.rows[index];
        let res = match motion {
            Motion::Right => row
                .iter()
                .zip_longest(row.iter().skip(1))
                .all(|items| match items {
                    EitherOrBoth::Both(Cell::Moving, Cell::Empty) => true,
                    EitherOrBoth::Both(Cell::Moving, Cell::Moving) => true,
                    EitherOrBoth::Both(Cell::Moving, _) => false,
                    EitherOrBoth::Left(Cell::Moving) => false,
                    _ => true,
                }),
            Motion::Left => row
                .iter()
                .rev()
                .zip_longest(row.iter().rev().skip(1))
                .all(|items| match items {
                    EitherOrBoth::Both(Cell::Moving, Cell::Empty) => true,
                    EitherOrBoth::Both(Cell::Moving, Cell::Moving) => true,
                    EitherOrBoth::Both(Cell::Moving, _) => false,
                    EitherOrBoth::Left(Cell::Moving) => false,
                    _ => true,
                }),
            Motion::Down => unreachable!(),
        };
        res
    }

    fn can_move_down(&self, index: usize) -> bool {
        if index >= self.rows.len() {
            return true;
        }
        (0..WIDTH).all(|x| {
            if self.rows[index][x] == Cell::Moving {
                index > 0
                    && (self.rows[index - 1][x] == Cell::Empty
                        || self.rows[index - 1][x] == Cell::Moving)
            } else {
                true
            }
        })
    }

    fn can_move(&self, motion: Motion, bottom_row: usize) -> bool {
        if motion == Motion::Down {
            (bottom_row..bottom_row + TALLEST_SHAPE).all(|index| self.can_move_down(index))
        } else {
            (bottom_row..bottom_row + TALLEST_SHAPE)
                .all(|index| self.can_move_laterally(motion, index))
        }
    }

    fn do_move(&mut self, motion: Motion, bottom_row: usize) {
        for index in bottom_row..(bottom_row + TALLEST_SHAPE) {
            if index >= self.rows.len() {
                continue;
            }
            if motion == Motion::Down {
                let (before, after) = self.rows.split_at_mut(index);
                move_down(after.first_mut().unwrap(), before.last_mut().unwrap());
            } else {
                move_dir(motion, &mut self.rows[index])
            }
        }
    }

    fn freeze(&mut self, bottom_row: usize) {
        for index in bottom_row..min(bottom_row + TALLEST_SHAPE, self.rows.len() - 1) {
            self.rows[index] = self.rows[index].map(|c| c.frozen());
        }
    }

    fn find_highest_occupied_row(&self) -> usize {
        let mut max = 0;
        for (i, row) in self.rows.iter().enumerate() {
            if row.iter().any(|c| *c == Cell::Stuck) {
                max = i
            }
        }
        max + 1
    }

    fn trim(&mut self) {
        self.rows.truncate(self.find_highest_occupied_row());
    }

    fn tick(&mut self) {
        if let Some(bottom_row) = self.shape_bottom_row {
            let motion = match self.next_tick {
                Tick::Lr => {
                    let motion = self.motions[self.next_motion];
                    self.next_motion = (self.next_motion + 1) % self.motions.len();
                    motion
                }
                Tick::Ud => Motion::Down,
            };
            if self.can_move(motion, bottom_row) {
                self.do_move(motion, bottom_row);
                if motion == Motion::Down {
                    self.shape_bottom_row = Some(bottom_row - 1);
                }
            } else if motion == Motion::Down {
                self.freeze(bottom_row);
                self.shape_bottom_row = None;
                self.trim();
            } else {
            }
            self.next_tick = self.next_tick.next();
        } else {
            self.add_shape();
        }
    }

    fn find_last_full_row(&self) -> Option<usize> {
        let mut res = None;
        for (i, row) in self.rows.iter().enumerate() {
            if row.iter().all(|c| *c == Cell::Stuck) {
                res = Some(i);
            }
        }
        res
    }

    fn check_drop_bottom(&mut self) {
        if let Some(idx) = self.find_last_full_row() {
            let mut new_rows = self.rows.drain(idx..).collect::<Vec<_>>();
            std::mem::swap(&mut self.rows, &mut new_rows);
            if let Some(shape_bottom_row) = self.shape_bottom_row {
                self.shape_bottom_row = Some(shape_bottom_row - idx);
            }
            self.floor_offset += idx;
        }
    }

    fn draw(&self) {
        for row in self.rows.iter().rev() {
            println!("{}", row.iter().map(|r| r.as_char()).collect::<String>());
        }
        println!("-------");
        println!("and then another {} rows", self.floor_offset);
    }
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
    let motions = input
        .trim()
        .chars()
        .map(|c| match c {
            '<' => Ok(Motion::Left),
            '>' => Ok(Motion::Right),
            _ => anyhow::bail!("what is {:?}", c),
        })
        .collect::<anyhow::Result<Vec<Motion>>>()?;
    let mut scene = Scene::new(motions);
    let mut ticks = 0;
    while scene.shapes_added < 2023 {
        scene.tick();
        ticks += 1;
        if ticks % 100 == 0 {
            scene.check_drop_bottom();
        }
    }
    if args.verbose {
        scene.draw();
    }
    println!("{}", scene.find_highest_occupied_row() + scene.floor_offset);
    Ok(())
}

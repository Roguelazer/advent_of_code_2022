use clap::{Parser, ValueEnum};
use itertools::Itertools;
use nom::{
    bytes::complete::tag, character, combinator::map, multi::separated_list1,
    sequence::separated_pair, IResult,
};

use aoclib::{DenseGrid, HasEmpty, Point};

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Cell {
    Empty,
    Rock,
    Sand,
}

impl HasEmpty for Cell {
    fn empty_value() -> Self {
        Self::Empty
    }
}

impl Cell {
    fn is_empty(&self) -> bool {
        *self == Cell::Empty
    }

    fn as_char(&self) -> char {
        match self {
            Cell::Empty => '.',
            Cell::Rock => '#',
            Cell::Sand => 'o',
        }
    }
}

#[derive(Debug)]
struct Scene {
    grid: DenseGrid<Cell>,
    current_sand: Option<Point>,
    sand_created: usize,
}

impl Scene {
    fn new(top_left: Point, bottom_right: Point) -> Self {
        Scene {
            grid: DenseGrid::new(top_left, bottom_right),
            current_sand: None,
            sand_created: 0,
        }
    }

    fn add_path(&mut self, start: Point, end: Point, of: Cell) {
        for coordinate in start.line_to(end) {
            self.grid.set(coordinate, of);
        }
    }

    fn dump(&self) {
        self.grid.dump_with(|c| c.as_char())
    }

    fn step(&mut self) -> bool {
        if let Some(coordinate) = self.current_sand.take() {
            let down = coordinate + Point { x: 0, y: 1 };
            let down_left = coordinate + Point { x: -1, y: 1 };
            let down_right = coordinate + Point { x: 1, y: 1 };
            if !self.grid.contains(down)
                || !self.grid.contains(down_left)
                || !self.grid.contains(down_right)
            {
                self.sand_created -= 1;
                return false;
            }
            if self.grid[down].is_empty() {
                self.current_sand = Some(down);
            } else if self.grid[down_left].is_empty() {
                self.current_sand = Some(down_left);
            } else if self.grid[down_right].is_empty() {
                self.current_sand = Some(down_right);
            } else {
                self.grid[coordinate] = Cell::Sand;
            }
        } else if self.grid[Point { x: 500, y: 0 }].is_empty() {
            self.sand_created += 1;
            self.current_sand = Some(Point { x: 500, y: 0 });
        } else {
            return false;
        }
        true
    }

    fn simulate(&mut self) {
        while self.step() {}
    }
}

fn parse_path(s: &str) -> IResult<&str, Vec<Point>> {
    separated_list1(
        tag(" -> "),
        map(
            separated_pair(character::complete::i64, tag(","), character::complete::i64),
            |(x, y)| Point { x, y },
        ),
    )(s)
}

fn parse_scene(s: &str, mode: Mode) -> anyhow::Result<Scene> {
    let paths = s
        .split('\n')
        .filter(|l| !l.is_empty())
        .enumerate()
        .map(|(i, line)| {
            let (remaining, path) = parse_path(line)
                .map_err(|e| anyhow::anyhow!("error parsing line {}: {:?}", i + 1, e))?;
            if !remaining.trim().is_empty() {
                anyhow::bail!("unhandled input in line {}: {:?}", i + 1, remaining);
            }
            Ok(path)
        })
        .collect::<anyhow::Result<Vec<Vec<Point>>>>()?;
    let min_x = paths
        .iter()
        .filter_map(|path| path.iter().map(|coordinate| coordinate.x).min())
        .min()
        .unwrap();
    let max_x = std::cmp::max(
        paths
            .iter()
            .filter_map(|path| path.iter().map(|coordinate| coordinate.x).max())
            .max()
            .unwrap(),
        500,
    );
    let min_y = std::cmp::min(
        paths
            .iter()
            .filter_map(|path| path.iter().map(|coordinate| coordinate.y).min())
            .min()
            .unwrap(),
        0,
    );
    let max_y = paths
        .iter()
        .filter_map(|path| path.iter().map(|coordinate| coordinate.y).max())
        .max()
        .unwrap();
    let top_left = if mode == Mode::Part2 {
        Point::new(min_x - 100, min_y)
    } else {
        Point::new(min_x, min_y)
    };
    let bottom_right = if mode == Mode::Part2 {
        Point::new(max_x + 100, max_y + 2)
    } else {
        Point::new(max_x, max_y)
    };
    let mut scene = Scene::new(top_left, bottom_right);
    for path in paths.into_iter() {
        for (lhs, rhs) in path.into_iter().tuple_windows() {
            scene.add_path(lhs, rhs, Cell::Rock);
        }
    }
    if mode == Mode::Part2 {
        scene.add_path(
            Point { x: 0, y: max_y + 2 },
            Point {
                x: 1000,
                y: max_y + 2,
            },
            Cell::Rock,
        );
    }
    Ok(scene)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let mut scene = parse_scene(&input, args.mode)?;
    if args.verbose {
        println!("Before: ");
        scene.dump()
    }
    scene.simulate();
    println!("CREATED: {}", scene.sand_created);
    if args.verbose {
        println!("After: ");
        scene.dump()
    }
    Ok(())
}

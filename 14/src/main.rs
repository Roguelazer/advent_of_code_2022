use clap::{Parser, ValueEnum};
use itertools::Itertools;
use nom::{
    bytes::complete::tag, character, combinator::map, multi::separated_list1,
    sequence::separated_pair, IResult,
};

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

#[derive(Debug)]
struct LineToStruct {
    start: Point,
    end: Point,
    direction: Point,
    done: bool,
}

impl LineToStruct {
    fn new(start: Point, end: Point) -> Self {
        debug_assert!(start.x == end.x || start.y == end.y);
        let dir = if start.x == end.x {
            if start.y < end.y {
                Point { x: 0, y: 1 }
            } else {
                Point { x: 0, y: -1 }
            }
        } else {
            if start.x < end.x {
                Point { x: 1, y: 0 }
            } else {
                Point { x: -1, y: 0 }
            }
        };
        Self {
            start,
            end,
            direction: dir,
            done: false,
        }
    }
}

impl Iterator for LineToStruct {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let current = self.start;
        if self.start == self.end {
            self.done = true
        }
        self.start = self.start + self.direction;
        Some(current)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

impl std::ops::Add for Point {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Point {
    fn line_to(&self, other: Self) -> impl Iterator<Item = Point> {
        LineToStruct::new(*self, other)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Cell {
    Empty,
    Rock,
    Sand,
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
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    current_sand: Option<Point>,
    sand_created: usize,
}

impl Scene {
    fn new(width: i32, height: i32) -> Self {
        Scene {
            width: width as usize,
            height: height as usize,
            cells: vec![Cell::Empty; width as usize * height as usize],
            current_sand: None,
            sand_created: 0,
        }
    }

    fn index_for(&self, coordinate: Point) -> usize {
        coordinate.x as usize + self.width * (coordinate.y as usize)
    }

    fn set(&mut self, coordinate: Point, contents: Cell) {
        let index = self.index_for(coordinate);
        self.cells[index] = contents;
    }

    fn get(&self, coordinate: Point) -> Cell {
        let index = self.index_for(coordinate);
        self.cells[index]
    }

    fn add_path(&mut self, start: Point, end: Point, of: Cell) {
        for coordinate in start.line_to(end) {
            self.set(coordinate, of);
        }
    }

    fn dump(&self) {
        for y in 0..self.height {
            let cells = (400..self.width)
                .map(|x| {
                    self.get(Point {
                        x: x as i32,
                        y: y as i32,
                    })
                    .as_char()
                })
                .collect::<String>();
            println!("{}", cells);
        }
    }

    fn step(&mut self) -> bool {
        if let Some(coordinate) = self.current_sand.take() {
            let down = coordinate + Point { x: 0, y: 1 };
            let down_left = coordinate + Point { x: -1, y: 1 };
            let down_right = coordinate + Point { x: 1, y: 1 };
            if down.y >= self.height as i32 || down.x >= self.width as i32 {
                self.sand_created -= 1;
                return false;
            }
            if self.get(down).is_empty() {
                self.current_sand = Some(down);
            } else if self.get(down_left).is_empty() {
                self.current_sand = Some(down_left);
            } else if self.get(down_right).is_empty() {
                self.current_sand = Some(down_right);
            } else {
                self.set(coordinate, Cell::Sand);
            }
        } else {
            if self.get(Point { x: 500, y: 0 }).is_empty() {
                self.sand_created += 1;
                self.current_sand = Some(Point { x: 500, y: 0 });
            } else {
                return false;
            }
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
            separated_pair(character::complete::i32, tag(","), character::complete::i32),
            |(x, y)| Point { x, y },
        ),
    )(s)
}

fn parse_scene(s: &str, mode: Mode) -> anyhow::Result<Scene> {
    let min_x = 0;
    let min_y = 0;
    let paths = s
        .split('\n')
        .filter(|l| !l.is_empty())
        .enumerate()
        .map(|(i, line)| {
            let (remaining, path) = parse_path(line)
                .map_err(|e| anyhow::anyhow!("error parsing line {}: {:?}", i + 1, e))?;
            if remaining.len() > 0 {
                anyhow::bail!("unhandled input in line {}: {:?}", i + 1, remaining);
            }
            Ok(path)
        })
        .collect::<anyhow::Result<Vec<Vec<Point>>>>()?;
    let max_x = paths
        .iter()
        .filter_map(|path| path.iter().map(|coordinate| coordinate.x).max())
        .max()
        .unwrap();
    let max_y = paths
        .iter()
        .filter_map(|path| path.iter().map(|coordinate| coordinate.y).max())
        .max()
        .unwrap();
    let height = 4 + max_y - min_y;
    let width = if mode == Mode::Part2 {
        // close enough to infinity
        1000
    } else {
        1 + max_x - min_x
    };
    let mut scene = Scene::new(width, height);
    for path in paths.into_iter() {
        for (lhs, rhs) in path.into_iter().tuple_windows() {
            scene.add_path(lhs, rhs, Cell::Rock);
        }
    }
    if mode == Mode::Part2 {
        scene.add_path(
            Point { x: 0, y: max_y + 2 },
            Point {
                x: width,
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

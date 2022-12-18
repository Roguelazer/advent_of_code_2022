use std::cmp::max;
use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;

use clap::{Parser, ValueEnum};
use itertools::{Itertools, MinMaxResult};

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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Vec3 {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3 {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    fn neighbors(&self) -> Vec<Vec3> {
        vec![
            Self::new(self.x + 1, self.y, self.z),
            Self::new(self.x - 1, self.y, self.z),
            Self::new(self.x, self.y + 1, self.z),
            Self::new(self.x, self.y - 1, self.z),
            Self::new(self.x, self.y, self.z + 1),
            Self::new(self.x, self.y, self.z - 1),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Unknown,
    Outside,
    AirBubble,
    Lava,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Unknown
    }
}

#[derive(Debug)]
struct Grid<V: Debug + Default + Clone + Copy> {
    cells: Vec<V>,
    width: i32,
    height: i32,
    depth: i32,
}

impl<V: Debug + Default + Clone + Copy> Grid<V> {
    fn new(b1: Vec3, b2: Vec3) -> Self {
        let width = max(b2.x, b1.x) + 1;
        let height = max(b2.y, b1.y) + 1;
        let depth = max(b2.z, b1.z) + 1;
        Self {
            cells: vec![V::default(); width as usize * height as usize * depth as usize],
            width: width as i32,
            height: height as i32,
            depth: depth as i32,
        }
    }

    fn index(&self, p: Vec3) -> usize {
        (p.x + p.y * self.width + p.z * (self.width * self.height)) as usize
    }

    fn get(&self, p: Vec3) -> V {
        let idx = self.index(p);
        self.cells[idx]
    }

    fn set(&mut self, p: Vec3, v: V) {
        let idx = self.index(p);
        self.cells[idx] = v;
    }

    fn iter(&self) -> impl Iterator<Item = (Vec3, &V)> {
        let width = self.width;
        let height = self.height;
        let depth = self.depth;
        (0..width).flat_map(move |x| {
            (0..height).flat_map(move |y| {
                (0..depth).map(move |z| {
                    let point = Vec3::new(x, y, z);
                    let index = self.index(point);
                    (point, &self.cells[index])
                })
            })
        })
    }
}

fn flood_fill(g: &Grid<Cell>, start: Vec3) -> (Vec<Vec3>, bool) {
    let mut connected = vec![];
    let mut todo = VecDeque::new();
    let mut seen = HashSet::new();
    todo.push_back(start);
    let mut is_empty = false;
    while let Some(next) = todo.pop_front() {
        connected.push(next);
        for neighbor in next.neighbors() {
            if neighbor.x < 0 || neighbor.y < 0 || neighbor.z < 0 {
                is_empty = true;
            } else if neighbor.x >= g.width || neighbor.y >= g.height || neighbor.z >= g.depth {
                is_empty = true
            } else {
                match g.get(neighbor) {
                    Cell::Lava => {}
                    Cell::AirBubble => unreachable!(),
                    Cell::Unknown => {
                        if !seen.contains(&neighbor) {
                            seen.insert(neighbor);
                            todo.push_back(neighbor);
                        }
                    }
                    Cell::Outside => is_empty = true,
                }
            }
        }
    }
    (connected, is_empty)
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
    let positions = input
        .split('\n')
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let mut fields = line.split(',').map(|v| v.parse::<i32>().unwrap());
            Vec3 {
                x: fields.next().unwrap(),
                y: fields.next().unwrap(),
                z: fields.next().unwrap(),
            }
        })
        .collect::<HashSet<Vec3>>();
    let total_surface_area: usize = positions
        .iter()
        .map(|p| {
            p.neighbors()
                .into_iter()
                .filter(|n| !positions.contains(n))
                .count()
        })
        .sum();
    if args.mode == Mode::Part1 {
        println!("exerior surface area: {}", total_surface_area);
    } else {
        let (min_x, max_x) = match positions.iter().map(|v| v.x).minmax() {
            MinMaxResult::MinMax(a, b) => (a, b),
            _ => panic!("unhandled x boundary"),
        };
        let (min_y, max_y) = match positions.iter().map(|v| v.y).minmax() {
            MinMaxResult::MinMax(a, b) => (a, b),
            _ => panic!("unhandled x boundary"),
        };
        let (min_z, max_z) = match positions.iter().map(|v| v.z).minmax() {
            MinMaxResult::MinMax(a, b) => (a, b),
            _ => panic!("unhandled x boundarz"),
        };
        let mut grid = Grid::<Cell>::new(
            Vec3::new(min_x, min_y, min_z),
            Vec3::new(max_x, max_y, max_z),
        );
        for item in positions {
            grid.set(item, Cell::Lava);
        }
        // pick the first unknown cell and try to flood fill
        loop {
            let coord = grid
                .iter()
                .find(|(_, c)| **c == Cell::Unknown)
                .map(|(v, _)| v)
                .clone();
            if let Some(coord) = coord {
                let (cells, state) = flood_fill(&grid, coord);
                for cell in cells {
                    grid.set(
                        cell,
                        if state {
                            Cell::Outside
                        } else {
                            Cell::AirBubble
                        },
                    )
                }
            } else {
                break;
            }
        }
        // compute the surface area of the air bubble(s)
        let bubbles: HashSet<Vec3> = grid
            .iter()
            .filter(|(_, c)| **c == Cell::AirBubble)
            .map(|(v, _)| v)
            .collect();
        let bubble_surface_area: usize = bubbles
            .iter()
            .map(|p| {
                p.neighbors()
                    .into_iter()
                    .filter(|n| !bubbles.contains(n))
                    .count()
            })
            .sum();
        println!(
            "{} - {} = {}",
            total_surface_area,
            bubble_surface_area,
            total_surface_area - bubble_surface_area
        );
    }
    Ok(())
}

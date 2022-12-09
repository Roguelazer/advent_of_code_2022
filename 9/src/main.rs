use std::collections::HashSet;
use std::io::BufRead;
use std::str::FromStr;

use clap::Parser;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Debug, PartialEq, Eq, Default, Clone, Copy, Hash, PartialOrd, Ord)]
struct Coordinate {
    x: i32,
    y: i32,
}

impl Coordinate {
    fn touches(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < 2 && (self.y - other.y).abs() < 2
    }

    fn apply(&self, ordinal: Ordinal) -> Self {
        match ordinal {
            Ordinal::Up => Coordinate {
                x: self.x,
                y: self.y + 1,
            },
            Ordinal::Down => Coordinate {
                x: self.x,
                y: self.y - 1,
            },
            Ordinal::Left => Coordinate {
                x: self.x - 1,
                y: self.y,
            },
            Ordinal::Right => Coordinate {
                x: self.x + 1,
                y: self.y,
            },
            Ordinal::UpRight => Coordinate {
                x: self.x + 1,
                y: self.y + 1,
            },
            Ordinal::UpLeft => Coordinate {
                x: self.x - 1,
                y: self.y + 1,
            },
            Ordinal::DownRight => Coordinate {
                x: self.x + 1,
                y: self.y - 1,
            },
            Ordinal::DownLeft => Coordinate {
                x: self.x - 1,
                y: self.y - 1,
            },
        }
    }

    fn direction_to(&self, other: &Coordinate) -> Option<Ordinal> {
        let x_offset = (other.x - self.x).signum();
        let y_offset = (other.y - self.y).signum();
        Some(match (x_offset, y_offset) {
            (0, 0) => return None,
            (1, 0) => Ordinal::Right,
            (-1, 0) => Ordinal::Left,
            (0, 1) => Ordinal::Up,
            (0, -1) => Ordinal::Down,
            (1, 1) => Ordinal::UpRight,
            (1, -1) => Ordinal::DownRight,
            (-1, 1) => Ordinal::UpLeft,
            (-1, -1) => Ordinal::DownLeft,
            _ => unreachable!("signum only returns [-1, 0, 1]"),
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum Ordinal {
    Up,
    Down,
    Left,
    Right,
    UpRight,
    UpLeft,
    DownRight,
    DownLeft,
}

impl FromStr for Ordinal {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            anyhow::bail!("invalid ordinal len");
        }
        Ok(match s {
            "R" => Ordinal::Right,
            "U" => Ordinal::Up,
            "D" => Ordinal::Down,
            "L" => Ordinal::Left,
            _ => anyhow::bail!("invalid ordinal {}", s),
        })
    }
}

#[derive(Debug)]
struct Command {
    ordinal: Ordinal,
    step: u32,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.len() < 3 {
            anyhow::bail!("invalid command length");
        }
        Ok(Command {
            ordinal: s[0..1].parse()?,
            step: s[2..].parse()?,
        })
    }
}

#[derive(Debug)]
struct Knot {
    label: char,
    position: Coordinate,
    visited_positions: HashSet<Coordinate>,
}

impl Knot {
    fn new(label: u8) -> Self {
        let position = Coordinate::default();
        let mut visited_positions = HashSet::new();
        visited_positions.insert(position);
        let label = if label == 0 {
            'H'
        } else {
            char::from(label + b'a')
        };
        Self {
            label,
            position,
            visited_positions,
        }
    }

    fn move_to(&mut self, position: Coordinate) {
        self.visited_positions.insert(position);
        self.position = position;
    }

    fn apply(&self, ordinal: Ordinal) -> Coordinate {
        self.position.apply(ordinal)
    }

    fn follow(&self, other: &Knot) -> Coordinate {
        if !self.position.touches(&other.position) {
            if let Some(direction_to) = self.position.direction_to(&other.position) {
                self.position.apply(direction_to)
            } else {
                self.position
            }
        } else {
            self.position
        }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_parser)]
    num_knots: u8,
    #[arg(short, long)]
    verbose: bool,
    #[arg(short, long, value_parser, default_value("32"))]
    ms_per_frame: u64,
}

fn render<W: std::io::Write>(out: &mut W, knots: &[Knot], i: usize) -> anyhow::Result<()> {
    let (width, height) = crossterm::terminal::size()
        .map(|(w, h)| (w as i32, h as i32))
        .unwrap_or((80, 40));
    let (min_x, max_x, min_y, max_y) = (
        -1 * width / 2 + 1,
        width / 2,
        -1 * height / 2 + 1,
        height / 2,
    );
    execute!(out, MoveTo(0, 0))?;
    (min_y..=max_y).for_each(|y| {
        (min_x..=max_x).for_each(|x| {
            if let Some(k) = knots.iter().find(|k| k.position == Coordinate { x, y }) {
                print!("{}", k.label);
            } else {
                print!(" ");
            }
        });
        print!("\n");
    });
    execute!(out, MoveTo(0, (height as u16) - 1))?;
    print!("{:10}", i);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin_r = std::io::stdin();
    let stdin = stdin_r.lock();
    let num_knots = args.num_knots;
    let mut knots = (0..num_knots).map(|i| Knot::new(i)).collect::<Vec<Knot>>();
    let stdout_r = std::io::stdout();
    let mut stdout = stdout_r.lock();
    if args.verbose {
        execute!(&mut stdout, EnterAlternateScreen)?;
        execute!(&mut stdout, Clear(ClearType::All))?;
        execute!(&mut stdout, Hide)?;
    }
    for (i, line) in stdin.lines().enumerate() {
        let command: Command = line?.parse()?;
        for _ in 0..command.step {
            for knot_offset in 0..knots.len() {
                let dir = if knot_offset == 0 {
                    knots[knot_offset].apply(command.ordinal)
                } else {
                    knots[knot_offset].follow(&knots[knot_offset - 1])
                };
                knots.get_mut(knot_offset).unwrap().move_to(dir);
            }
            if args.verbose {
                render(&mut stdout, knots.as_slice(), i)?;
                std::thread::sleep(std::time::Duration::from_millis(args.ms_per_frame));
            }
        }
    }
    if args.verbose {
        execute!(&mut stdout, Show)?;
        execute!(&mut stdout, LeaveAlternateScreen)?;
    }
    if let Some(last) = knots.last() {
        println!("{}", last.visited_positions.len());
    }
    Ok(())
}

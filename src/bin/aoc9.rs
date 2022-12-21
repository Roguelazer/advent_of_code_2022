use std::collections::HashSet;
use std::io::BufRead;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
        }
    }

    fn move_toward(&self, other: &Coordinate) -> Coordinate {
        Coordinate {
            x: self.x + (other.x - self.x).signum(),
            y: self.y + (other.y - self.y).signum(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Ordinal {
    Up,
    Down,
    Left,
    Right,
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
            char::from((label % 53) + b'I')
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

    fn apply(&mut self, ordinal: Ordinal) {
        self.move_to(self.position.apply(ordinal));
    }

    fn follow(&self, other: &Knot) -> Coordinate {
        if !self.position.touches(&other.position) {
            self.position.move_toward(&other.position)
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
    #[arg(long)]
    trails: bool,
}

fn render<W: std::io::Write>(
    out: &mut W,
    knots: &[Knot],
    frame: u64,
    command: usize,
    trails: bool,
) -> anyhow::Result<()> {
    let (width, height) = crossterm::terminal::size()
        .map(|(w, h)| (w as i32, h as i32))
        .unwrap_or((80, 40));
    let (min_x, max_x, min_y, max_y) = (-width / 2 + 1, width / 2, -height / 2 + 1, height / 2 - 1);
    execute!(out, MoveTo(0, 0))?;
    (min_y..=max_y).for_each(|y| {
        let line = (min_x..=max_x)
            .map(|x| {
                let coord = Coordinate { x, y };
                if let Some(k) = knots.iter().find(|k| k.position == coord) {
                    k.label
                } else if trails
                    && knots
                        .iter()
                        .last()
                        .map(|k| k.visited_positions.contains(&coord))
                        .unwrap_or(false)
                {
                    '#'
                } else {
                    ' '
                }
            })
            .collect::<String>();
        println!("{}", line);
    });
    execute!(out, MoveTo(0, height as u16))?;
    print!(" [ frame {:<10} (command {:<10}) ]", frame, command + 1);
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin_r = std::io::stdin();
    let stdin = stdin_r.lock();
    let num_knots = args.num_knots;
    let mut knots = (0..num_knots).map(Knot::new).collect::<Vec<Knot>>();
    let stdout_r = std::io::stdout();
    let mut stdout = stdout_r.lock();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    if args.verbose {
        execute!(&mut stdout, EnterAlternateScreen)?;
        execute!(&mut stdout, Clear(ClearType::All))?;
        execute!(&mut stdout, Hide)?;
    }
    let mut applied = 0u64;
    for (i, line) in stdin.lines().enumerate() {
        let command: Command = line?.parse()?;
        for _ in 0..command.step {
            knots[0].apply(command.ordinal);
            for knot_offset in 1..knots.len() {
                let dir = knots[knot_offset].follow(&knots[knot_offset - 1]);
                knots.get_mut(knot_offset).unwrap().move_to(dir);
            }
            applied += 1;
            if args.verbose {
                render(&mut stdout, knots.as_slice(), applied, i, args.trails)?;
                std::thread::sleep(std::time::Duration::from_millis(args.ms_per_frame));
            }
        }
        if !running.load(Ordering::SeqCst) {
            break;
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

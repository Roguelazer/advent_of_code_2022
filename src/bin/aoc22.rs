use clap::{Parser, ValueEnum};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::one_of,
    combinator::map,
    multi::{many1, separated_list1},
    IResult,
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
    #[clap(short, long)]
    verbose: bool,
    #[clap(short, long, value_enum)]
    mode: Mode,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
enum Cell {
    Traversed(Direction),
    Missing,
    Empty,
    Wall,
}

impl Cell {
    fn as_char(&self) -> char {
        match self {
            Cell::Traversed(d) => d.as_char(),
            Cell::Missing => ' ',
            Cell::Empty => '.',
            Cell::Wall => '#',
        }
    }
}

impl HasEmpty for Cell {
    fn empty_value() -> Self {
        Cell::Missing
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    fn as_char(&self) -> char {
        match self {
            Self::Up => '^',
            Self::Down => 'v',
            Self::Left => '<',
            Self::Right => '>',
        }
    }

    fn turn(&self, turn: &Turn) -> Self {
        match (self, turn) {
            (Self::Up, Turn::Clockwise) => Self::Right,
            (Self::Right, Turn::Clockwise) => Self::Down,
            (Self::Down, Turn::Clockwise) => Self::Left,
            (Self::Left, Turn::Clockwise) => Self::Up,
            (Self::Up, Turn::Counterclockwise) => Self::Left,
            (Self::Right, Turn::Counterclockwise) => Self::Up,
            (Self::Down, Turn::Counterclockwise) => Self::Right,
            (Self::Left, Turn::Counterclockwise) => Self::Down,
        }
    }

    fn score(&self) -> u32 {
        match self {
            Self::Up => 3,
            Self::Right => 0,
            Self::Down => 1,
            Self::Left => 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Turn {
    Clockwise,
    Counterclockwise,
}

#[derive(Debug)]
enum Instruction {
    Steps(usize),
    Turn(Turn),
}

#[derive(Debug)]
struct State {
    position: Point,
    direction: Direction,
}

#[derive(Debug)]
struct Board {
    grid: DenseGrid<Cell>,
    instructions: Vec<Instruction>,
}

fn parse_grid_line(s: &str) -> IResult<&str, Vec<Cell>> {
    many1(map(one_of(" .#"), |c| match c {
        ' ' => Cell::Missing,
        '.' => Cell::Empty,
        '#' => Cell::Wall,
        _ => unreachable!(),
    }))(s)
}

fn parse_grid(s: &str) -> IResult<&str, DenseGrid<Cell>> {
    map(separated_list1(tag("\n"), parse_grid_line), |lines| {
        let width = lines.iter().map(|l| l.len()).max().unwrap();
        let mut g = DenseGrid::new_with(
            Point::new(1, 1),
            Point::new(width as i64, lines.len() as i64),
            Cell::Missing,
        );
        for (y, row) in lines.into_iter().enumerate() {
            for (x, cell) in row.into_iter().enumerate() {
                g.set(Point::new(x as i64 + 1, y as i64 + 1), cell);
            }
        }
        g
    })(s)
}

fn parse_instructions(s: &str) -> IResult<&str, Vec<Instruction>> {
    many1(alt((
        map(nom::character::complete::u32, |s| {
            Instruction::Steps(s as usize)
        }),
        map(one_of("RL"), |rl| {
            Instruction::Turn(match rl {
                'R' => Turn::Clockwise,
                'L' => Turn::Counterclockwise,
                _ => unreachable!(),
            })
        }),
    )))(s)
}

fn step(mut position: Point, direction: Direction, grid: &DenseGrid<Cell>) -> (Point, Cell) {
    loop {
        let increment = match direction {
            Direction::Right => Point::new(1, 0),
            Direction::Left => Point::new(-1, 0),
            Direction::Up => Point::new(0, -1),
            Direction::Down => Point::new(0, 1),
        };
        position = position + increment;
        if position.x > grid.width() as i64 {
            position.x = 1
        } else if position.x < 1 {
            position.x = grid.width() as i64;
        } else if position.y > grid.height() as i64 {
            position.y = 1;
        } else if position.y < 1 {
            position.y = grid.height() as i64;
        }
        match grid.get(position) {
            Some(Cell::Missing) => continue,
            Some(other) => return (position, other),
            None => unreachable!(),
        }
    }
}

fn simulate(board: &mut Board) -> u32 {
    let first_empty = (1..=board.grid.width())
        .find_map(|x| {
            let coordinate = Point::new(x as i64, 1);
            if board.grid.get(coordinate) == Some(Cell::Empty) {
                Some(coordinate)
            } else {
                None
            }
        })
        .unwrap();
    let mut state = State {
        position: first_empty,
        direction: Direction::Right,
    };
    for instruction in board.instructions.iter() {
        board
            .grid
            .set(state.position, Cell::Traversed(state.direction));
        log::debug!("state = {:?}", state);
        log::debug!("running {:?}", instruction);
        match instruction {
            Instruction::Steps(s) => {
                for _ in 0..*s {
                    board
                        .grid
                        .set(state.position, Cell::Traversed(state.direction));
                    let (next, cell) = step(state.position, state.direction, &board.grid);
                    match cell {
                        Cell::Empty | Cell::Traversed(_) => state.position = next,
                        Cell::Wall => {
                            log::debug!("hit a wall at {:?} pointed {:?}", next, state.direction);
                            break;
                        }
                        Cell::Missing => unreachable!(),
                    }
                }
            }
            Instruction::Turn(t) => {
                state.direction = state.direction.turn(t);
            }
        }
    }
    board
        .grid
        .set(state.position, Cell::Traversed(state.direction));
    1000 * state.position.y as u32 + 4 * state.position.x as u32 + state.direction.score()
}

fn parse_board(s: &str) -> anyhow::Result<Board> {
    let (grid, instructions) = s.split_once("\n\n").unwrap();
    let (rem, grid) =
        parse_grid(grid).map_err(|e| anyhow::anyhow!("unable to parse grid: {:?}", e))?;
    if !rem.trim().is_empty() {
        anyhow::bail!("unhandled input: {:?}", rem);
    }
    let (rem, instructions) = parse_instructions(instructions.trim())
        .map_err(|e| anyhow::anyhow!("unable to parse grid: {:?}", e))?;
    if !rem.trim().is_empty() {
        anyhow::bail!("unhandled input: {:?}", rem);
    }
    Ok(Board { grid, instructions })
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
    let mut board = parse_board(&input)?;
    if args.verbose {
        board.grid.dump_with(Cell::as_char)
    }
    let score = simulate(&mut board);
    if args.verbose {
        board.grid.dump_with(Cell::as_char)
    }
    println!("{}", score);
    Ok(())
}

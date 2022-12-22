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

fn step_part1(
    mut position: Point,
    direction: Direction,
    grid: &DenseGrid<Cell>,
) -> (Point, Direction, Cell) {
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
            Some(other) => return (position, direction, other),
            None => unreachable!(),
        }
    }
}

fn get_face(position: Point, face_size: usize) -> u8 {
    if face_size == 4 {
        if position.y < 5 {
            1
        } else if position.y < 9 && position.x < 5 {
            2
        } else if position.y < 9 && position.x < 9 {
            3
        } else if position.y < 9 && position.x < 13 {
            4
        } else if position.y > 8 && position.y < 13 && position.x > 8 && position.x < 13 {
            5
        } else if position.y > 8 && position.y < 13 && position.x > 12 && position.x < 17 {
            6
        } else {
            unreachable!()
        }
    } else if face_size == 50 {
        if position.x < 101 && position.y < 51 {
            1
        } else if position.y < 51 {
            2
        } else if position.y > 50 && position.y < 101 {
            3
        } else if position.y < 151 && position.x < 51 {
            4
        } else if position.y < 151 && position.x < 101 {
            5
        } else if position.y < 201 && position.x < 51 {
            6
        } else {
            unreachable!();
        }
    } else {
        panic!("unhandled face size {:?}", 50)
    }
}

fn step_part2(
    position: Point,
    direction: Direction,
    grid: &DenseGrid<Cell>,
    face_size: usize,
) -> (Point, Direction, Cell) {
    let increment = match direction {
        Direction::Right => Point::new(1, 0),
        Direction::Left => Point::new(-1, 0),
        Direction::Up => Point::new(0, -1),
        Direction::Down => Point::new(0, 1),
    };
    let next_position = position + increment;
    match grid.get(next_position) {
        Some(Cell::Missing) => {}
        Some(other) => return (next_position, direction, other),
        _ => {}
    };
    let face = get_face(position, face_size);
    log::debug!("wraparound at {} ({})", position, face);
    let adjacencies = if face_size == 4 {
        vec![
            (
                Direction::Left,
                1,
                Point::new(position.y + 4, 5),
                Direction::Down,
            ),
            (
                Direction::Up,
                1,
                Point::new(13 - position.x, 5),
                Direction::Down,
            ),
            (
                Direction::Right,
                1,
                Point::new(16, 13 - position.y),
                Direction::Left,
            ),
            (
                Direction::Left,
                2,
                Point::new(21 - position.y, 12),
                Direction::Up,
            ),
            (
                Direction::Up,
                2,
                Point::new(13 - position.x, 1),
                Direction::Down,
            ),
            (
                Direction::Down,
                2,
                Point::new(13 - position.x, 12),
                Direction::Down,
            ),
            (
                Direction::Up,
                3,
                Point::new(9, position.x - 4),
                Direction::Right,
            ),
            (
                Direction::Down,
                3,
                Point::new(9, 17 - position.x),
                Direction::Right,
            ),
            (
                Direction::Right,
                4,
                Point::new(21 - position.y, 9),
                Direction::Down,
            ),
            (
                Direction::Left,
                5,
                Point::new(17 - position.y, 8),
                Direction::Up,
            ),
            (
                Direction::Down,
                5,
                Point::new(13 - position.x, 8),
                Direction::Up,
            ),
            (
                Direction::Down,
                6,
                Point::new(1, 21 - position.x),
                Direction::Right,
            ),
            (
                Direction::Right,
                6,
                Point::new(12, 13 - position.y),
                Direction::Left,
            ),
            (
                Direction::Up,
                6,
                Point::new(12, 21 - position.y),
                Direction::Left,
            ),
        ]
    } else if face_size == 50 {
        vec![
            (
                Direction::Left,
                1,
                Point::new(1, 151 - position.y),
                Direction::Right,
            ),
            (
                Direction::Up,
                1,
                Point::new(1, position.x + 100),
                Direction::Right,
            ),
            (
                Direction::Up,
                2,
                Point::new(position.x - 100, 200),
                Direction::Up,
            ),
            (
                Direction::Right,
                2,
                Point::new(100, 151 - position.y),
                Direction::Left,
            ),
            (
                Direction::Down,
                2,
                Point::new(100, position.x - 50),
                Direction::Left,
            ),
            (
                Direction::Right,
                3,
                Point::new(position.y + 50, 50),
                Direction::Up,
            ),
            (
                Direction::Right,
                5,
                Point::new(150, 151 - position.y),
                Direction::Left,
            ),
            (
                Direction::Down,
                5,
                Point::new(50, position.x + 100),
                Direction::Left,
            ),
            (
                Direction::Right,
                6,
                Point::new(position.y - 100, 150),
                Direction::Up,
            ),
            (
                Direction::Down,
                6,
                Point::new(position.x + 100, 1),
                Direction::Down,
            ),
            (
                Direction::Left,
                6,
                Point::new(position.y - 100, 1),
                Direction::Down,
            ),
            (
                Direction::Left,
                4,
                Point::new(51, 151 - position.y),
                Direction::Right,
            ),
            (
                Direction::Up,
                4,
                Point::new(51, position.x + 50),
                Direction::Right,
            ),
            (
                Direction::Left,
                3,
                Point::new(position.y - 50, 101),
                Direction::Down,
            ),
        ]
    } else {
        panic!("unhandled face size {}", face_size);
    };
    for (pdirection, pface, point, new_direction) in adjacencies {
        if pdirection == direction && pface == face {
            return (point, new_direction, grid.get(point).unwrap());
        }
    }
    panic!(
        "no transition found for face {} going {:?}",
        face, direction
    );
}

fn simulate(board: &mut Board, mode: Mode) -> u32 {
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
    let face_size = (1..board.grid.height())
        .map(|y| {
            (1..board.grid.width())
                .filter(|x| board.grid.get(Point::new(*x as i64, y as i64)) != Some(Cell::Missing))
                .count()
        })
        .min()
        .unwrap();
    log::info!("part 2 face size is {}", face_size);
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
                    let (next, direction, cell) = match mode {
                        Mode::Part1 => step_part1(state.position, state.direction, &board.grid),
                        Mode::Part2 => {
                            step_part2(state.position, state.direction, &board.grid, face_size)
                        }
                    };
                    match cell {
                        Cell::Empty | Cell::Traversed(_) => {
                            state.direction = direction;
                            state.position = next;
                        }
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
    let score = simulate(&mut board, args.mode);
    if args.verbose {
        board.grid.dump_with(Cell::as_char)
    }
    println!("{}", score);
    Ok(())
}

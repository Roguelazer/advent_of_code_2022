use std::ops::RangeInclusive;

use clap::{Parser, ValueEnum};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    combinator::map,
    multi::separated_list1,
    sequence::{preceded, separated_pair},
    IResult,
};

use aoclib::Point;

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
    #[clap(short, long, value_parser)]
    target_line: i64,
}

#[derive(Debug)]
struct SensorLine {
    me: Point,
    #[allow(dead_code)]
    neighbor: Point,
    radius: usize,
}

impl SensorLine {
    fn new(me: Point, neighbor: Point) -> Self {
        let radius = me.manhattan_distance_to(neighbor);
        Self {
            me,
            neighbor,
            radius,
        }
    }

    fn occludes(&self, point: Point) -> bool {
        self.me.manhattan_distance_to(point) <= self.radius
    }

    fn projected_to_x(&self, x: i64) -> Option<RangeInclusive<i64>> {
        if (self.me.x - x).abs() > self.radius as i64 {
            None
        } else {
            let distance = (self.me.x - x).abs();
            let radius_at_distance = self.radius as i64 - distance;
            Some((self.me.y - radius_at_distance)..=(self.me.y + radius_at_distance))
        }
    }

    fn projected_to_y(&self, y: i64) -> Option<RangeInclusive<i64>> {
        if (self.me.y - y).abs() > self.radius as i64 {
            None
        } else {
            let distance = (self.me.y - y).abs();
            let radius_at_distance = self.radius as i64 - distance;
            Some((self.me.x - radius_at_distance)..=(self.me.x + radius_at_distance))
        }
    }
}

fn parse_coordinate(s: &str) -> IResult<&str, Point> {
    map(
        separated_pair(
            preceded(tag("x="), nom::character::complete::i64),
            tag(", "),
            preceded(tag("y="), nom::character::complete::i64),
        ),
        |(x, y)| Point::new(x, y),
    )(s)
}

fn parse_sensor_line(s: &str) -> IResult<&str, SensorLine> {
    map(
        separated_pair(
            preceded(tag("Sensor at "), parse_coordinate),
            tag(": closest beacon is at "),
            parse_coordinate,
        ),
        |(me, neighbor)| SensorLine::new(me, neighbor),
    )(s)
}

fn parse_sensor_lines(s: &str) -> anyhow::Result<Vec<SensorLine>> {
    let (remaining, lines) = separated_list1(tag("\n"), parse_sensor_line)(s)
        .map_err(|e| anyhow::anyhow!("invalid line {:?}", e))?;

    if !remaining.trim().is_empty() {
        anyhow::bail!("unhandled input {:?}", remaining);
    }
    Ok(lines)
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
    let lines = parse_sensor_lines(&input)?;
    if args.mode == Mode::Part1 {
        let possible_x_coordinates = lines
            .iter()
            .filter_map(|sensor| {
                let distance = sensor.me.y.abs_diff(args.target_line);
                if distance <= sensor.radius as u64 {
                    Some(sensor)
                } else {
                    None
                }
            })
            .inspect(|p| log::debug!("intersection with {:?}", p))
            .map(|sensor| {
                let min_x = sensor.me.x - sensor.radius as i64;
                let max_x = sensor.me.x + sensor.radius as i64;
                min_x..=max_x
            })
            .collect::<Vec<_>>();
        log::debug!("possible_x_coordinates: {:?}", possible_x_coordinates);
        let min_x = possible_x_coordinates
            .iter()
            .map(|r| *r.start())
            .min()
            .unwrap();
        let max_x = possible_x_coordinates
            .iter()
            .map(|r| *r.end())
            .max()
            .unwrap();
        log::debug!("inspecting {:?}", min_x..=max_x);
        let covered = (min_x..=max_x)
            .filter(|x| possible_x_coordinates.iter().any(|c| c.contains(x)))
            .map(|x| {
                let point = Point::new(x, args.target_line);
                if lines
                    .iter()
                    .any(|s| s.neighbor != point && s.occludes(point))
                {
                    1
                } else {
                    0
                }
            })
            .sum::<usize>();
        println!("covered: {:?}", covered);
    } else {
        let min = 0;
        let max = args.target_line;
        let non_covered_x = (min..=max)
            .find(|x| {
                let mut current = None;
                for range in lines
                    .iter()
                    .filter_map(|sensor| sensor.projected_to_x(*x))
                    .sorted_by_key(|r| *r.start())
                {
                    match current {
                        None => current = Some(range),
                        Some(c) if *range.start() <= *c.end() + 1 => {
                            current = Some(*c.start()..=std::cmp::max(*range.end(), *c.end()))
                        }
                        _ => {
                            return true;
                        }
                    }
                }
                false
            })
            .unwrap();
        let non_covered_y = (min..=max)
            .find(|y| {
                let mut current = None;
                for range in lines
                    .iter()
                    .filter_map(|sensor| sensor.projected_to_y(*y))
                    .sorted_by_key(|r| *r.start())
                {
                    match current {
                        None => current = Some(range),
                        Some(c) if *range.start() <= *c.end() + 1 => {
                            current = Some(*c.start()..=std::cmp::max(*range.end(), *c.end()))
                        }
                        _ => {
                            return true;
                        }
                    }
                }
                false
            })
            .unwrap();
        let point = Point::new(non_covered_x, non_covered_y);
        if lines.iter().any(|s| s.occludes(point)) {
            panic!("uh oh! occlusion!")
        }
        println!("{}", point.x * 4000000 + point.y)
    }
    Ok(())
}

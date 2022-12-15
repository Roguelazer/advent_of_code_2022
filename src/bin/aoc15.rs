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
    param: i64,
}

#[derive(Debug)]
struct Sensor {
    me: Point,
    neighbor: Point,
    radius: usize,
}

impl Sensor {
    fn new(me: Point, neighbor: Point) -> Self {
        let radius = me.manhattan_distance_to(neighbor);
        Self {
            me,
            neighbor,
            radius,
        }
    }

    pub fn occludes(&self, point: Point) -> bool {
        self.me.manhattan_distance_to(point) <= self.radius
    }

    fn projected_to(&self, my: i64, their: i64, center: i64) -> Option<RangeInclusive<i64>> {
        let distance = (my - their).abs();
        if distance > self.radius as i64 {
            None
        } else {
            let radius_at_distance = self.radius as i64 - distance;
            Some((center - radius_at_distance)..=(center + radius_at_distance))
        }
    }

    /// Find the number of points covered on a line at going from (x, -inf) to (x, inf)
    pub fn projected_to_x(&self, x: i64) -> Option<RangeInclusive<i64>> {
        self.projected_to(self.me.x, x, self.me.y)
    }

    /// Find the number of points covered on a line at going from (-inf, y) to (inf, y)
    pub fn projected_to_y(&self, y: i64) -> Option<RangeInclusive<i64>> {
        self.projected_to(self.me.y, y, self.me.x)
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

fn parse_sensor_line(s: &str) -> IResult<&str, Sensor> {
    map(
        separated_pair(
            preceded(tag("Sensor at "), parse_coordinate),
            tag(": closest beacon is at "),
            parse_coordinate,
        ),
        |(me, neighbor)| Sensor::new(me, neighbor),
    )(s)
}

fn parse_sensor_lines(s: &str) -> anyhow::Result<Vec<Sensor>> {
    let (remaining, lines) = separated_list1(tag("\n"), parse_sensor_line)(s)
        .map_err(|e| anyhow::anyhow!("invalid line {:?}", e))?;

    if !remaining.trim().is_empty() {
        anyhow::bail!("unhandled input {:?}", remaining);
    }
    Ok(lines)
}

fn has_gap_in_ranges(i: &mut Vec<RangeInclusive<i64>>, min: i64, max: i64) -> bool {
    merge_ranges(i);
    i.iter()
        .tuple_windows()
        .any(|(lhs, rhs)| *lhs.end() > min && *rhs.start() > min && *lhs.end() < max)
}

fn merge_ranges(r: &mut Vec<RangeInclusive<i64>>) {
    if r.len() < 2 {
        return;
    }
    r.sort_by_key(|r| *r.start());
    let (mut current, mut current_index) = (r[0].clone(), 0);
    for i in 1..r.len() {
        let this = r[i].clone();
        if *this.start() <= *current.end() + 1 {
            current = *current.start()..=std::cmp::max(*this.end(), *current.end());
            r[current_index] = current.clone();
        } else {
            current_index += 1;
            current = this.clone();
            r[current_index] = this.clone();
        }
    }
    r.truncate(current_index + 1);
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
    let start = std::time::Instant::now();
    log::debug!("parsing input");
    let lines = parse_sensor_lines(&input)?;
    if args.mode == Mode::Part1 {
        let mut covered_ranges = lines
            .iter()
            .filter_map(|sensor| sensor.projected_to_y(args.param))
            .collect::<Vec<_>>();
        covered_ranges.sort_by_key(|r| *r.start());
        log::debug!("covered before merging: {:?}", covered_ranges);
        merge_ranges(&mut covered_ranges);
        log::debug!("covered after merging: {:?}", covered_ranges);
        let beacons_in_range = lines
            .iter()
            .filter(|s| s.neighbor.y == args.param)
            .filter(|s| covered_ranges.iter().any(|r| r.contains(&s.neighbor.x)))
            .map(|s| s.neighbor.x)
            .unique()
            .count() as u64;
        log::debug!("there are {} beacons on the line", beacons_in_range);
        let covered = covered_ranges
            .into_iter()
            .map(|r| r.end().abs_diff(*r.start()) + 1)
            .sum::<u64>()
            - beacons_in_range;
        println!("covered: {:?}", covered);
    } else {
        let min = 0;
        let max = args.param;
        let mut buf = Vec::with_capacity(lines.len());
        log::debug!("scanning for potential x coordinates");
        let non_covered_x = (min..=max)
            .filter(|x| {
                buf.clear();
                buf.extend(lines.iter().filter_map(|sensor| sensor.projected_to_x(*x)));
                has_gap_in_ranges(&mut buf, min, max)
            })
            .collect::<Vec<i64>>();
        log::debug!("scanning for potential y coordinates");
        let non_covered_y = (min..=max)
            .filter(|y| {
                buf.clear();
                buf.extend(lines.iter().filter_map(|sensor| sensor.projected_to_y(*y)));
                has_gap_in_ranges(&mut buf, min, max)
            })
            .collect::<Vec<i64>>();
        log::debug!(
            "found {} x coordinates and {} y coordinates",
            non_covered_x.len(),
            non_covered_y.len()
        );
        'outer: for x in non_covered_x {
            for y in &non_covered_y {
                let point = Point::new(x, *y);
                if !lines.iter().any(|s| s.occludes(point)) {
                    log::info!("Frequency {} at {}", point.x * args.param + point.y, point);
                    break 'outer;
                }
            }
        }
        log::debug!("succeeded in {:?}", start.elapsed());
    }
    Ok(())
}

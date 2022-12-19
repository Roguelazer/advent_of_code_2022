use clap::{Parser, ValueEnum};
use lru_cache::LruCache;
use nom::{
    bytes::complete::tag,
    combinator::map,
    multi::separated_list1,
    sequence::{delimited, preceded, terminated},
    IResult,
};
use rayon::prelude::*;

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

#[derive(Debug)]
struct Blueprint {
    id: u16,
    ore_cost: u16,
    clay_cost: u16,
    obsidian_cost: (u16, u16),
    geode_cost: (u16, u16),
}

impl Blueprint {
    fn max_ore_use(&self) -> u16 {
        std::cmp::max(
            self.ore_cost,
            std::cmp::max(
                self.clay_cost,
                std::cmp::max(self.obsidian_cost.0, self.geode_cost.0),
            ),
        )
    }
    fn max_clay_use(&self) -> u16 {
        self.obsidian_cost.1
    }
    fn max_obsidian_use(&self) -> u16 {
        self.geode_cost.1
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
struct Inventory {
    obsidian: u16,
    clay: u16,
    obsidian_robots: u16,
    clay_robots: u16,
    ore: u16,
    ore_robots: u16,
}

impl Inventory {
    fn new() -> Self {
        Self {
            ore: 0,
            clay: 0,
            obsidian: 0,
            ore_robots: 1,
            clay_robots: 0,
            obsidian_robots: 0,
        }
    }

    fn next(&self) -> Self {
        let mut n = self.clone();
        n.ore += self.ore_robots;
        n.clay += self.clay_robots;
        n.obsidian += self.obsidian_robots;
        n
    }
}

fn parse_blueprint(s: &str) -> IResult<&str, Blueprint> {
    map(
        nom::sequence::tuple((
            delimited(tag("Blueprint "), nom::character::complete::u16, tag(": ")),
            delimited(
                tag("Each ore robot costs "),
                nom::character::complete::u16,
                tag(" ore. "),
            ),
            delimited(
                tag("Each clay robot costs "),
                nom::character::complete::u16,
                tag(" ore. "),
            ),
            preceded(
                tag("Each obsidian robot costs "),
                nom::sequence::tuple((
                    terminated(nom::character::complete::u16, tag(" ore and ")),
                    terminated(nom::character::complete::u16, tag(" clay. ")),
                )),
            ),
            preceded(
                tag("Each geode robot costs "),
                nom::sequence::tuple((
                    terminated(nom::character::complete::u16, tag(" ore and ")),
                    terminated(nom::character::complete::u16, tag(" obsidian.")),
                )),
            ),
        )),
        |(id, ore_cost, clay_cost, (obs_ore, obs_clay), (geode_ore, geode_obs))| Blueprint {
            id,
            ore_cost,
            clay_cost,
            obsidian_cost: (obs_ore, obs_clay),
            geode_cost: (geode_ore, geode_obs),
        },
    )(s)
}

fn parse_blueprints(s: &str) -> anyhow::Result<Vec<Blueprint>> {
    let (res, bps) = separated_list1(tag("\n"), parse_blueprint)(s)
        .map_err(|e| anyhow::anyhow!("unable to parse input: {:?}", e))?;
    if !res.trim().is_empty() {
        anyhow::bail!("unparsed input: {:?}", res);
    }
    Ok(bps)
}

fn simulate_with(blueprint: &Blueprint, inventory: Inventory, ticks: u16) -> u16 {
    let mut work = Vec::new();
    let mut next_work = Vec::new();
    let mut seen = LruCache::new(1000000);
    let mut best = 0;
    let mut done = false;
    work.push((0, inventory, ticks));
    while !done {
        while let Some((geodes, inventory, remaining_ticks)) = work.pop() {
            best = std::cmp::max(best, geodes);
            if remaining_ticks <= 1 {
                done = true
            }
            if seen
                .insert((geodes, inventory.clone(), remaining_ticks), ())
                .is_some()
            {
                continue;
            }
            if inventory.ore >= blueprint.geode_cost.0
                && inventory.obsidian >= blueprint.geode_cost.1
            {
                // greedy, probably unsafe optimization: always buy a geode robot if you can
                // you could probably construct a parameter set where this fails but what
                // are the odds that AoC did that?
                let mut next = inventory.next();
                next.ore -= blueprint.geode_cost.0;
                next.obsidian -= blueprint.geode_cost.1;
                let these_geodes = remaining_ticks - 1;
                next_work.push((geodes + these_geodes, next, remaining_ticks - 1));
            } else {
                if inventory.ore >= blueprint.obsidian_cost.0
                    && inventory.clay >= blueprint.obsidian_cost.1
                    && inventory.obsidian_robots < blueprint.max_obsidian_use()
                {
                    let mut next = inventory.next();
                    next.ore -= blueprint.obsidian_cost.0;
                    next.clay -= blueprint.obsidian_cost.1;
                    next.obsidian_robots += 1;
                    next_work.push((geodes, next, remaining_ticks - 1))
                }
                if inventory.ore >= blueprint.ore_cost
                    && inventory.ore_robots < blueprint.max_ore_use()
                {
                    let mut next = inventory.next();
                    next.ore -= blueprint.ore_cost;
                    next.ore_robots += 1;
                    next_work.push((geodes, next, remaining_ticks - 1))
                }
                if inventory.ore >= blueprint.clay_cost
                    && inventory.clay_robots < blueprint.max_clay_use()
                {
                    let mut next = inventory.next();
                    next.ore -= blueprint.clay_cost;
                    next.clay_robots += 1;
                    next_work.push((geodes, next, remaining_ticks - 1))
                }
                next_work.push((geodes, inventory.next(), remaining_ticks - 1));
            }
        }
        // this trick is borred from vwoo; only consider the most successful fronts from this BFS
        next_work.sort_by(|a, b| b.cmp(a));
        next_work.truncate(std::cmp::min(next_work.len(), 10000));
        std::mem::swap(&mut work, &mut next_work);
    }
    best
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
    let blueprints = parse_blueprints(&input)?;
    let minutes = match args.mode {
        Mode::Part1 => 24,
        Mode::Part2 => 32,
    };
    let geodes = blueprints.par_iter().map(|blueprint| {
        let start = std::time::Instant::now();
        log::debug!("about to start simulating {:?}", blueprint);
        let geodes = simulate_with(blueprint, Inventory::new(), minutes);
        log::info!(
            "best score for {} {} (in {:?})",
            blueprint.id,
            geodes,
            start.elapsed()
        );
        (blueprint, geodes)
    });
    if args.mode == Mode::Part1 {
        let total_score: u16 = geodes.map(|(b, g)| b.id * g).sum();
        println!("{}", total_score);
    } else {
        let total_score: u16 = geodes.map(|(_, g)| g).take(3).product();
        println!("{}", total_score);
    }
    Ok(())
}

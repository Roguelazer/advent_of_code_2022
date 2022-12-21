use std::cmp::max;
use std::collections::{BTreeMap, HashMap};

use bit_set::BitSet;
use clap::{Parser, ValueEnum};
use nom::{
    bytes::complete::tag,
    character,
    combinator::{map_res, opt},
    multi::separated_list1,
    sequence::preceded,
    IResult,
};
use petgraph::algo::floyd_warshall::floyd_warshall;
use petgraph::graph::{DiGraph, NodeIndex};

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

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash)]
struct ValveName([u8; 2]);

impl ValveName {
    #[allow(dead_code)]
    fn ordinal(&self) -> usize {
        (self.0[0] - b'A') as usize * 26 + (self.0[1] - b'A') as usize
    }
}

impl TryFrom<&str> for ValveName {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            anyhow::bail!("invalid valve name");
        }
        let mut cs = value.bytes();
        Ok(Self([cs.next().unwrap(), cs.next().unwrap()]))
    }
}

impl std::fmt::Debug for ValveName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", char::from(self.0[0]), char::from(self.0[1]))
    }
}

impl std::fmt::Display for ValveName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", char::from(self.0[0]), char::from(self.0[1]))
    }
}

#[derive(Debug)]
struct Line {
    valve: ValveName,
    flow_rate: u64,
    neighbors: Vec<ValveName>,
}

impl Line {
    fn parse(s: &str) -> IResult<&str, Self> {
        map_res(
            nom::sequence::tuple((
                preceded(tag("Valve "), character::complete::alpha1::<&str, _>),
                preceded(tag(" has flow rate="), character::complete::u64),
                preceded(
                    nom::sequence::tuple((
                        nom::sequence::tuple((
                            tag("; tunnel"),
                            opt(tag("s")),
                            tag(" lead"),
                            opt(tag("s")),
                            tag(" to valve"),
                        )),
                        opt(tag("s")),
                        tag(" "),
                    )),
                    separated_list1(tag(", "), character::complete::alpha1),
                ),
            )),
            |(valve, flow_rate, neighbors)| -> anyhow::Result<Line> {
                Ok(Line {
                    valve: ValveName::try_from(valve)?,
                    flow_rate,
                    neighbors: neighbors
                        .into_iter()
                        .map(ValveName::try_from)
                        .collect::<anyhow::Result<Vec<ValveName>>>()?,
                })
            },
        )(s)
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Scene {
    graph: DiGraph<ValveName, ()>,
    nodes: BTreeMap<ValveName, NodeIndex>,
    openable_valves: BTreeMap<ValveName, u64>,
}

impl Scene {
    fn parse(s: &str) -> anyhow::Result<Self> {
        let (remaining, lines) = separated_list1(tag("\n"), Line::parse)(s)
            .map_err(|e| anyhow::anyhow!("error parsing: {:?}", e))?;
        if !remaining.trim().is_empty() {
            anyhow::bail!("unparsed input {:?}", remaining);
        }
        let mut graph = DiGraph::new();
        let mut nodes = BTreeMap::new();
        let mut openable_valves = BTreeMap::new();

        for line in lines {
            if line.flow_rate > 0 {
                openable_valves.insert(line.valve, line.flow_rate);
            }
            let this = *nodes
                .entry(line.valve)
                .or_insert_with(|| graph.add_node(line.valve));
            for neighbor in line.neighbors {
                let that = *nodes
                    .entry(neighbor)
                    .or_insert_with(|| graph.add_node(neighbor));
                graph.add_edge(this, that, ());
            }
        }
        Ok(Scene {
            graph,
            nodes,
            openable_valves,
        })
    }

    fn find_best_rec(
        &self,
        state: State,
        context: &mut Context,
        is_part2: bool,
        depth: usize,
    ) -> u64 {
        let prefix = " ".repeat(depth);
        log::debug!(
            "{0} at {1} [t={2}, d={3}]",
            prefix,
            state.position,
            state.remaining,
            depth,
        );
        if let Some(v) = context.memo.get(&state) {
            return *v;
        }
        let value = if state.remaining == 0 {
            if is_part2 {
                let mut new_state = state.clone();
                new_state.remaining = 26;
                new_state.position = ValveName::try_from("AA").unwrap();
                new_state.is_part2 = false;
                self.find_best_rec(new_state, context, false, depth + 1)
            } else {
                0
            }
        } else if context.is_done(&state) {
            0
        } else {
            let mut res = 0;

            if let Some(my_flow_rate) = self.openable_valves.get(&state.position) {
                if state.can_open(&state.position) {
                    let this_contribution = (state.remaining - 1) as u64 * my_flow_rate;
                    let mut next = state.next();
                    next.open(&state.position);
                    log::debug!("{0} opening {1}", prefix, state.position);
                    res = max(
                        res,
                        this_contribution + self.find_best_rec(next, context, is_part2, depth + 1),
                    );
                }
            }
            for item in context.useful_valves.clone() {
                if item == state.position {
                    continue;
                }
                if !state.can_open(&item) {
                    continue;
                }
                let distance = context.distances[&(state.position, item)];
                if distance <= state.remaining {
                    let mut next = state.clone();
                    next.position = item;
                    next.remaining -= distance;
                    res = max(res, self.find_best_rec(next, context, is_part2, depth + 1))
                }
            }
            res
        };
        context.memo.insert(state, value);
        value
    }

    fn find_best(&self, is_part2: bool) -> u64 {
        let start = ValveName::try_from("AA").unwrap();
        let mut useful_valves = self
            .openable_valves
            .iter()
            .filter(|(_, fr)| **fr > 0)
            .map(|s| s.0)
            .cloned()
            .collect::<Vec<_>>();
        useful_valves.push(start);
        let mut context = Context::build(&self.graph, useful_valves);
        let state = State::initial(start, is_part2);
        self.find_best_rec(state, &mut context, is_part2, 0)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
struct State {
    open_valves: BitSet,
    position: ValveName,
    remaining: u32,
    is_part2: bool,
}

impl State {
    fn initial(start: ValveName, is_part2: bool) -> Self {
        let remaining = if is_part2 { 26 } else { 30 };
        State {
            open_valves: BitSet::with_capacity(64),
            position: start,
            remaining,
            is_part2,
        }
    }

    fn next(&self) -> Self {
        let mut n = self.clone();
        n.remaining -= 1;
        n
    }

    fn can_open(&self, position: &ValveName) -> bool {
        !self.open_valves.contains(position.ordinal())
    }

    fn open(&mut self, position: &ValveName) {
        let position = position.ordinal();
        debug_assert!(!self.open_valves.contains(position));
        self.open_valves.insert(position);
    }
}

#[derive(Debug)]
struct Context {
    memo: HashMap<State, u64>,
    useful_valves: Vec<ValveName>,
    useful_valves_len: usize,
    distances: BTreeMap<(ValveName, ValveName), u32>,
}

impl Context {
    fn build<E>(graph: &DiGraph<ValveName, E>, useful_valves: Vec<ValveName>) -> Self {
        let fw = floyd_warshall(graph, |_| 1).unwrap();
        let distances = fw
            .iter()
            .map(|(&(lhs_i, rhs_i), dist)| {
                let lhs = *graph.node_weight(lhs_i).unwrap();
                let rhs = *graph.node_weight(rhs_i).unwrap();
                ((lhs, rhs), *dist as u32)
            })
            .collect::<BTreeMap<_, _>>();
        Self {
            useful_valves_len: useful_valves.len(),
            useful_valves,
            distances,
            memo: HashMap::with_capacity(10000000),
        }
    }

    fn is_done(&self, state: &State) -> bool {
        state.open_valves.len() == self.useful_valves_len
    }
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
    let scene = Scene::parse(&input)?;
    let best = scene.find_best(args.mode == Mode::Part2);
    println!("{:?}", best);
    Ok(())
}

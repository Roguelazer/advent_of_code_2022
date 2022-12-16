use std::collections::{BTreeMap, BTreeSet, HashMap};

use clap::{Parser, ValueEnum};
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character,
    combinator::{map_res, opt},
    multi::separated_list1,
    sequence::preceded,
    IResult,
};
use petgraph::{
    algo::astar::astar,
    graph::{NodeIndex, UnGraph},
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

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Hash)]
struct ValveName([u8; 2]);

impl ValveName {
    fn ordinal(&self) -> usize {
        (self.0[0] - b'A') as usize * 26 + (self.0[1] - b'A') as usize
    }
}

impl TryFrom<&str> for ValveName {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.bytes().count() != 2 {
            anyhow::bail!("invalid valve name");
        }
        let mut cs = value.bytes();
        Ok(Self([cs.next().unwrap(), cs.next().unwrap()]))
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
                        .map(|n| ValveName::try_from(n))
                        .collect::<anyhow::Result<Vec<ValveName>>>()?,
                })
            },
        )(s)
    }
}

#[derive(Debug)]
struct Scene {
    graph: UnGraph<ValveName, f32>,
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
        let mut graph = UnGraph::new_undirected();
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
                graph.update_edge(this, that, 1.0);
            }
        }
        Ok(Scene {
            graph,
            nodes,
            openable_valves,
        })
    }

    fn find_best_rec(&self, state: State, context: &mut Context) -> Option<u64> {
        if let Some(v) = context.memo.get(&state) {
            return *v;
        }
        let value = if state.remaining == 0 {
            Some(state.flowed)
        } else if context.is_done(&state) {
            self.find_best_rec(state.next(), context)
        } else {
            let current_node = self.nodes[&state.position];
            let mut options = vec![];
            if let Some(flow_rate) = self.openable_valves.get(&state.position) {
                if let Some(new_state) = state.open(*flow_rate) {
                    options.push(self.find_best_rec(new_state, context)?);
                }
            }
            for neighbor in self.graph.neighbors(current_node) {
                let neighbor_name = self.graph.node_weight(neighbor).unwrap();
                options.push(self.find_best_rec(state.travel_to(*neighbor_name), context)?);
            }
            options.into_iter().max()
        };
        context.memo.insert(state, value);
        value
    }

    fn find_best(&self) -> Option<u64> {
        let useful_valves = self
            .openable_valves
            .iter()
            .filter(|(_, fr)| **fr > 0)
            .map(|s| s.0)
            .collect::<Vec<_>>();
        let uvlen = useful_valves.len();
        let mut context = Context::build(
            &self.graph,
            self.nodes.iter().permutations(2).map(|v| {
                let (v01, v02) = v[0];
                let (v11, v12) = v[1];
                ((*v01, *v02), (*v11, *v12))
            }),
            uvlen,
        );
        let state = State::initial();
        self.find_best_rec(state, &mut context)
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
struct State {
    open_valves: BTreeSet<usize>,
    position: ValveName,
    remaining: u32,
    flow_rate: u64,
    flowed: u64,
}

impl State {
    fn initial() -> Self {
        State {
            open_valves: BTreeSet::new(),
            position: ValveName::try_from("AA").unwrap(),
            remaining: 30,
            flow_rate: 0,
            flowed: 0,
        }
    }

    fn next(&self) -> Self {
        State {
            open_valves: self.open_valves.clone(),
            position: self.position,
            remaining: self.remaining - 1,
            flow_rate: self.flow_rate,
            flowed: self.flowed + self.flow_rate,
        }
    }

    fn travel_to(&self, position: ValveName) -> Self {
        let mut s = self.next();
        s.position = position;
        s
    }

    fn open(&self, rate: u64) -> Option<Self> {
        if self.open_valves.contains(&self.position.ordinal()) {
            None
        } else {
            let mut new_open_valves = self.open_valves.clone();
            new_open_valves.insert(self.position.ordinal());
            Some(State {
                open_valves: new_open_valves,
                position: self.position,
                remaining: self.remaining - 1,
                flow_rate: self.flow_rate + rate,
                flowed: self.flowed + self.flow_rate,
            })
        }
    }
}

#[derive(Debug)]
struct Context {
    paths_by_node: BTreeMap<(NodeIndex, NodeIndex), Vec<NodeIndex>>,
    paths_by_name: BTreeMap<(ValveName, ValveName), Vec<NodeIndex>>,
    memo: HashMap<State, Option<u64>>,
    useful_valves: usize,
}

impl Context {
    fn build<I>(g: &UnGraph<ValveName, f32>, i: I, useful_valves: usize) -> Self
    where
        I: Iterator<Item = ((ValveName, NodeIndex), (ValveName, NodeIndex))>,
    {
        // precompute all the paths; floyd-warshall can do this, but not in petgraph so just use A*
        let mut paths_by_node = BTreeMap::new();
        let mut paths_by_name = BTreeMap::new();
        for ((first_name, first_node), (second_name, second_node)) in i {
            let (_, mut path) = astar(g, first_node, |n| n == second_node, |_| 1, |_| 0).unwrap();
            path.remove(0);
            paths_by_node.insert((first_node, second_node), path.clone());
            paths_by_name.insert((first_name, second_name), path);
        }
        Self {
            paths_by_node,
            paths_by_name,
            useful_valves,
            memo: HashMap::new(),
        }
    }

    fn is_done(&self, state: &State) -> bool {
        state.open_valves.len() == self.useful_valves
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
    println!("{:?}", scene);
    let best = scene.find_best();
    println!("{:?}", best);
    Ok(())
}

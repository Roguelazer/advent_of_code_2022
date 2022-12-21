use std::collections::{BTreeMap, BTreeSet};

use clap::{Parser, ValueEnum};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    multi::separated_list1,
    sequence::{preceded, terminated},
    IResult,
};
use petgraph::graph::{DiGraph, NodeIndex};

type Value = f64;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

impl Operation {
    fn execute(&self, lhs: Value, rhs: Value) -> Value {
        match self {
            Self::Add => lhs + rhs,
            Self::Sub => lhs - rhs,
            Self::Mul => lhs * rhs,
            Self::Div => lhs / rhs,
        }
    }

    fn inverse(&self) -> Self {
        match self {
            Operation::Add => Operation::Sub,
            Operation::Sub => Operation::Add,
            Operation::Mul => Operation::Div,
            Operation::Div => Operation::Mul,
        }
    }
}

#[derive(Debug, Clone)]
enum Job {
    Op {
        lhs: String,
        op: Operation,
        rhs: String,
    },
    Literal(Value),
    Variable,
}

impl Job {
    fn dependencies(&self) -> Vec<&str> {
        if let Job::Op { lhs, rhs, .. } = self {
            vec![lhs.as_ref(), rhs.as_ref()]
        } else {
            vec![]
        }
    }

    fn other_side_of(&self, side: &str) -> String {
        if let Job::Op { lhs, rhs, .. } = self {
            if lhs == side {
                return rhs.clone();
            } else if rhs == side {
                return lhs.clone();
            }
        }
        panic!("invalid argument passed to other_side_of");
    }

    fn execute(&self, values: &BTreeMap<String, Value>) -> Value {
        match self {
            Job::Literal(i) => *i,
            Job::Op { lhs, op, rhs } => {
                let lhs_value = values.get(lhs).unwrap();
                let rhs_value = values.get(rhs).unwrap();
                op.execute(*lhs_value, *rhs_value)
            }
            Job::Variable => panic!("cannot evaluate a variable"),
        }
    }
}

fn parse_label(s: &str) -> IResult<&str, String> {
    map(nom::character::complete::alpha1, String::from)(s)
}

fn parse_operand(s: &str) -> IResult<&str, String> {
    parse_label(s)
}

fn parse_op(s: &str) -> IResult<&str, Operation> {
    map(nom::character::complete::one_of("*+/-"), |c| match c {
        '*' => Operation::Mul,
        '+' => Operation::Add,
        '-' => Operation::Sub,
        '/' => Operation::Div,
        _ => unreachable!(),
    })(s)
}

fn parse_job(s: &str) -> IResult<&str, (String, Job)> {
    nom::sequence::tuple((
        parse_label,
        preceded(
            tag(": "),
            alt((
                map(nom::character::complete::i64, |i| Job::Literal(i as Value)),
                map(
                    nom::sequence::tuple((
                        terminated(parse_operand, tag(" ")),
                        parse_op,
                        preceded(tag(" "), parse_operand),
                    )),
                    |(lhs, op, rhs)| Job::Op { lhs, op, rhs },
                ),
            )),
        ),
    ))(s)
}

fn parse_jobs(s: &str) -> anyhow::Result<BTreeMap<String, Job>> {
    let (res, jobs) = separated_list1(tag("\n"), parse_job)(s)
        .map_err(|e| anyhow::anyhow!("unable to parse: {:?}", e))?;
    if !res.trim().is_empty() {
        anyhow::bail!("unhandled input {:?}", res);
    }
    Ok(jobs.into_iter().collect())
}

struct Evaluator {
    jobs: BTreeMap<String, Job>,
    nodes: BTreeMap<String, NodeIndex>,
    graph: DiGraph<String, ()>,
    values: BTreeMap<String, Value>,
}

impl Evaluator {
    fn new(jobs: BTreeMap<String, Job>) -> Self {
        let mut nodes: BTreeMap<String, _> = BTreeMap::new();
        let mut graph: DiGraph<String, ()> = DiGraph::new();
        for (key, job) in jobs.iter() {
            let outer_node = *nodes
                .entry(key.clone())
                .or_insert_with(|| graph.add_node(key.to_owned()));
            for dependency in job.dependencies().into_iter() {
                let inner_node = *nodes
                    .entry(dependency.to_string())
                    .or_insert_with(|| graph.add_node(dependency.to_owned()));
                graph.add_edge(outer_node, inner_node, ());
            }
        }
        Self {
            jobs,
            nodes,
            graph,
            values: BTreeMap::new(),
        }
    }

    fn partially_evaluate(&mut self, target_node: NodeIndex) -> Value {
        let mut visitor = petgraph::visit::DfsPostOrder::new(&self.graph, target_node);
        let label = self.graph.node_weight(target_node).unwrap().clone();
        while let Some(item) = visitor.next(&self.graph) {
            let label = self.graph.node_weight(item).unwrap();
            let job = self.jobs.get(label).unwrap();
            let value = job.execute(&self.values);
            self.values.insert(label.clone(), value);
        }
        *self.values.get(&label).unwrap()
    }

    fn partially_evaluate_operand(&mut self, operand: &str) -> Value {
        let node = self.nodes.get(operand).unwrap();
        self.partially_evaluate(*node)
    }

    fn evaluate_part1(&mut self, target: &str) -> Value {
        let target_node = self.nodes.get(target).unwrap();
        self.partially_evaluate(*target_node)
    }

    fn evaluate_part2(&mut self) -> Value {
        self.jobs.insert("humn".to_owned(), Job::Variable);
        let us = self.nodes.get("humn").unwrap();
        let root = self.nodes.get("root").unwrap();
        // first, find out which side of <<root>> we are on, and evaluate the other side
        let (_, path_to_root) =
            petgraph::algo::astar(&self.graph, *root, |f| f == *us, |_| 1, |_| 0).unwrap();
        let this_side = self.graph.node_weight(path_to_root[1]).unwrap();
        let other_side = self.jobs["root"].other_side_of(this_side);
        let mut value = self.partially_evaluate_operand(&other_side);
        // now, walk down the graph from root, inverting each operation as we go
        log::debug!("other side of root ({:?}) is {}", other_side, value);
        let nodes_in_this_path = path_to_root
            .iter()
            .map(|n| self.graph.node_weight(*n).unwrap().to_owned())
            .collect::<BTreeSet<String>>();
        for item in path_to_root.into_iter().skip(1) {
            let node_label = self.graph.node_weight(item).unwrap().to_owned();
            let job = self.jobs.get(&node_label).unwrap().clone();
            log::debug!("walking through {} ({:?})", node_label, job);
            if let Job::Op { lhs, op, rhs } = job {
                if nodes_in_this_path.contains(&lhs) {
                    let rhs_value = self.partially_evaluate_operand(&rhs);
                    value = op.inverse().execute(value, rhs_value);
                } else if nodes_in_this_path.contains(&rhs) {
                    let lhs_value = self.partially_evaluate_operand(&lhs);
                    // this bit is tricky because of stupid commutativity rules
                    value = match op {
                        Operation::Mul | Operation::Add => op.inverse().execute(value, lhs_value),
                        Operation::Sub | Operation::Div => op.execute(lhs_value, value),
                    };
                } else {
                    panic!("wut");
                }
            } else if let Job::Variable = job {
                return value;
            }
            log::debug!("value = {}", value);
        }
        panic!("failed to find solution");
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
    let start = std::time::Instant::now();
    let jobs = parse_jobs(&input)?;
    let mut e = Evaluator::new(jobs);
    let res = match args.mode {
        Mode::Part1 => e.evaluate_part1("root"),
        Mode::Part2 => e.evaluate_part2(),
    };
    log::info!("computed result in {:?}", start.elapsed());
    println!("{}", res);
    Ok(())
}

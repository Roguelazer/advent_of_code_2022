use std::collections::BTreeMap;
use std::str::FromStr;

use clap::{Parser, ValueEnum};
use petgraph::graph::DiGraph;

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
}

#[derive(Debug)]
struct Grid {
    cells: Vec<Vec<u8>>,
    graph_indices: BTreeMap<(usize, usize), petgraph::graph::NodeIndex>,
    graph: DiGraph<(), ()>,
    start_coordinate: (usize, usize),
    end_coordinate: (usize, usize),
}

impl Grid {
    fn shortest_path(&self) -> usize {
        let start_index = *self.graph_indices.get(&self.start_coordinate).unwrap();
        let end_index = *self.graph_indices.get(&self.end_coordinate).unwrap();
        let dk =
            petgraph::algo::dijkstra::dijkstra(&self.graph, start_index, Some(end_index), |_| 1);
        *dk.get(&end_index).unwrap()
    }

    fn shortest_paths_any_start(&self) -> usize {
        let mut rev_graph = self.graph.clone();
        rev_graph.reverse();
        let end_index = *self.graph_indices.get(&self.end_coordinate).unwrap();
        let dres = petgraph::algo::dijkstra::dijkstra(&rev_graph, end_index, None, |_| 1);
        *self
            .graph_indices
            .iter()
            .filter(|((x, y), _i)| self.cells[*y][*x] == 0)
            .filter_map(|(_xy, i)| dres.get(i))
            .min()
            .unwrap()
    }
}

impl FromStr for Grid {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut start_coordinate = None;
        let mut end_coordinate = None;
        let cells = s
            .split('\n')
            .filter(|r| r.len() > 0)
            .enumerate()
            .map(|(y, row)| {
                row.as_bytes()
                    .iter()
                    .enumerate()
                    .map(|(x, cell)| {
                        let height = match cell {
                            b'S' => {
                                start_coordinate = Some((x, y));
                                0
                            }
                            b'E' => {
                                end_coordinate = Some((x, y));
                                25
                            }
                            other if *other >= b'a' && *other <= b'z' => (*other - b'a') as u8,
                            other => {
                                anyhow::bail!("invalid char {}", other);
                            }
                        };
                        Ok(height)
                    })
                    .collect::<Result<Vec<u8>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let height = cells.len();
        let width = cells[0].len();
        let mut graph = DiGraph::new();
        let mut coordinates = BTreeMap::new();
        cells.iter().enumerate().for_each(|(y, row)| {
            row.iter().enumerate().for_each(|(x, _)| {
                coordinates.insert((x, y), graph.add_node(()));
            })
        });
        for ((x, y), this_node) in coordinates.iter() {
            let mut neighbors = vec![];
            let x = *x;
            let y = *y;
            if x == 0 {
                neighbors.push((x + 1, y));
            } else if x + 1 == width {
                neighbors.push((x - 1, y));
            } else {
                neighbors.push((x + 1, y));
                neighbors.push((x - 1, y));
            }

            if y == 0 {
                neighbors.push((x, y + 1));
            } else if y + 1 == height {
                neighbors.push((x, y - 1));
            } else {
                neighbors.push((x, y + 1));
                neighbors.push((x, y - 1));
            }
            for neighbor in neighbors.into_iter() {
                let neighbor_val = cells[neighbor.1][neighbor.0];
                let my_val = cells[y][x];
                if neighbor_val <= my_val + 1 {
                    graph.add_edge(*this_node, *coordinates.get(&neighbor).unwrap(), ());
                }
            }
        }
        Ok(Grid {
            graph_indices: coordinates,
            cells,
            graph,
            start_coordinate: start_coordinate
                .ok_or_else(|| anyhow::anyhow!("no start coordinate"))?,
            end_coordinate: end_coordinate.ok_or_else(|| anyhow::anyhow!("no end coordinate"))?,
        })
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    let grid = input.parse::<Grid>()?;
    let res = match args.mode {
        Mode::Part1 => grid.shortest_path(),
        Mode::Part2 => grid.shortest_paths_any_start(),
    };
    println!("{:?}", res);
    Ok(())
}

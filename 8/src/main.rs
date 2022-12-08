use std::io::BufRead;

use clap::{Parser, ValueEnum};
use nonempty::NonEmpty;

type TreeHeight = u8;

#[derive(Debug)]
struct Scene {
    rows: NonEmpty<NonEmpty<TreeHeight>>,
    cols: NonEmpty<NonEmpty<TreeHeight>>,
}

impl Scene {
    fn from_reader<R: BufRead>(r: R) -> anyhow::Result<Self> {
        let rows = NonEmpty::collect(
            r.lines()
                .filter_map(Result::ok)
                .filter_map(|line| NonEmpty::collect(line.as_bytes().iter().map(|b| b - b'0'))),
        )
        .ok_or_else(|| anyhow::anyhow!("no lines found"))?;
        let cols = NonEmpty::collect(
            (0..rows.first().len())
                .filter_map(|i| NonEmpty::collect(rows.iter().map(|row| row[i]))),
        )
        .unwrap();
        Ok(Self { rows, cols })
    }

    fn num_visible(&self) -> usize {
        let width = self.rows.first().len() - 1;
        let height = self.rows.len() - 1;
        self.rows
            .iter()
            .enumerate()
            .map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .map(|(x, cell)| {
                        if y == 0 || x == 0 || y == height || x == width {
                            1
                        } else {
                            // check the row
                            let visible_to_left = row.iter().take(x).all(|i| *i < *cell);
                            let visible_to_right = row.iter().skip(x + 1).all(|i| *i < *cell);
                            let visible_above = self.cols[x].iter().take(y).all(|i| *i < *cell);
                            let visible_below = self.cols[x].iter().skip(y + 1).all(|i| *i < *cell);
                            if visible_to_left || visible_to_right || visible_above || visible_below
                            {
                                1
                            } else {
                                0
                            }
                        }
                    })
                    .sum::<usize>()
            })
            .sum()
    }

    fn max_scenic_score(&self) -> usize {
        self.rows
            .iter()
            .enumerate()
            .filter_map(|(y, row)| {
                row.iter()
                    .enumerate()
                    .map(|(x, cell)| {
                        let col = &self.cols[x];
                        let up_score = scenic_score_helper(col.iter().take(y).rev(), *cell);
                        let down_score = scenic_score_helper(col.iter().skip(y + 1), *cell);
                        let left_score = scenic_score_helper(row.iter().take(x).rev(), *cell);
                        let right_score = scenic_score_helper(row.iter().skip(x + 1), *cell);
                        let combined = left_score * right_score * up_score * down_score;
                        combined
                    })
                    .max()
            })
            .max()
            .unwrap()
    }
}

fn scenic_score_helper<'a, I: Iterator<Item = &'a u8>>(iter: I, height: u8) -> usize {
    let mut score = 0;
    for tree in iter {
        score += 1;
        if *tree >= height {
            break;
        }
    }
    score
}

#[derive(ValueEnum, Debug, PartialEq, Eq, Clone, Copy)]
enum Mode {
    Part1,
    Part2,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_enum)]
    mode: Mode,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let scene = Scene::from_reader(&mut handle)?;
    if args.mode == Mode::Part1 {
        println!("{}", scene.num_visible());
    } else {
        println!("{}", scene.max_scenic_score());
    }
    Ok(())
}

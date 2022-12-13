use clap::{Parser, ValueEnum};
use std::cmp::{Ordering, PartialOrd};
use std::fmt::{Display, Formatter};

use itertools::{EitherOrBoth, Itertools};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::map,
    multi::{many1, separated_list0, separated_list1},
    sequence::{delimited, pair, terminated},
    IResult,
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

#[derive(Debug, Clone, PartialEq, Eq, Ord)]
enum Packet {
    Number(i32),
    List(Vec<Packet>),
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::List(p) => {
                write!(f, "[")?;
                for (i, item) in p.iter().enumerate() {
                    if i < p.len() - 1 {
                        write!(f, "{}, ", item)
                    } else {
                        write!(f, "{}", item)
                    }?
                }
                write!(f, "]")
            }
        }
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Packet) -> Option<Ordering> {
        match (self, other) {
            (Packet::Number(lhs), Packet::Number(rhs)) => Some(lhs.cmp(rhs)),
            (Packet::List(lhs), Packet::List(rhs)) => {
                for item in lhs.iter().zip_longest(rhs.iter()) {
                    match item {
                        EitherOrBoth::Left(_) => return Some(Ordering::Greater),
                        EitherOrBoth::Right(_) => return Some(Ordering::Less),
                        EitherOrBoth::Both(l, r) => match l.partial_cmp(r) {
                            s @ Some(Ordering::Less) => return s,
                            s @ Some(Ordering::Greater) => return s,
                            _ => {}
                        },
                    }
                }
                Some(Ordering::Equal)
            }
            (lhs, rhs @ Packet::List(_)) => Packet::List(vec![lhs.clone()]).partial_cmp(&rhs),
            (lhs @ Packet::List(_), rhs) => lhs.partial_cmp(&Packet::List(vec![rhs.clone()])),
        }
    }
}

fn parse_packet(s: &str) -> IResult<&str, Packet> {
    alt((
        map(nom::character::complete::i32, Packet::Number),
        map(
            delimited(tag("["), separated_list0(tag(","), parse_packet), tag("]")),
            Packet::List,
        ),
    ))(s)
}

fn parse_packets(s: &str) -> IResult<&str, Vec<(Packet, Packet)>> {
    separated_list1(
        tag("\n\n"),
        pair(terminated(parse_packet, tag("\n")), parse_packet),
    )(s)
}

fn parse_packet_pairs(s: &str) -> anyhow::Result<Vec<(Packet, Packet)>> {
    let (remainder, packets) =
        parse_packets(s).map_err(|e| anyhow::anyhow!("error parsing: {:?}", e))?;
    if !remainder.trim().is_empty() {
        anyhow::bail!("unconsumed input {:?}", remainder);
    }
    Ok(packets)
}

fn parse_all_packets(s: &str) -> anyhow::Result<Vec<Packet>> {
    let (remainder, packets) = separated_list1(many1(tag("\n")), parse_packet)(s)
        .map_err(|e| anyhow::anyhow!("error parsing {:?}", e))?;
    if !remainder.trim().is_empty() {
        anyhow::bail!("unconsumed input {:?}", remainder);
    }
    Ok(packets)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let stdin = std::io::stdin();
    let input = std::io::read_to_string(stdin)?;
    if args.mode == Mode::Part1 {
        let ok_indices = parse_packet_pairs(&input)?
            .into_iter()
            .enumerate()
            .filter_map(|(i, (lhs, rhs))| {
                if lhs <= rhs {
                    if args.verbose {
                        println!("OK:  {} ⇐ {}", lhs, rhs);
                    }
                    Some(i + 1)
                } else {
                    if args.verbose {
                        println!("BAD: {} > {}", lhs, rhs);
                    }
                    None
                }
            })
            .sum::<usize>();
        println!("{}", ok_indices);
    } else {
        let delimiters = &[
            Packet::List(vec![Packet::List(vec![Packet::Number(2)])]),
            Packet::List(vec![Packet::List(vec![Packet::Number(6)])]),
        ];
        let mut all_packets = parse_all_packets(&input)?;
        all_packets.push(delimiters[0].clone());
        all_packets.push(delimiters[1].clone());
        all_packets.sort();
        if args.verbose {
            for packet in &all_packets {
                println!("{}", packet);
            }
        }
        let decoder_key = delimiters
            .iter()
            .map(|d| all_packets.iter().position(|i| i == d).unwrap() + 1)
            .product::<usize>();
        println!("{}", decoder_key);
    }
    Ok(())
}

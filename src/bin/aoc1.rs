use std::io::BufRead;

type ElfId = u32;

#[derive(Debug)]
struct Best<const N: usize> {
    inner: [Option<(ElfId, u64)>; N],
}

impl<const N: usize> Best<N> {
    fn new() -> Self {
        Best { inner: [None; N] }
    }

    fn handle(&mut self, elf_id: ElfId, calories: u64) {
        let insert_index = self.inner.iter().position(|i| match i {
            None => true,
            Some((_, v)) => *v <= calories,
        });
        if let Some(index) = insert_index {
            // backshift all the other elements
            if index + 1 < self.inner.len() {
                for source in (index..(self.inner.len() - 1)).rev() {
                    self.inner[source + 1] = self.inner[source]
                }
            }
            self.inner[index] = Some((elf_id, calories));
        }
    }

    fn total(&self) -> u64 {
        self.inner
            .iter()
            .map(|i| match i {
                None => 0,
                Some((_, v)) => *v,
            })
            .sum()
    }
}

fn main() {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = String::new();
    let mut best = Best::<3>::new();
    let mut acc = 0u64;
    let mut current = 1u32;

    while let Ok(sz) = handle.read_line(&mut buffer) {
        if sz == 0 {
            break;
        }
        let val = buffer.trim();
        if val.is_empty() {
            best.handle(current, acc);
            acc = 0;
            current += 1;
        } else {
            acc += val.parse::<u64>().unwrap();
        }
        buffer.clear();
    }
    best.handle(current, acc);
    println!("{:?}", best.total());
}

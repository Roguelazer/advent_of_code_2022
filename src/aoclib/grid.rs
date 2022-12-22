use std::cmp::{max, min};
use std::fmt;

use super::point::Point;

type Index = i64;

pub trait HasEmpty {
    fn empty_value() -> Self;
}

#[derive(Debug)]
pub struct DenseGrid<V: Clone + fmt::Debug> {
    min_x: Index,
    min_y: Index,
    max_x: Index,
    max_y: Index,
    width: usize,
    height: usize,
    cells: Vec<V>,
}

impl<V: Clone + fmt::Debug + HasEmpty> DenseGrid<V> {
    pub fn new(upper_left: Point<Index>, lower_right: Point<Index>) -> Self {
        Self::new_with(upper_left, lower_right, V::empty_value())
    }
}

impl<V: Clone + fmt::Debug> DenseGrid<V> {
    pub fn new_with(upper_left: Point<Index>, lower_right: Point<Index>, empty_value: V) -> Self {
        let min_x = min(upper_left.x, lower_right.x);
        let max_x = max(upper_left.x, lower_right.x);
        let min_y = min(upper_left.y, lower_right.y);
        let max_y = max(upper_left.y, lower_right.y);
        let width = 1 + max_x.abs_diff(min_x) as usize;
        let height = 1 + max_y.abs_diff(min_y) as usize;
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            width,
            height,
            cells: vec![empty_value; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn size(&self) -> usize {
        self.width * self.height
    }

    /// Get a value by coordinate. Returns None if the coordinate is out-of-bounds.
    pub fn get(&self, coordinate: Point<Index>) -> Option<V> {
        let index = self.index_for(coordinate)?;
        self.cells.get(index).cloned()
    }

    /// Set a value by coordinate. Returns None if the coordinate is out-of-bounds.
    pub fn set(&mut self, coordinate: Point<Index>, value: V) -> Option<()> {
        let index = self.index_for(coordinate)?;
        self.cells[index] = value;
        Some(())
    }

    pub fn contains(&self, coordinate: Point<Index>) -> bool {
        coordinate.x >= self.min_x
            && coordinate.x <= self.max_x
            && coordinate.y >= self.min_y
            && coordinate.y <= self.max_y
    }

    pub fn dump_with<F: Fn(&V) -> char>(&self, f: F) {
        for y in self.min_y..=self.max_y {
            let cells = (self.min_x..=self.max_x)
                .map(|x| {
                    let coordinate = Point::new(x, y);
                    f(&self[coordinate])
                })
                .collect::<String>();
            println!("{}", cells);
        }
    }

    fn index_for(&self, coordinate: Point<Index>) -> Option<usize> {
        if coordinate.x < self.min_x
            || coordinate.x > self.max_x
            || coordinate.y < self.min_y
            || coordinate.y > self.max_y
        {
            None
        } else {
            let row = coordinate.y.abs_diff(self.min_y) as usize * self.width;
            let col = coordinate.x.abs_diff(self.min_x) as usize;
            Some(row + col)
        }
    }
}

impl<V: Clone + std::fmt::Debug> std::ops::Index<Point<Index>> for DenseGrid<V> {
    type Output = V;

    fn index(&self, coordinate: Point<Index>) -> &Self::Output {
        let index = self.index_for(coordinate).unwrap();
        self.cells.get(index).unwrap()
    }
}

impl<V: Clone + std::fmt::Debug> std::ops::IndexMut<Point<Index>> for DenseGrid<V> {
    fn index_mut(&mut self, coordinate: Point<Index>) -> &mut Self::Output {
        let index = self.index_for(coordinate).unwrap();
        self.cells.get_mut(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{DenseGrid, Point};

    #[test]
    fn test_small() {
        let origin = Point { x: 10, y: 10 };
        let mut g = DenseGrid::new_with(origin, origin, 0u8);
        assert_eq!(g.size(), 1);
        assert_eq!(g.get(Point { x: 0, y: 0 }), None);
        assert_eq!(g.get(origin), Some(0u8));
        g.set(origin, 255u8);
        assert_eq!(g.get(origin), Some(255u8));
    }

    #[test]
    fn test_basic() {
        let mut g = DenseGrid::new_with(Point { x: 0, y: 0 }, Point { x: 99, y: 99 }, 0u8);
        assert_eq!(g.size(), 10000);
        assert_eq!(g[Point { x: 50, y: 50 }], 0);
        g[Point { x: 50, y: 50 }] = 4;
        assert_eq!(g[Point { x: 49, y: 50 }], 0);
        assert_eq!(g[Point { x: 50, y: 50 }], 4);
    }
}

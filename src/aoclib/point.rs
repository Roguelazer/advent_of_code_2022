use std::cmp::Ordering;
use std::fmt;

pub trait DimVal:
    num_traits::Signed
    + num_traits::ToPrimitive
    + std::cmp::Ord
    + std::cmp::Eq
    + Clone
    + Copy
    + std::fmt::Display
    + std::fmt::Debug
{
}

impl<
        S: num_traits::Signed
            + num_traits::ToPrimitive
            + std::cmp::Ord
            + std::cmp::Eq
            + Clone
            + Copy
            + std::fmt::Display
            + std::fmt::Debug,
    > DimVal for S
{
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Point<I: DimVal = i64> {
    pub x: I,
    pub y: I,
}

impl<I: DimVal> Point<I> {
    pub const fn new(x: I, y: I) -> Self {
        Point { x, y }
    }

    pub fn transpose(&self) -> Self {
        Point::new(self.y, self.x)
    }

    pub fn line_to(&self, other: Point<I>) -> impl Iterator<Item = Point<I>> {
        LineToIter::new(*self, other)
    }

    pub fn manhattan_distance_to(&self, other: Point<I>) -> usize {
        ((self.x - other.x).abs() + (self.y - other.y).abs())
            .to_u64()
            .unwrap() as usize
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<I: DimVal> std::ops::Add for Point<I> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl<I: DimVal> std::ops::Mul<I> for Point<I> {
    type Output = Self;

    fn mul(self, other: I) -> Self {
        Point {
            x: self.x * other,
            y: self.y + other,
        }
    }
}

#[derive(Debug)]
struct LineToIter<I: DimVal> {
    start: Point<I>,
    end: Point<I>,
    direction: Point<I>,
    done: bool,
}

impl<I: DimVal> LineToIter<I> {
    fn new(start: Point<I>, end: Point<I>) -> Self {
        debug_assert!(start.x == end.x || start.y == end.y);
        let zero = I::zero();
        let one = I::one();
        let neg_one = one.neg();
        let dir = match (start.x.cmp(&end.x), start.y.cmp(&end.y)) {
            (Ordering::Less, _) => Point { x: one, y: zero },
            (Ordering::Greater, _) => Point {
                x: neg_one,
                y: zero,
            },
            (Ordering::Equal, Ordering::Less) => Point { x: zero, y: one },
            (Ordering::Equal, Ordering::Greater) => Point {
                x: zero,
                y: neg_one,
            },
            (Ordering::Equal, Ordering::Equal) => unreachable!(),
        };
        Self {
            start,
            end,
            direction: dir,
            done: false,
        }
    }
}

impl<I: DimVal> Iterator for LineToIter<I> {
    type Item = Point<I>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let current = self.start;
        if self.start == self.end {
            self.done = true
        }
        self.start = self.start + self.direction;
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::Point;

    #[test]
    fn transpose() {
        assert_eq!(Point::new(1, 0).transpose(), Point::new(0, 1));
        assert_eq!(Point::new(0, 1).transpose(), Point::new(1, 0));
    }

    #[test]
    fn test_line_to_y() {
        let start = Point::new(0, 0);
        let end = Point::new(0, 10);
        let points = start.line_to(end).collect::<Vec<_>>();
        assert_eq!(points.len(), 11);
        assert_eq!(points[0], Point::new(0, 0));
        assert_eq!(points[5], Point::new(0, 5));
        assert_eq!(points[10], Point::new(0, 10));

        let mut other_dir = end.line_to(start).collect::<Vec<_>>();
        other_dir.reverse();
        assert_eq!(points, other_dir);
    }

    #[test]
    fn test_line_to_x() {
        let start = Point::new(0, 0);
        let end = Point::new(10, 0);
        let points = start.line_to(end).collect::<Vec<_>>();
        assert_eq!(points.len(), 11);
        assert_eq!(points[0], Point::new(0, 0));
        assert_eq!(points[5], Point::new(5, 0));
        assert_eq!(points[10], Point::new(10, 0));
    }
}

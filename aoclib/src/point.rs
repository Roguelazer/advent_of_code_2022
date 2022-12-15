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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point<I: DimVal = i64> {
    pub x: I,
    pub y: I,
}

impl<I: DimVal> Point<I> {
    pub fn new(x: I, y: I) -> Self {
        Point { x, y }
    }

    pub fn line_to(&self, other: Point<I>) -> impl Iterator<Item = Point<I>> {
        LineToIter::new(self.clone(), other)
    }

    pub fn manhattan_distance_to(&self, other: Point<I>) -> usize {
        let val = ((self.x - other.x).abs() + (self.y - other.y).abs())
            .to_u64()
            .unwrap() as usize;
        val
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
        let neg_one = one.clone().neg();
        let dir = if start.x == end.x {
            if start.y < end.y {
                Point { x: zero, y: one }
            } else {
                Point {
                    x: zero,
                    y: neg_one,
                }
            }
        } else {
            if start.x < end.x {
                Point { x: one, y: zero }
            } else {
                Point {
                    x: neg_one,
                    y: zero,
                }
            }
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

pub type Coord = (usize, usize);
pub type Color = usize;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Point {
    pub coord: Coord,
    pub color: Color,
}

impl Point {
    pub fn new(coord: Coord, color: Color) -> Self {
        Self { coord, color }
    }
}

pub fn find_other_point(points: &Vec<Point>, point: &Point) -> Point {
    for p in points {
        if p.coord != point.coord && p.color == point.color {
            return p.clone();
        }
    }

    Point::default()
}

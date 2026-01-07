use std::{fs, io};

use crate::point::{Color, Coord, Point};

const P_UL: &str = "╯";
const P_UR: &str = "╰";
const P_UD: &str = "│";
const P_DL: &str = "╮";
const P_DR: &str = "╭";
const P_LR: &str = "─";
const P_SP: &str = " ";

#[derive(Clone)]
pub struct Grid {
    pub data: Vec<Vec<Option<Color>>>,
}

impl Grid {
    pub fn from(filename: &String) -> io::Result<Self> {
        Ok(Self {
            data: fs::read_to_string(filename)?
                .split("\n")
                .filter(|s| s.len() >= 2)
                .map(|line| {
                    line.chars()
                        .map(|c| c.to_digit(10).map(|i| i as usize))
                        .collect()
                })
                .collect(),
        })
    }

    pub fn new(dims: &(usize, usize)) -> Self {
        Self {
            data: vec![vec![None; dims.1]; dims.0],
        }
    }

    pub fn get_dims(&self) -> (usize, usize) {
        (self.data.len(), self.data[0].len())
    }

    pub fn get(&self, coord: &Coord) -> Option<Color> {
        self.data[coord.0][coord.1]
    }

    pub fn set(&mut self, coord: &Coord, val: Option<Color>) {
        self.data[coord.0][coord.1] = val;
    }

    pub fn find_points(&self) -> Vec<Point> {
        let (nb_rows, nb_cols) = self.get_dims();

        let mut points = vec![];

        for row in 0..nb_rows {
            for col in 0..nb_cols {
                if let Some(color) = self.get(&(row, col)) {
                    points.push(Point::new((row, col), color));
                }
            }
        }

        return points;
    }

    pub fn to_string(&self) -> String {
        let (nb_rows, nb_cols) = self.get_dims();

        let mut res = "".to_string();

        for row in 0..nb_rows {
            for col in 0..nb_cols {
                let point = self.get(&(row, col));
                if let Some(n) = point {
                    if row != 0
                        && col != 0
                        && point == self.get(&(row - 1, col))
                        && point == self.get(&(row, col - 1))
                    {
                        res += P_UL
                    } else if row != 0
                        && col != nb_cols - 1
                        && point == self.get(&(row - 1, col))
                        && point == self.get(&(row, col + 1))
                    {
                        res += P_UR
                    } else if row != 0
                        && row != nb_rows - 1
                        && point == self.get(&(row - 1, col))
                        && point == self.get(&(row + 1, col))
                    {
                        res += P_UD
                    } else if row != nb_rows - 1
                        && col != 0
                        && point == self.get(&(row + 1, col))
                        && point == self.get(&(row, col - 1))
                    {
                        res += P_DL
                    } else if row != nb_rows - 1
                        && col != nb_cols - 1
                        && point == self.get(&(row + 1, col))
                        && point == self.get(&(row, col + 1))
                    {
                        res += P_DR
                    } else if col != 0
                        && col != nb_cols - 1
                        && point == self.get(&(row, col - 1))
                        && point == self.get(&(row, col + 1))
                    {
                        res += P_LR
                    } else {
                        res += &n.to_string()
                    }
                } else {
                    res += P_SP
                };
            }

            if row != nb_rows - 1 {
                res += "\n";
            }
        }

        return res;
    }
}

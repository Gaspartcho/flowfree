use std::{env, usize};

use grid::*;
use point::*;

mod grid;
mod point;

type Path = Vec<Coord>;

fn has_arrived(coord: &Coord, goal: &Coord) -> bool {
    return coord.0.abs_diff(goal.0) + coord.1.abs_diff(goal.1) == 1;
}

fn get_neighbors(coord: &Coord, grid: &Grid) -> Vec<Coord> {
    let (nb_rows, nb_cols) = grid.get_dims();
    let &(row, col) = coord;

    let mut res = vec![];

    if row != 0 {
        res.push((row - 1, col));
    }
    if row != nb_rows - 1 {
        res.push((row + 1, col));
    }
    if col != 0 {
        res.push((row, col - 1));
    }
    if col != nb_cols - 1 {
        res.push((row, col + 1));
    }

    return res;
}

fn get_smart_neighbors(path: &Path, grid: &Grid) -> Vec<Coord> {
    let last = path.last().unwrap();
    return get_neighbors(last, grid)
        .into_iter()
        .filter(|n| {
            grid.get(n).is_none()
                && !path.contains(n)
                && get_neighbors(n, grid)
                    .iter()
                    .all(|p| (!path.contains(p)) || (p == last))
        })
        .collect();
}

fn generate_paths_rec(goal: &Coord, path: &Path, grid: &Grid) -> Vec<Path> {
    if has_arrived(path.last().unwrap(), goal) {
        let mut res = path.clone();
        res.push(goal.clone());
        return vec![res];
    }

    let mut res = vec![];

    for neighbor in get_smart_neighbors(path, grid) {
        let mut next = path.clone();
        next.push(neighbor);
        res.append(&mut generate_paths_rec(goal, &next, grid));
    }

    return res;
}

fn find_paths(begin: &Coord, end: &Coord, grid: &Grid) -> Vec<Path> {
    return generate_paths_rec(end, &vec![begin.clone()], grid);
}

fn find_all_paths(grid: &Grid) -> Vec<Vec<Path>> {
    let (nb_rows, nb_cols) = grid.get_dims();
    let points = grid.find_points();
    let nb_colors = points.len() / 2;

    let mut res = vec![];

    for i in 0..nb_colors {
        let begin = find_other_point(&points, &Point::new((nb_rows, nb_cols), i));
        let end = find_other_point(&points, &begin);
        res.push(find_paths(&begin.coord, &end.coord, &grid));
    }

    return res;
}

fn find_forced_coord(paths: &Vec<Path>, dims: &(usize, usize)) -> Vec<Coord> {
    if paths.is_empty() {
        return vec![];
    }

    let &(nb_rows, nb_cols) = dims;
    let nb_paths = paths.len();

    let mut map = vec![vec![0; nb_cols]; nb_rows];

    for path in paths {
        for &(row, col) in path {
            map[row][col] += 1;
        }
    }

    let mut res = vec![];

    for row in 0..nb_rows {
        for col in 0..nb_cols {
            if map[row][col] == nb_paths {
                res.push((row, col));
            }
        }
    }

    return res;
}

fn generate_forced_grid(paths: &Vec<Vec<Path>>, dims: &(usize, usize)) -> Grid {
    let mut grid = Grid::new(dims);

    for (color, p) in paths.iter().enumerate() {
        for coord in &find_forced_coord(p, dims) {
            grid.set(coord, Some(color));
        }
    }

    return grid;
}

fn generate_single_grid(paths: &Vec<Vec<Path>>, dims: &(usize, usize)) -> Grid {
    let nb_colors = paths.len();
    let mut grid = Grid::new(dims);

    for color in 0..nb_colors {
        for path in &paths[color] {
            for coord in path {
                grid.set(
                    coord,
                    match grid.get(coord) {
                        None => Some(color),
                        Some(c) => {
                            if c == color {
                                Some(color)
                            } else {
                                Some(usize::MAX)
                            }
                        }
                    },
                );
            }
        }
    }

    for row in 0..dims.0 {
        for col in 0..dims.1 {
            let coord = &(row, col);
            if grid.get(coord) == Some(usize::MAX) {
                grid.set(coord, None);
            }
        }
    }

    return grid;
}

fn get_single_coords(grid: &Grid, color: &Color) -> Vec<Coord> {
    let (nb_rows, nb_cols) = grid.get_dims();

    let mut res = vec![];

    for row in 0..nb_rows {
        for col in 0..nb_cols {
            let coord = (row, col);
            if grid.get(&coord) == Some(*color) {
                res.push(coord);
            }
        }
    }

    return res;
}

fn filter_paths_forced(paths: &mut Vec<Vec<Path>>, dims: &(usize, usize)) {
    let nb_colors = paths.len();
    let grid = generate_forced_grid(paths, dims);

    // println!("{}", grid.to_string());

    for color in 0..nb_colors {
        paths[color] = paths[color]
            .iter()
            .filter(|path| {
                !path
                    .iter()
                    .any(|coord| grid.get(coord).is_some_and(|c| c != color))
            })
            .map(|p| p.clone())
            .collect()
    }

    return;
}

fn filter_paths_single(paths: &mut Vec<Vec<Path>>, dims: &(usize, usize)) {
    let nb_colors = paths.len();
    let grid = generate_single_grid(paths, dims);
    let &(nb_rows, nb_cols) = dims;

    for color in 0..nb_colors {
        let single = get_single_coords(&grid, &color);
        let mut n_p = vec![];
        for path in &paths[color] {
            let mut map = vec![vec![false; nb_cols]; nb_rows];

            for coord in path {
                map[coord.0][coord.1] = true;
            }

            if single.iter().all(|coord| map[coord.0][coord.1]) {
                n_p.push(path.clone());
            }
        }

        paths[color] = n_p;
    }

    return;
}

fn prune(paths: &mut Vec<Vec<Path>>, dims: &(usize, usize)) {
    let mut nb_paths: usize = paths.iter().map(|p| p.len()).sum();
    let mut last_nb_paths = 0;

    while nb_paths != last_nb_paths {
        filter_paths_single(paths, dims);
        filter_paths_forced(paths, dims);

        last_nb_paths = nb_paths;
        nb_paths = paths.iter().map(|p| p.len()).sum();
    }

    return;
}

fn is_solved(paths: &Vec<Vec<Path>>) -> bool {
    return paths.iter().all(|p| p.len() == 1);
}

fn is_impossible(paths: &Vec<Vec<Path>>) -> bool {
    return paths.iter().any(|p| p.is_empty());
}

fn backtrack(paths: &mut Vec<Vec<Path>>, dims: &(usize, usize)) -> bool {
    let nb_colors = paths.len();

    prune(paths, dims);

    if is_solved(paths) {
        return true;
    }
    if is_impossible(paths) {
        return false;
    }

    let mut index = 0;
    while paths[index].len() == 1 {
        index += 1;
    }

    for path in &paths[index] {
        let mut n_paths = vec![];
        for color in 0..nb_colors {
            n_paths.push(if color == index {
                vec![path.clone()]
            } else {
                paths[color].clone()
            });
        }

        if backtrack(&mut n_paths, dims) {
            *paths = n_paths;
            return true;
        }
    }

    return false;
}

fn generate_final_grid(paths: &Vec<Vec<Path>>, dims: &(usize, usize)) -> Grid {
    let mut grid = Grid::new(dims);

    for (color, path) in paths.iter().enumerate() {
        for coord in &path[0] {
            grid.set(&coord, Some(color));
        }
    }

    return grid;
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || args[1] == "-h" || args[1] == "help" {
        println!("Flowfree Solver");
        println!();
        println!("Usage: flowfree [Option] [Path to Puzzle File]");
        println!();
        println!("Options:");
        println!("-h: Prints this help message and exits.");

        return;
    }

    match Grid::from(&args[1]) {
        Ok(grid) => {
            let dims = &grid.get_dims();
            let mut paths = find_all_paths(&grid);

            backtrack(&mut paths, dims);

            println!("{}", generate_final_grid(&paths, dims).to_string())
        }

        Err(e) => {
            println!("Error: could not open the file \"{}\" - {}", args[1], e);
        }
    }
}

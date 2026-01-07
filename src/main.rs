use ansi_control_codes::c0::ESC;
use std::{
    env,
    io::{self, Write},
};

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

fn generate_paths(goal: &Coord, path: &Path, grid: &Grid) -> Vec<Path> {
    if has_arrived(path.last().unwrap(), goal) {
        let mut res = path.clone();
        res.push(goal.clone());
        return vec![res];
    }

    let mut res = vec![];

    for neighbor in get_smart_neighbors(path, grid) {
        let mut next = path.clone();
        next.push(neighbor);
        res.append(&mut generate_paths(goal, &next, grid));
    }

    return res;
}

fn generate_all_paths(grid: &Grid) -> Vec<Vec<Path>> {
    let (nb_rows, nb_cols) = grid.get_dims();
    let points = grid.find_points();
    let nb_colors = points.len() / 2;

    let mut res = vec![];

    for i in 0..nb_colors {
        let begin = find_other_point(&points, &Point::new((nb_rows, nb_cols), i));
        let end = find_other_point(&points, &begin);
        res.push(generate_paths(&end.coord, &vec![begin.coord], grid));
    }

    return res;
}

fn get_paths_refs(paths: &Vec<Vec<Path>>) -> Vec<Vec<&Path>> {
    return paths.iter().map(|p| p.iter().collect()).collect();
}

fn get_forced_coord(paths: &[&Path], dims: &(usize, usize)) -> Vec<Coord> {
    if paths.is_empty() {
        return vec![];
    }

    let &(nb_rows, nb_cols) = dims;
    let nb_paths = paths.len();

    let mut map = vec![0; nb_cols * nb_rows];
    for &path in paths {
        for &(row, col) in path {
            map[col + nb_cols * row] += 1;
        }
    }

    let map = map;
    let mut res = vec![];
    for (i, elem) in map.iter().enumerate() {
        if elem == &nb_paths {
            res.push((i / nb_cols, i % nb_cols));
        }
    }

    return res;
}

fn generate_forced_grid(paths: &Vec<Vec<&Path>>, dims: &(usize, usize)) -> Grid {
    let mut grid = Grid::new(dims);

    for (color, p) in paths.iter().enumerate() {
        for coord in &get_forced_coord(p, dims) {
            grid.set(coord, Some(color));
        }
    }

    return grid;
}

fn generate_single_grid(paths: &Vec<Vec<&Path>>, dims: &(usize, usize)) -> Grid {
    let nb_colors = paths.len();
    let mut grid = Grid::new(dims);

    for color in 0..nb_colors {
        for &path in &paths[color] {
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

    for line in &mut grid.data {
        for elem in line {
            if elem == &Some(usize::MAX) {
                *elem = None;
            }
        }
    }

    return grid;
}

fn get_single_coords(grid: &Grid, color: &Color) -> Vec<Coord> {
    let mut res = vec![];

    for (row, line) in grid.data.iter().enumerate() {
        for (col, elem) in line.iter().enumerate() {
            if elem == &Some(*color) {
                res.push((row, col));
            }
        }
    }

    return res;
}

fn filter_paths_forced(paths: &mut Vec<Vec<&Path>>, dims: &(usize, usize)) {
    let nb_colors = paths.len();
    let grid = generate_forced_grid(paths, dims);

    for color in 0..nb_colors {
        paths[color] = paths[color]
            .iter()
            .filter(|path| {
                !path
                    .iter()
                    .any(|coord| grid.get(coord).is_some_and(|c| c != color))
            })
            .map(|&p| p)
            .collect()
    }

    return;
}

fn filter_paths_single(paths: &mut Vec<Vec<&Path>>, dims: &(usize, usize)) {
    let nb_colors = paths.len();
    let grid = generate_single_grid(paths, dims);
    let &(nb_rows, nb_cols) = dims;

    for color in 0..nb_colors {
        paths[color] = paths[color]
            .iter()
            .filter(|&&path| {
                let mut map = vec![false; nb_cols * nb_rows];

                for (row, col) in path {
                    map[col + row * nb_cols] = true;
                }

                get_single_coords(&grid, &color)
                    .iter()
                    .all(|(row, col)| map[col + row * nb_cols])
            })
            .map(|&path| path)
            .collect();
    }

    return;
}

fn check_reachable(paths: &Vec<Vec<&Path>>, dims: &(usize, usize)) -> bool {
    let &(nb_rows, nb_cols) = dims;
    let mut map = vec![false; nb_cols * nb_rows];
    let mut count = nb_rows * nb_cols;

    for p in paths {
        for &path in p {
            for &(row, col) in path {
                if !map[col + row * nb_cols] {
                    count -= 1;
                    if count == 0 {
                        return true;
                    }
                }
                map[col + row * nb_cols] = true;
            }
        }
    }

    return false;
}

fn prune(paths: &mut Vec<Vec<&Path>>, dims: &(usize, usize), verbose: bool) -> Option<bool> {
    let nb_colors = paths.len();
    let mut nb_paths: usize = paths.iter().map(|p| p.len()).sum();
    let mut last_nb_paths = 0;
    let mut grid_str = None;

    while !(nb_paths == last_nb_paths) {
        if verbose {
            grid_str = Some(generate_forced_grid(paths, dims).to_string());
        }

        filter_paths_forced(paths, dims);
        filter_paths_single(paths, dims);

        last_nb_paths = nb_paths;
        nb_paths = paths.iter().map(|p| p.len()).sum();

        if verbose {
            println!("{}", grid_str.as_ref().unwrap());
            print!("{ESC}[2K");
            print!(
                "Number of possible paths: {} (",
                paths.iter().map(|p| p.len()).sum::<usize>()
            );
            for color in 0..nb_colors {
                print!("{}", paths[color].len());
                if color != nb_colors - 1 {
                    print!(" - ")
                }
            }
            print!("){ESC}[{}F", dims.0);
            io::stdout().flush().unwrap();
        }

        if is_solved(paths) {
            return Some(true);
        }
        if is_impossible(paths) || !check_reachable(paths, dims) {
            return Some(false);
        }
    }

    return None;
}

fn is_solved(paths: &Vec<Vec<&Path>>) -> bool {
    return paths.iter().all(|p| p.len() == 1);
}

fn is_impossible(paths: &Vec<Vec<&Path>>) -> bool {
    return paths.iter().any(|p| p.is_empty());
}

fn backtrack(paths: &mut Vec<Vec<&Path>>, dims: &(usize, usize), verbose: bool) -> bool {
    let nb_colors = paths.len();
    let res = prune(paths, dims, verbose);

    if let Some(r) = res {
        return r;
    }

    let mut index = 0;
    while paths[index].len() == 1 {
        index += 1;
    }

    for &path in &paths[index] {
        let mut n_paths = vec![];
        for color in 0..nb_colors {
            n_paths.push(if color == index {
                vec![path]
            } else {
                paths[color].clone()
            });
        }

        if backtrack(&mut n_paths, dims, verbose) {
            *paths = n_paths;
            return true;
        }
    }

    return false;
}

fn generate_final_grid(paths: &Vec<Vec<&Path>>, dims: &(usize, usize)) -> Grid {
    let mut grid = Grid::new(dims);

    for (color, path) in paths.iter().enumerate() {
        for coord in path[0] {
            grid.set(&coord, Some(color));
        }
    }

    return grid;
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || args.contains(&"-h".to_string()) || args[1].contains(&"help".to_string())
    {
        println!("Flowfree Solver");
        println!();
        println!("Usage: flowfree [Option] [Path to Puzzle File]");
        println!();
        println!("Options:");
        println!("-h: Prints this help message and exits.");

        return;
    }

    let verbose_i = args.iter().position(|s| s == &"-v");

    match Grid::from(&args[if verbose_i == Some(1) { 2 } else { 1 }]) {
        Ok(grid) => {
            let verbose = verbose_i.is_some();
            let dims = &grid.get_dims();

            if verbose {
                print!("Generating the paths... ")
            }

            let paths = generate_all_paths(&grid);

            if verbose {
                println!("Done!");
            }

            let mut paths_refs = get_paths_refs(&paths);

            if verbose {
                println!("Pruning the paths using backtracking... ");
            }

            backtrack(&mut paths_refs, dims, verbose);

            if verbose {
                print!("{ESC}[{}E", dims.0 + 1);
                println!("Done!");
                println!("The following solution has been found: ")
            }

            println!("{}", generate_final_grid(&paths_refs, dims).to_string())
        }

        Err(e) => {
            println!("Error: could not open the file \"{}\" - {}", args[1], e);
        }
    }
}

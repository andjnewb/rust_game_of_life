use core::num;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::isize;
use std::num::ParseIntError;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use thiserror::Error;

//Think of these as the amount you would have to add to get to any cells neighbor, with origin at top left
const NORTH: (isize, isize) = (0, -1);
const EAST: (isize, isize) = (1, 0);
const WEST: (isize, isize) = (-1, 0);
const SOUTH: (isize, isize) = (0, 1);
const NORTH_EAST: (isize, isize) = (1, -1);
const NORTH_WEST: (isize, isize) = (-1, -1);
const SOUTH_EAST: (isize, isize) = (1, 1);
const SOUTH_WEST: (isize, isize) = (-1, 1);

const DIRECTIONS: [(isize, isize); 8] = [
    NORTH, SOUTH, EAST, WEST, NORTH_EAST, NORTH_WEST, SOUTH_EAST, SOUTH_WEST,
];

const ALIVE: char = '*';
const DEAD: char = '.';
const WALL: char = '+';

struct Grid {
    //X and Y for dims but stored in cells as Y and X
    dims: (usize, usize),
    cells: Vec<Vec<bool>>,
}

#[derive(Error)]
enum GridError {
    #[error(r#"Issue opening the grid file:"#)]
    GridFileError(#[from] std::io::Error),

    #[error(
        r#"Error reading dimensions of grid file. Ensure it is formatted as per the example file."#
    )]
    GridDimError(#[from] std::num::ParseIntError),
}

impl std::fmt::Debug for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = writeln!(f, "{}", self);

        if let Some(source) = self.source() {
            writeln!(f, "Error caused by: {}", source)?;
        }
        Ok(())
    }
}

impl Grid {
    fn setup_row(&mut self, row: &str, row_idx: usize) {
        let mut new_row: String = row.to_string();

        //Trim the walls off

        for (idx, c) in new_row.char_indices() {
            if c == ALIVE {
                self.cells[row_idx][idx] = true;
            }
        }
    }

    fn check_valid_dir(&self, cell: (usize, usize), dir: (isize, isize)) -> bool {
        //This is not a good solution.
        if cell.0 == 0 && dir.0 == -1 {
            return false;
        }

        if cell.1 == 0 && dir.1 == -1 {
            return false;
        }

        //If this is confusing, it's because the dims are X and Y but cells are stored Y and X.
        if cell.0 == self.dims.1 - 1 && dir.0 == 1 {
            return false;
        }

        if cell.1 == self.dims.0 - 1 && dir.1 == 1 {
            return false;
        }

        if(cell.0 == self.dims.1 || cell.1 == self.dims.0)
        {
            println!("WTF");
            return false;
        }

        return false;
    }

    fn translate_coords(cell: (usize, usize), coord: (isize, isize)) -> (usize, usize) {
        let mut new_coord: (usize, usize) = cell;
        //Only use this after having used check_valid_dir. This sucks and is too verbose.
        //TODO: Handle over max size
        if coord.0.is_negative() {
            new_coord.0 = cell.0 - coord.0.wrapping_abs() as usize;
        } else {
            new_coord.0 = cell.0 + coord.0.wrapping_abs() as usize;
        }

        if coord.1.is_negative() {
            new_coord.1 = cell.1 - coord.1.wrapping_abs() as usize;
        } else {
            new_coord.1 = cell.1 + coord.1.wrapping_abs() as usize;
        }

        //This is dumb but whatever
        let test = new_coord.0;
        new_coord.0 = new_coord.1;
        new_coord.1 = test;

        return new_coord;
    }

    fn get_neighbors(&self, cell: (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors: Vec<(usize, usize)> = Vec::new();

        for direction in DIRECTIONS {
            if Self::check_valid_dir(&self, cell, direction) {
                //TODO: Check if the neighbor is TRUE
                let coord = Self::translate_coords(cell, direction);

                //println!("{},{}", coord.1, coord.0);

                if  self.cells[coord.1][coord.0] {
                    //neighbors.push((coord.1, coord.0));

                }
            }
        }

        return neighbors;
    }

    fn setup_grid(&mut self, grid_file_path: &str) -> Result<Self, GridError> {
        use GridError::*;
        let path = Path::new(&grid_file_path);

        let grid_file = File::open(path)?;

        let mut reader = BufReader::new(grid_file);

        let mut dim_line = String::new();
        let _dim_line_len = reader.read_line(&mut dim_line);

        let dimension_strs: (&str, &str) = dim_line.split_once('x').unwrap();

        println!("{},{}", dimension_strs.0, dimension_strs.1);

        //-2 because we want to ignore the walls. The size in the gridfile includes the walls.
        self.dims = (
            dimension_strs.0.parse::<usize>()? - 2,
            dimension_strs.1.trim_end().parse::<usize>()? - 2,
        );

        self.cells = vec![vec![false; self.dims.0]; self.dims.1];

        let mut ceiling = String::new();

        let _ = reader.read_line(&mut ceiling);

        ceiling = ceiling.trim_end().to_string();

        let mut row = String::new();

        //Floor will look exactly the same as the ceiling

        let mut row_idx = 0;
        while (row != ceiling) {
            row.clear();
            let _ = reader.read_line(&mut row);
            Self::setup_row(self, &row, row_idx);
            row_idx += 1;
        }

        Ok(Grid {
            dims: self.dims,
            cells: self.cells.clone(),
        })
    }

    fn print_grid(&self) {
        for row in &self.cells {
            for col in row {
                if *col == true {
                    print!("{}", ALIVE);
                } else {
                    print!("{}", DEAD);
                }
            }
            println!();
        }
    }

    fn determine_cell_state(&self, cell: (usize, usize)) -> bool {
        let neighbors = self.get_neighbors(cell);
        let num_neighbors = neighbors.len();

        if num_neighbors < 2 {
            return false;
        }

        if num_neighbors == 3 {
            return true;
        }

        if num_neighbors > 3 {
            return false;
        }

        return true;
    }

    fn iterate_grid(&mut self) {
        let mut y: usize = 0;

        while (y < self.dims.0) {
            let mut x: usize = 0;
            while (x < self.dims.1) {
                self.cells[y][x] = self.determine_cell_state((y, x));
                x += 1;
            }
            y += 1;
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut grid: Grid;
    grid = Grid {
        cells: vec![vec![false; 0]; 0],
        dims: (0, 0),
    };

    grid = Grid::setup_grid(&mut grid, "test.grid").unwrap();

    let wait = Duration::new(1, 0);


    //println!("{}", grid.cells.len());

    loop {
        clearscreen::clear().expect("Failed to clear screen..");
        grid.print_grid();
        println!();
        grid.iterate_grid();
        sleep(wait);
    }

    // let test = Grid::get_neighbors(&grid, (5, 11));

    // for (x, y) in test {
    //     println!("Neighbor for ({},{}) at ({},{})", 5, 11, y, x);
    // }
}

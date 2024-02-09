use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use std::{env, process::exit};

//Think of these as the amount you would have to add to get to any cells neighbor, with origin at top left
const NORTH: (i8, i8) = (0, -1);
const EAST: (i8, i8) = (1, 0);
const WEST: (i8, i8) = (-1, 0);
const SOUTH: (i8, i8) = (0, 1);
const NORTH_EAST: (i8, i8) = (1, -1);
const NORTH_WEST: (i8, i8) = (-1, -1);
const SOUTH_EAST: (i8, i8) = (1, 1);
const SOUTH_WEST: (i8, i8) = (-1, 1);

const DIRECTIONS: [(i8, i8); 8] = [
    NORTH, SOUTH, EAST, WEST, NORTH_EAST, NORTH_WEST, SOUTH_EAST, SOUTH_WEST,
];

const ALIVE: char = '*';
const DEAD: char = '.';
const WALL: char = '+';

struct Grid {
    //X and Y
    dims: (usize, usize),
    cells: Vec<Vec<bool>>,
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

    fn setup_grid(&mut self, grid_file_path: &str) -> Result<Self, std::io::Error> {
        let path = Path::new(&grid_file_path);

        let grid_file = File::open(path)?;

        let mut reader = BufReader::new(grid_file);

        let mut dim_line = String::new();
        let _dim_line_len = reader.read_line(&mut dim_line);

        let dimension_strs: (&str, &str) = dim_line.split_once('x').unwrap();

        println!("{},{}", dimension_strs.0, dimension_strs.1);

        //-2 because we want to ignore the walls. The size in the gridfile includes the walls.
        self.dims = (
            dimension_strs.0.parse::<usize>().unwrap() - 2,
            dimension_strs.1.trim_end().parse::<usize>().unwrap(),
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
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut grid: Grid;
    grid = Grid {
        cells: vec![vec![false; 0]; 0],
        dims: (0, 0),
    };

    grid = Grid::setup_grid(&mut grid, "test.grid").unwrap();
    grid.print_grid();
}

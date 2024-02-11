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
use std::io::Read;
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

struct coord
{
    loc: (isize, isize),
}

impl std::ops::Add  for coord {
    type Output = Option<(usize, usize)>;

    fn add(self, rhs: Self) -> Self::Output {
        let mut out:(isize, isize) = (0,0);
        if rhs.loc.0.is_negative()
        {
            out.0 = self.loc.0 - rhs.loc.0.abs();
        }
        else
        {
            out.0 = self.loc.0 + rhs.loc.0;
        }

        if rhs.loc.1.is_negative()
        {
            out.1 = self.loc.1 - rhs.loc.1.abs();
        }
        else
        {
            out.1 = self.loc.1 + rhs.loc.1;
        }



        Some((out.0 as usize, out.1 as usize))
    }
}

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Issue opening the grid file: {0}")]
    Io(#[from] std::io::Error),

    #[error(
        "Error reading dimensions of grid file: {0}. Ensure it is formatted as per the example file."
    )]
    Format(#[from] std::num::ParseIntError),

    #[error("Reached the end of the file unexpectedly")]
    Eof,

    // this could include more information, like the character in question, or where to find it
    #[error("todo")]
    BadChar,

    #[error("Dimensions did not match file header")]
    DimensionMismatch,
    
    #[error("you shouldn't actually have something this unhelpful")]
    Other,
}



impl Grid {
    
    pub fn load_from_file(path: &str) -> Result<Self, LoadError> {
        let text = std::fs::read_to_string(path)?;

        let cells = text
            .lines()
            .skip_while(|l| l.starts_with('#'))
            .map(|l| l.chars().map(|c| c == '*').collect())
            .collect::<Vec<Vec<bool>>>();

        for row in &cells {
            if row.len() != cells[0].len() {
                // todo
                return Err(LoadError::DimensionMismatch);
            }
        }
        let dimY = cells.len();
        let dimX = cells[0].len();

        Ok(Self { cells, dims: (dimY, dimX) })
    }

    fn check_valid_dir(&self, cell: (usize, usize), dir: (isize, isize)) -> bool {
        //This is not a good solution.

        
        let orig = coord{loc: (cell.0 as isize, cell.1 as isize)};

        let orig2 = coord{loc: (dir.0 , dir.1)};

        let test = orig + orig2;

        if(test.unwrap().0 >= self.dims.0 || test.unwrap().1 >= self.dims.1)
        {
            return false;
        }

        if(cell.0 == 0 && dir.0.is_negative()) || (cell.1 == 0 && dir.1.is_negative()) 
        {
            return false;
        }

        // println!("Neighbor for {:?} at {:?}", 
        // cell, test);

        return true;
    }

    

    fn get_neighbors(&self, cell: (usize, usize)) -> Vec<(usize, usize)> {
        let mut neighbors: Vec<(usize, usize)> = Vec::new();

        for direction in DIRECTIONS {
            if Self::check_valid_dir(&self, cell, direction) {

                let c = coord {loc: (cell.0 as isize, cell.1 as isize)};
                let d = coord {loc: (direction.0 as isize, direction.1 as isize)};

                let res = c + d;

                if(self.cells[res.unwrap().0][res.unwrap().1])
                {
                    neighbors.push( (res).expect("Uh oh"));
                }
                
            }
        }

        return neighbors;
    }

    

    fn print_grid(&self) {

        // let mut x = 0;
        // while(x < self.dims.1)
        // {
        //     print!("{}", x);
        //     x += 1;
        // }
        // println!{};

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

    fn determine_cell_state(& mut self, cell: (usize, usize)) -> bool {
        let mut live : bool = self.cells[cell.0][cell.1];
        let neighbors = self.get_neighbors(cell);
        let num_living_neighbors = neighbors
        .iter()
        .filter(|c| self.cells[c.0][c.1])
        .count();

        //println!("Cell: {},{} {:?}", cell.0, cell.1, num_living_neighbors);

        //TODO::FIX
        if live && (num_living_neighbors < 2)
        {
            live = false;
        }

        if ( live && (num_living_neighbors == 2)) || ( live && (num_living_neighbors == 3))
        {
            live = true;
        }

        if live && (num_living_neighbors > 3)
        {
            live = false;
        }

        if !live && (num_living_neighbors == 3)
        {
            live = true;
        }


        return live;
    }

    fn iterate_grid(&mut self) {
        let mut new_grid:Vec<Vec<bool>> = self.cells.clone();


        let mut y:usize = 0;

        while y < self.dims.0
        {
            let mut x:usize = 0;
            while x < self.dims.1
            {
                new_grid[y][x] = self.determine_cell_state((y,x));
                x += 1;
            }
            
            y += 1;
        }

        //print!("{:?}", new_grid);
        self.cells = new_grid;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut grid: Grid;
    grid = Grid {
        cells: vec![vec![false; 0]; 0],
        dims: (0, 0),
    };


    grid = Grid::load_from_file("test.grid").unwrap();

    

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

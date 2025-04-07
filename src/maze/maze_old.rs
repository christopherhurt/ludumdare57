use rand::prelude::*;

const CELL_WIDTH: usize = 9;
const EDGE_WIDTH: usize = 1;

fn create_maze_vector(maze_area: usize) -> Vec<Vec<char>> {
    let grid_length;
    let grid_width;

    if maze_area < 16 {
        grid_length = 4;
        grid_width = 4;
    } else {
        let mut rng = rand::rng();
        grid_length = rng.random_range(4..(maze_area / 4));
        grid_width = maze_area / grid_length;
    }

    let mut maze = init_grid_door(grid_length, grid_width);
    maze = create_main_path_door(maze);
    let output = create_output_array_door(maze);

    //print_output(output, grid_length, grid_width);

    output
}

#[derive(Copy,Clone)]
pub struct Edge {
    pub is_wall: bool,
    pub is_starting_edge: bool,
    pub is_ending_edge: bool,
}

impl Default for Edge {
    fn default() -> Edge {
        Edge {
            is_wall: true,
            is_starting_edge: false,
            is_ending_edge: false,
        }
    }
}

#[derive(Copy,Clone)]
pub struct Cell {
    pub north_edge: Edge,
    pub east_edge: Edge,
    pub south_edge: Edge,
    pub west_edge: Edge,
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            north_edge: Default::default(),
            east_edge: Default::default(),
            south_edge: Default::default(),
            west_edge: Default::default(),
        }
    }
}

pub struct Maze {
    pub grid_length: usize,
    pub grid_width: usize,
    pub grid: Vec<Vec<Cell>>,
    pub start_x: usize,
    pub start_y: usize,
    pub end_x: usize,
    pub end_y: usize,
}

fn init_grid_door(grid_length: usize, grid_width: usize) -> Maze {
    let mut grid:Vec<Vec<Cell>> = vec![vec![]];

    // Create all cells
    for i in 0..grid_length {
        grid.push(vec![]);
        for j in 0..grid_width {
            let cell:Cell;
            if i == 0 {
                let north_edge = Edge {
                    is_wall: true,
                    ..Default::default()
                };
                if j == 0 {
                    let west_edge = Edge {
                        is_wall: true,
                        ..Default::default()
                    };
                    cell = Cell { north_edge: north_edge, west_edge: west_edge, ..Default::default() };
                } else {
                    cell = Cell { north_edge: north_edge, west_edge: grid[i][j - 1].east_edge, ..Default::default() };
                }
            } else {
                if j == 0 {
                    let west_edge = Edge {
                        is_wall: true,
                        ..Default::default()
                    };
                    cell = Cell { north_edge: grid[i - 1][j].south_edge, west_edge: west_edge, ..Default::default() };
                } else {
                    cell = Cell { north_edge: grid[i - 1][j].south_edge, west_edge: grid[i][j - 1].east_edge, ..Default::default() };
                }
            }
            grid[i].push(cell);
        }
    }

    let maze = Maze {
        grid_length: grid_length,
        grid_width: grid_width,
        grid: grid,
        start_x: 0,
        start_y: 0,
        end_x: 0,
        end_y: 0,
    };

    maze
}

fn create_main_path_door(mut maze: Maze) -> Maze {
    let mut rng = rand::rng();
    let starting_x = rng.random_range(0..maze.grid_width);
    let starting_y = rng.random_range(0..maze.grid_length);
    let starting_wall = rng.random_range(0..4);

    if starting_wall ==  0 {
        maze.grid[starting_y][starting_x].north_edge.is_starting_edge = true;
    }
    else if starting_wall == 1 {
        maze.grid[starting_y][starting_x].east_edge.is_starting_edge = true;
    }
    else if starting_wall == 2 {
        maze.grid[starting_y][starting_x].south_edge.is_starting_edge = true;
    }
    else {
        maze.grid[starting_y][starting_x].west_edge.is_starting_edge = true;
    }

    let mut ending_x = rng.random_range(0..maze.grid_width);
    let ending_y = rng.random_range(0..maze.grid_length);
    let ending_wall = rng.random_range(0..4);

    let x_diff;
    let y_diff;

    if ending_x > starting_x {
        x_diff = ending_x - starting_x;
    } else {
        x_diff = starting_x - ending_x;
    }

    if ending_y > starting_y {
        y_diff = ending_y - starting_y;
    } else {
        y_diff = starting_y - ending_y;
    }

    if x_diff.abs_diff(0) < 2 && y_diff.abs_diff(0) < 2 {
        if ending_x < starting_x {
            if ending_x == 0 {
                ending_x += 3;
            } else {
                ending_x -= 1;
            }
        } 
        else if x_diff == 0 {
            if ending_x < 2  {
                ending_x += 2;
            } else {
                ending_x -= 2;
            }
        }
        else {
            if ending_x == maze.grid_width - 1 {
                ending_x -= 3;
            } else {
                ending_x += 1;
            }
        }
    }

    if ending_wall ==  0 {
        maze.grid[ending_y][ending_x].north_edge.is_ending_edge = true;
    }
    else if ending_wall == 1 {
        maze.grid[ending_y][ending_x].east_edge.is_ending_edge = true;
    }
    else if ending_wall == 2 {
        maze.grid[ending_y][ending_x].south_edge.is_ending_edge = true;
    }
    else {
        maze.grid[ending_y][ending_x].west_edge.is_ending_edge = true;
    }

    maze.start_x = starting_x;
    maze.start_y = starting_y;
    maze.end_x = ending_x;
    maze.end_y = ending_y;

    maze
}

fn create_output_array_door(maze: Maze) -> Vec<Vec<char>> {
    // create array
    let total_cell_length = CELL_WIDTH + 2 * EDGE_WIDTH;
    let array_length:usize = (CELL_WIDTH + 2 * EDGE_WIDTH) * maze.grid_length;
    let mut output:Vec<Vec<char>> = vec![vec![]];

    // initialize output
    for _i in 0..array_length {
        output.push(vec![]);
    }

    for i in 0..maze.grid_length {
        for j in 0..maze.grid_width {
            for grid_i in 0..total_cell_length {
                for grid_j in 0..total_cell_length {
                    if grid_i == 0 && maze.grid[i][j].north_edge.is_wall {
                        if grid_j == total_cell_length / 2 + 1 || 
                            grid_j == total_cell_length / 2 || 
                            grid_j == total_cell_length / 2 - 1 {
                            output[i * total_cell_length + grid_i].push('d');
                        } else {
                            output[i * total_cell_length + grid_i].push('w');
                        }
                    }
                    else if grid_j == 0 && maze.grid[i][j].west_edge.is_wall {
                        if grid_i == total_cell_length / 2 + 1 || 
                            grid_i == total_cell_length / 2 || 
                            grid_i == total_cell_length / 2 - 1 {
                            output[i * total_cell_length + grid_i].push('d');
                        } else {
                            output[i * total_cell_length + grid_i].push('w');
                        }
                    }
                    else if grid_i == total_cell_length - 1 && maze.grid[i][j].south_edge.is_wall {
                        if grid_j == total_cell_length / 2 + 1 || 
                            grid_j == total_cell_length / 2 || 
                            grid_j == total_cell_length / 2 - 1 {
                            output[i * total_cell_length + grid_i].push('d');
                        } else {
                            output[i * total_cell_length + grid_i].push('w');
                        }
                    }
                    else if grid_j == total_cell_length - 1 && maze.grid[i][j].east_edge.is_wall {
                        if grid_i == total_cell_length / 2 + 1 || 
                            grid_i == total_cell_length / 2 || 
                            grid_i == total_cell_length / 2 - 1 {
                            output[i * total_cell_length + grid_i].push('d');
                        } else {
                            output[i * total_cell_length + grid_i].push('w');
                        }
                    } 
                    else {
                        output[i * total_cell_length + grid_i].push('o');
                    }    
                }
            }
        }
    }

    let mut rng = rand::rng();
    let dist_from_wall = CELL_WIDTH / 4 + EDGE_WIDTH;

    let start_position;
    if rng.random_range(0..2) == 1 {
        start_position = total_cell_length - dist_from_wall;
    } else {
        start_position = dist_from_wall;
    }

    let end_position;
    if rng.random_range(0..2) == 1 {
        end_position = total_cell_length - dist_from_wall;
    } else {
        end_position = dist_from_wall;
    }

    if maze.grid[maze.start_y][maze.start_x].north_edge.is_starting_edge {
        output[maze.start_y * total_cell_length][maze.start_x * total_cell_length + start_position] = 's';
        output[maze.start_y * total_cell_length + 1][maze.start_x * total_cell_length + start_position] = 'p';
    }
    else if maze.grid[maze.start_y][maze.start_x].east_edge.is_starting_edge {
        output[maze.start_y * total_cell_length + start_position][(maze.start_x + 1) * total_cell_length - 1] = 's';
        output[maze.start_y * total_cell_length + start_position][(maze.start_x + 1) * total_cell_length - 2] = 'p';
    }
    else if maze.grid[maze.start_y][maze.start_x].south_edge.is_starting_edge {
        output[(maze.start_y + 1) * total_cell_length - 1][maze.start_x * total_cell_length + start_position] = 's';
        output[(maze.start_y + 1) * total_cell_length - 2][maze.start_x * total_cell_length + start_position] = 'p';
    }
    else {
        output[maze.start_y * total_cell_length + start_position][maze.start_x * total_cell_length] = 's';
        output[maze.start_y * total_cell_length + start_position][maze.start_x * total_cell_length + 1] = 'p';
    }

    if maze.grid[maze.end_y][maze.end_x].north_edge.is_ending_edge {
        output[maze.end_y * total_cell_length][maze.end_x * total_cell_length + end_position] = 'e';
    }
    else if maze.grid[maze.end_y][maze.end_x].east_edge.is_ending_edge {
        output[maze.end_y * total_cell_length + end_position][(maze.end_x + 1) * total_cell_length - 1] = 'e';
    }
    else if maze.grid[maze.end_y][maze.end_x].south_edge.is_ending_edge {
        output[(maze.end_y + 1) * total_cell_length - 1][maze.end_x * total_cell_length + end_position] = 'e';
    }
    else {
        output[maze.end_y * total_cell_length + end_position][maze.end_x * total_cell_length] = 'e';
    }

    output

}

fn print_output(output: Vec<Vec<char>>, grid_length: usize, grid_width: usize) {
    let array_length:usize = (CELL_WIDTH + 2 * EDGE_WIDTH) * grid_length;
    let array_width:usize = (CELL_WIDTH + 2 * EDGE_WIDTH) * grid_width;
    for i in 0..array_length {
        for j in 0..array_width {
            let value = output[i][j];
            print!("{value}");
        }
        println!();
    }
}
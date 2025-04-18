use rand::prelude::*;
use rand::rng;
use rand::seq::SliceRandom;

const CELL_WIDTH: usize = 5;
const EDGE_WIDTH: usize = 1;
const MIN_SIDE_LENGTH: usize = 4;

pub fn create_maze_vector(maze_area: usize) -> Vec<Vec<char>> {
    let grid_length;
    let grid_width;

    if maze_area <= MIN_SIDE_LENGTH.pow(2) {
        grid_length = MIN_SIDE_LENGTH;
        grid_width = MIN_SIDE_LENGTH;
    } else {
        let mut rng = rand::rng();
        grid_length = rng.random_range(MIN_SIDE_LENGTH..(maze_area / MIN_SIDE_LENGTH + 1));
        grid_width = maze_area / grid_length;
    }

    let mut maze = init_grid(grid_width, grid_length);
    add_edges(&mut maze, grid_width, grid_length);
    add_player_exit(&mut maze, grid_width, grid_length);
    let output = create_output_vec(maze);

    output
}

#[derive(Copy,Clone)]
pub struct Cell {
    pub is_wall: bool,
    pub is_player: bool,
    pub is_end: bool,
    pub is_pathed: bool,
    pub is_connected: bool,
    pub is_outer_edge: bool,
    pub tag: usize,
    pub prev_junction_x: usize,
    pub prev_junction_y: usize,
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            is_wall: false,
            is_player: false,
            is_end: false,
            is_pathed: false,
            is_connected: false,
            is_outer_edge: false,
            tag: 0,
            prev_junction_x: 0,
            prev_junction_y: 0,
        }
    }
}

fn init_grid(grid_width: usize, grid_length: usize) -> Vec<Vec<Cell>> {
    let mut grid:Vec<Vec<Cell>> = vec![vec![]];

    let vec_width = grid_width * (CELL_WIDTH + EDGE_WIDTH) + 1;
    let vec_length = grid_length * (CELL_WIDTH + EDGE_WIDTH) + 1;


    // Initialize grid
    for i in 0..vec_width {
        if i != 0 {
            grid.push(vec![]);
        }
        for _j in 0..vec_length {
            let cell: Cell = Default::default();
            grid[i].push(cell);
        }
    }

    // Add outer wall
    for i in 0..vec_width {
        grid[i][0].is_wall = true;
        grid[i][0].is_connected = true;
        grid[i][0].is_outer_edge = true;
        grid[i][0].tag = 1;
        grid[i][vec_length - 1].is_wall = true;
        grid[i][vec_length - 1].is_connected = true;
        grid[i][vec_length - 1].is_outer_edge = true;
        grid[i][vec_length - 1].tag = 1;
    }

    for j in 0..vec_length {
        grid[0][j].is_wall = true;
        grid[0][j].is_connected = true;
        grid[0][j].is_outer_edge = true;
        grid[0][j].tag = 1;
        grid[vec_width - 1][j].is_wall = true;
        grid[vec_width - 1][j].is_connected = true;
        grid[vec_width - 1][j].is_outer_edge = true;
        grid[vec_width - 1][j].tag = 1;
    }

    grid
}

fn add_edges(grid: &mut Vec<Vec<Cell>>, grid_width: usize, grid_length: usize) {
    let mut next_tag = 2;
    
    let edge_width = grid_width - 1;
    let edge_length = grid_length - 1;
    let edge_distance = CELL_WIDTH + EDGE_WIDTH;

    let mut edge_coordinates: Vec<(usize, usize)> = vec![(1, 1)];
    for i in 1..(edge_width + 1) {
        for j in 1..(edge_length + 1) {
            if i == 1 && j == 1 {
                continue;
            }
            edge_coordinates.push((i, j));
        }
    }
    edge_coordinates.shuffle(&mut rng());

    // Add random edges
    let mut counter = 0;
    for coordinate in 0..edge_coordinates.len() {
        add_edge(grid, edge_coordinates[coordinate].0, edge_coordinates[coordinate].1, edge_distance, counter % 2 == 0);
        next_tag = tag_edges(grid, grid_width, grid_length, next_tag);
        counter += 1;
    }

    // Add vertical edges
    for i in 1..(edge_width + 1) {
        for j in 1..(edge_length + 1) {
            add_edge(grid, i, j, edge_distance, false);
            next_tag = tag_edges(grid, grid_width, grid_length, next_tag);
        }
    }

    // Add horizontal edges
    for i in 1..(edge_width + 1) {
        for j in 1..(edge_length + 1) {
            add_edge(grid, i, j, edge_distance, true);
            next_tag = tag_edges(grid, grid_width, grid_length, next_tag);
        }
    }

    // Add vertical edges again
    for i in 1..(edge_width + 1) {
        for j in 1..(edge_length + 1) {
            add_edge(grid, i, j, edge_distance, false);
            next_tag = tag_edges(grid, grid_width, grid_length, next_tag);
        }
    }

    // Add horizontal edges again
    for i in 1..(edge_width + 1) {
        for j in 1..(edge_length + 1) {
            add_edge(grid, i, j, edge_distance, true);
            next_tag = tag_edges(grid, grid_width, grid_length, next_tag);
        }
    }
}

fn add_edge(grid: &mut Vec<Vec<Cell>>, start_x: usize, start_y: usize, edge_distance: usize, is_vertical: bool) {
    let mut rng = rand::rng();
    let mut is_up = rng.random_range(0..2) == 1;

    let edge_start_x = start_x * edge_distance;
    let mut edge_end_x = edge_start_x;
    let edge_start_y = start_y * edge_distance;
    let mut edge_end_y = edge_start_y;
    let mut hit_wall = false;
    let mut tag = grid[edge_start_x][edge_start_y].tag;

    let mut curr_tag;
    if is_up { 
        if is_vertical {
            curr_tag = grid[edge_start_x][edge_start_y + edge_distance].tag;
        } else {
            curr_tag = grid[edge_start_x + edge_distance][edge_start_y].tag;
        }
    } else { 
        if is_vertical {
            curr_tag = grid[edge_start_x][edge_start_y - edge_distance].tag;
        } else {
            curr_tag = grid[edge_start_x - edge_distance][edge_start_y].tag;
        }
    }

    if curr_tag > 0 && curr_tag == tag {
        hit_wall = true;
        is_up = !is_up;
    } else {
        if is_vertical {
            edge_end_y = if is_up { edge_start_y + edge_distance } else { edge_start_y - edge_distance };
        } else {
            edge_end_x = if is_up { edge_start_x + edge_distance } else { edge_start_x - edge_distance };
        }
        tag = curr_tag;
    }

    if hit_wall {
        if is_up { 
            if is_vertical {
                curr_tag = grid[edge_start_x][edge_start_y + edge_distance].tag;
            } else {
                curr_tag = grid[edge_start_x + edge_distance][edge_start_y].tag;
            }
        } else { 
            if is_vertical {
                curr_tag = grid[edge_start_x][edge_start_y - edge_distance].tag;
            } else {
                curr_tag = grid[edge_start_x - edge_distance][edge_start_y].tag;
            }
        }
    
        if curr_tag > 0 && curr_tag == tag {
            hit_wall = true;
        } else {
            if is_vertical {
                edge_end_y = if is_up { edge_start_y + edge_distance } else { edge_start_y - edge_distance };
            } else {
                edge_end_x = if is_up { edge_start_x + edge_distance } else { edge_start_x - edge_distance };
            }
            tag = curr_tag;
        }
    }

    if is_vertical {
        if edge_start_y != edge_end_y {
            if is_up {
                for index in edge_start_y..edge_end_y + 1 {
                    grid[edge_start_x][index].is_wall = true;
                    grid[edge_start_x][index].is_connected = hit_wall;
                    grid[edge_start_x][index].tag = tag;
                }
            } else {
                for index in edge_end_y..edge_start_y + 1 {
                    grid[edge_start_x][index].is_wall = true;
                    grid[edge_start_x][index].is_connected = hit_wall;
                    grid[edge_start_x][index].tag = tag;
                }
            }
        }
    } else {
        if edge_start_x != edge_end_x {
            if is_up {
                for index in edge_start_x..edge_end_x + 1 {
                    grid[index][edge_start_y].is_wall = true;
                    grid[index][edge_start_y].is_connected = hit_wall;
                    grid[index][edge_start_y].tag = tag;
                }
            } else {
                for index in edge_end_x..edge_start_x + 1 {
                    grid[index][edge_start_y].is_wall = true;
                    grid[index][edge_start_y].is_connected = hit_wall;
                    grid[index][edge_start_y].tag = tag;
                }
            }
        }
    }
}

fn tag_edges(grid: &mut Vec<Vec<Cell>>, grid_width: usize, grid_length: usize, mut next_tag: usize) -> usize {
    // tag starting at 0,0 with next_tag
    let mut curr_x = 0;
    let mut curr_y = 0;

    traverse_edge(grid, curr_x, curr_y, next_tag);

    next_tag += 1;

    // for loop through all edge indices and tag with next tag
    let edge_width = grid_width - 1;
    let edge_length = grid_length - 1;
    let edge_distance = CELL_WIDTH + EDGE_WIDTH;

    for i in 1..(edge_width + 1) {
        for j in 1..(edge_length + 1) {
            let edge_start_x = i * edge_distance;
            let edge_start_y = j * edge_distance;
            curr_x = i * edge_distance;
            curr_y = j * edge_distance;

            if !grid[edge_start_x][edge_start_y].is_wall || grid[edge_start_x][edge_start_y].tag == next_tag - 1 {
                continue;
            }

            traverse_edge(grid, curr_x, curr_y, next_tag);

            next_tag += 1;
        }
    }

    next_tag
}

fn traverse_edge(grid: &mut Vec<Vec<Cell>>, mut curr_x: usize, mut curr_y: usize, next_tag: usize) {
    let vec_width:usize = grid.len();
    let vec_length:usize = grid[0].len();
    let mut direction = 0; // 0: north, 1: east, 2: south, 3 west
    let mut counter = 0;
    let max_tags = 100000;
    let mut can_move = true;

    while can_move && counter < max_tags {
        let wall_left = curr_x > 0 && curr_x - 1 < vec_width && grid[curr_x - 1][curr_y].is_wall;
        let wall_above = curr_y + 1 < vec_length && grid[curr_x][curr_y + 1].is_wall;
        let wall_right = curr_x + 1 < vec_width && grid[curr_x + 1][curr_y].is_wall;
        let wall_down = curr_y > 0 && curr_y - 1 < vec_length && grid[curr_x][curr_y - 1].is_wall;

        can_move = (curr_x != 0 || curr_y != 0) || (wall_left && grid[curr_x - 1][curr_y].tag != next_tag) || 
            (wall_above && grid[curr_x][curr_y + 1].tag != next_tag) || (wall_right && grid[curr_x + 1][curr_y].tag != next_tag) || 
            (wall_down && grid[curr_x][curr_y - 1].tag != next_tag);
            
        counter += 1;

        grid[curr_x][curr_y].is_connected = true;
        grid[curr_x][curr_y].tag = next_tag;
        if direction == 0 {
            if curr_x > 0 && curr_x - 1 < vec_width && grid[curr_x - 1][curr_y].is_wall {
                curr_x -= 1;
                direction = 3;
            }
            else if curr_y + 1 < vec_length && grid[curr_x][curr_y + 1].is_wall {
                curr_y += 1;
                direction = 0;
            }
            else if curr_x + 1 < vec_width && grid[curr_x + 1][curr_y].is_wall {
                curr_x += 1;
                direction = 1;
            }
            else {
                curr_y -= 1;
                direction = 2;
            }
        }
        else if direction == 1 {
            if curr_y + 1 < vec_length && grid[curr_x][curr_y + 1].is_wall {
                curr_y += 1;
                direction = 0;
            }
            else if curr_x + 1 < vec_width && grid[curr_x + 1][curr_y].is_wall {
                curr_x += 1;
                direction = 1;
            }
            else if curr_y > 0 && curr_y - 1 < vec_length && grid[curr_x][curr_y - 1].is_wall {
                curr_y -= 1;
                direction = 2;
            }
            else {
                curr_x -= 1;
                direction = 3;
            }
        }
        else if direction == 2 {
            if curr_x + 1 < vec_width && grid[curr_x + 1][curr_y].is_wall {
                curr_x += 1;
                direction = 1;
            }
            else if curr_y > 0 && curr_y - 1 < vec_length && grid[curr_x][curr_y - 1].is_wall {
                curr_y -= 1;
                direction = 2;
            }
            else if curr_x > 0 && curr_x - 1 < vec_width && grid[curr_x - 1][curr_y].is_wall {
                curr_x -= 1;
                direction = 3;
            }
            else {
                curr_y += 1;
                direction = 0;
            }
        }
        else {
            if curr_y > 0 && curr_y - 1 < vec_length && grid[curr_x][curr_y - 1].is_wall {
                curr_y -= 1;
                direction = 2;
            }
            else if curr_x > 0 && curr_x - 1 < vec_width && grid[curr_x - 1][curr_y].is_wall {
                curr_x -= 1;
                direction = 3;
            }
            else if curr_y + 1 < vec_length && grid[curr_x][curr_y + 1].is_wall {
                curr_y += 1;
                direction = 0;
            }
            else {
                curr_x += 1;
                direction = 1;
            }
        }
    }

}

fn add_player_exit(grid: &mut Vec<Vec<Cell>>, grid_width: usize, grid_length: usize) {
    let mut rng = rand::rng();
    let starting_x = (CELL_WIDTH + EDGE_WIDTH) / 2;
    let starting_y = (CELL_WIDTH + EDGE_WIDTH) / 2;
    grid[starting_x][starting_y].is_player = true;

    let ending = rng.random_range(0..4);
    let mut has_wall = false;

    if ending == 0 {
        let ending_x = (CELL_WIDTH + EDGE_WIDTH) / 2;
        let ending_y = (grid_length - 1) * (CELL_WIDTH + EDGE_WIDTH) + (CELL_WIDTH + EDGE_WIDTH) / 2;
        
        for j in starting_y..ending_y {
            if grid[ending_x][j].is_wall {
                has_wall = true;
                break;
            }
        }
        if has_wall {
            grid[ending_x][ending_y].is_end = true;
            return;
        }
    }
    if ending == 1 {
        let ending_x = (grid_width - 1) * (CELL_WIDTH + EDGE_WIDTH) + (CELL_WIDTH + EDGE_WIDTH) / 2;
        let ending_y = (CELL_WIDTH + EDGE_WIDTH) / 2;

        for i in starting_x..ending_x {
            if grid[i][ending_y].is_wall {
                has_wall = true;
                break;
            }
        }
        if has_wall {
            grid[ending_x][ending_y].is_end = true;
            return;
        }
    }
    if ending == 2 {
        let ending_x = (grid_width - 1) * (CELL_WIDTH + EDGE_WIDTH) + (CELL_WIDTH + EDGE_WIDTH) / 2;
        let ending_y = (grid_length - 1) * (CELL_WIDTH + EDGE_WIDTH) + (CELL_WIDTH + EDGE_WIDTH) / 2;
        grid[ending_x][ending_y].is_end = true;
    }
    else {
        let ending_x = ((grid_width - 1) / 2) * (CELL_WIDTH + EDGE_WIDTH) + (CELL_WIDTH + EDGE_WIDTH) / 2;
        let ending_y = ((grid_length - 1) / 2) * (CELL_WIDTH + EDGE_WIDTH) + (CELL_WIDTH + EDGE_WIDTH) / 2;
        grid[ending_x][ending_y].is_end = true;
    }
}

fn create_output_vec(grid: Vec<Vec<Cell>>) -> Vec<Vec<char>> {
    let mut output:Vec<Vec<char>> = vec![vec![]];

    let vec_width:usize = grid.len();
    let vec_length:usize = grid[0].len();
    for i in 0..vec_width {
        if i != 0 {
            output.push(vec![]);
        }
        for j in 0..vec_length {
            let value = grid[i][j];

            if value.is_player {
                output[i].push('p');
            }
            else if value.is_end {
                output[i].push('e');
            }
            else if value.is_wall {
                output[i].push('w');
            }
            else {
                output[i].push('o');
            }

            //output[i].push(value.tag);
        }
    }

    output

}

fn _print_output(output: Vec<Vec<char>>) {
    let array_width:usize = output.len();
    let array_length:usize = output[0].len();
    for i in 0..array_width {
        for j in 0..array_length {
            let value = output[i][j];
            print!("{value}");
        }
        println!();
    }
}
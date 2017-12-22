use std::collections::HashSet;

use engine::{Color, Transform};
use utils::{random, random_range};

mod tile;
pub use self::tile::Tile;

pub struct WorldState {
    pub debug: bool,
    pub width: u32,
    pub height: u32,
    pub quality: u32,
    pub tile_size: u32,
    pub start_id: i32,
    pub end_id: i32,
    pub player: Transform,
    pub tiles: Vec<Tile>,
    pub recent_regen: bool,
}

impl WorldState {
    pub fn new() -> WorldState {
        let quality = 2; // Make the canvas quality better
        let width: u32 = 900 * quality;
        let height: u32 = 600 * quality;
        let tile_size: u32 = 50;

        let mut w = WorldState {
            debug: false,
            width,
            height,
            quality,
            tile_size,
            tiles: Vec::new(),
            player: Transform::default(),
            start_id: -1,
            end_id: -1,
            recent_regen: false,
        };
        w.reset();
        w
    }

    pub fn reset(&mut self) {
        self.tiles = generate_tiles(self.width, self.height, self.tile_size);
        self.set_all_tile_sides();
        self.set_target_tiles();
        self.set_start_node();
    }

    pub fn set_start_node(&mut self) {
        let half_tile = (self.tile_size / 2) as f64;
        self.start_id = self.get_tile_id_closest_to(
            self.player.pos_x - half_tile,
            self.player.pos_y - half_tile,
        ) as i32;
    }

    pub fn calc_astar(&mut self) {
        let mut open_nodes: Vec<usize> = Vec::new();
        let mut closed_nodes = HashSet::new();

        open_nodes.push(self.start_id as usize);
        let mut current_node;
        let end = self.tiles[self.end_id as usize].clone();

        for t in self.tiles.iter_mut() {
            t.reset(&end);
        }

        let mut flip_side_check_counter = 0;

        // Stop searching when either:
        // 1) target is closed, in which case the path has been found
        // 2) failed to find the target and the open list is empty (no path)
        while !closed_nodes.contains(&(self.end_id as usize)) && open_nodes.len() > 0 {
            // Find lowest F score
            open_nodes.sort_by(|a, b| {
                let a_f = &self.tiles[*a].f;
                let b_f = &self.tiles[*b].f;
                a_f.cmp(b_f)
            });

            current_node = open_nodes.swap_remove(0);
            closed_nodes.insert(current_node);

            let side_ids = vec![
                self.tiles[current_node].top,
                self.tiles[current_node].bottom,
                self.tiles[current_node].right,
                self.tiles[current_node].left,
            ];

            // Flip the check of left/right and top/bottom checks every few iterations
            // so a zig zag pattern appears instead of producing L shaped paths.
            if flip_side_check_counter % 4 == 0 {
                // Check each side node.
                // If the side exists (id >= 0)
                // If it's a wall, it's not set as a side so we don't need to worry about it.
                for s in side_ids.iter() {
                    let id = *s as usize;
                    if *s >= 0 && !closed_nodes.contains(&id) {
                        self.check_node(&mut open_nodes, current_node, id);
                    }
                }
            } else {
                for s in side_ids.iter().rev() {
                    let id = *s as usize;
                    if *s >= 0 && !closed_nodes.contains(&id) {
                        self.check_node(&mut open_nodes, current_node, id);
                    }
                }
            };

            flip_side_check_counter += 1;
        }
    }

    fn check_node(
        &mut self,
        open_nodes: &mut Vec<usize>,
        curr_node_id: usize,
        side_node_id: usize,
    ) {
        let side = self.tiles[side_node_id as usize].clone();
        let mut parent_id = 0;
        let mut parent_g = -1;
        // if it's not already on the open list
        if !open_nodes.contains(&(side_node_id as usize)) {
            open_nodes.push(side_node_id as usize);
            parent_id = curr_node_id;
            parent_g = self.tiles[curr_node_id].g;
        }
        // if it's already on the open list and the path is better (lower G value)
        else if side.g > side.g + 10 {
            parent_id = curr_node_id;
            parent_g = self.tiles[curr_node_id].g;
        }
        if parent_g != -1 {
            let r = &mut self.tiles[side_node_id as usize];
            r.parent_id = parent_id as i32;
            r.calc_f_g(parent_g);
        }
    }

    fn get_tile_at(&mut self, x: u32, y: u32) -> &mut Tile {
        let index = self.get_tile_id_at(x, y);
        &mut self.tiles[index]
    }

    fn get_random_tile(&mut self) -> Tile {
        let num_x_tiles = (self.width / self.tile_size) as i32;
        let num_y_tiles = (self.height / self.tile_size) as i32;
        let index = self.get_tile_id_at(
            random_range(0, num_x_tiles - 1) as u32,
            random_range(0, num_y_tiles - 1) as u32,
        );
        self.tiles[index].clone()
    }

    fn get_tile_id_at(&self, x: u32, y: u32) -> usize {
        let num_tiles = self.width / self.tile_size;
        let index = y * num_tiles + x;
        index as usize
    }

    fn get_tile_id_closest_to(&self, x: f64, y: f64) -> usize {
        let size = self.tile_size as f64;
        let x_id = (x / size).ceil() as u32;
        let y_id = (y / size).ceil() as u32;
        self.get_tile_id_at(x_id, y_id)
    }

    fn get_random_tile_id(&self) -> usize {
        let num_x_tiles = (self.width / self.tile_size) as i32;
        let num_y_tiles = (self.height / self.tile_size) as i32;
        self.get_tile_id_at(
            random_range(0, num_x_tiles - 1) as u32,
            random_range(0, num_y_tiles - 1) as u32,
        )
    }

    fn set_target_tiles(&mut self) {
        // self.start_id = self.get_tile_id_at(3, 3) as i32;
        self.start_id = self.get_random_tile_id() as i32;
        // self.end_id = self.get_tile_id_at(17, 12) as i32;
        self.end_id = self.get_random_tile_id() as i32;
        self.player.pos_x = self.tiles[self.start_id as usize].transform.pos_x;
        self.player.pos_y = self.tiles[self.start_id as usize].transform.pos_y;
    }

    fn set_all_tile_sides(&mut self) {
        let num_x_tiles = (self.width / self.tile_size) as i32;
        let num_y_tiles = (self.height / self.tile_size) as i32;
        for t_id in 0..self.tiles.len() {
            let x_id = self.tiles[t_id].x_id;
            let y_id = self.tiles[t_id].y_id;
            if x_id + 1 < num_x_tiles {
                let right = y_id * num_x_tiles + x_id + 1;
                if !self.tiles[right as usize].is_wall {
                    self.tiles[t_id].right = right;
                }
            }
            if x_id - 1 >= 0 {
                let left = y_id * num_x_tiles + x_id - 1;
                if !self.tiles[left as usize].is_wall {
                    self.tiles[t_id].left = left;
                }
            }

            if y_id - 1 >= 0 {
                let top = ((y_id - 1) * num_x_tiles) + x_id;
                if !self.tiles[top as usize].is_wall {
                    self.tiles[t_id].top = top;
                }
            }
            if y_id + 1 < num_y_tiles {
                let bottom = ((y_id + 1) * num_x_tiles) + x_id;
                if !self.tiles[bottom as usize].is_wall {
                    self.tiles[t_id].bottom = bottom;
                }
            }
        }
    }
}

fn generate_tiles(grid_width: u32, grid_height: u32, tile_size: u32) -> Vec<Tile> {
    let mut vec = Vec::new();
    let num_y_tiles = grid_height / tile_size;
    let num_x_tiles = grid_width / tile_size;

    for y in 0..num_y_tiles {
        for x in 0..num_x_tiles {
            let px = x as f64 * tile_size as f64;
            let py = y as f64 * tile_size as f64;
            let size = tile_size as f64;
            let mut t: Tile = Tile::new(px, py, size);
            t.x_id = x as i32;
            t.y_id = y as i32;
            t.node_id = (y * num_x_tiles + x) as usize;
            t.is_wall = {
                if random() >= 0.7 {
                    true
                } else {
                    false
                }
            };
            // Every other tile is true and rows are offset by one. This creates a checkerboard
            // let checkerboard_tile_test = (x + y) % 2 == 0;
            // let lightness = if checkerboard_tile_test { 30 } else { 20 };
            let lightness = if t.is_wall { 20 } else { 30 };
            t.color = Color::new(0, 0, lightness, 1_f32);
            vec.push(t);
        }
    }
    vec
}

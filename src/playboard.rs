use piston::input::UpdateArgs;
use piston_window::{Context, G2d};
use rand::{thread_rng, Rng};
use graphics::*;
use std::cmp;

use crate::{
	utils::{Vector, RenderContext, Direction, Rgba, multi_line_text},
	tile::{Tile, MovingTile}
};

const TEXT_COLOR: Rgba = [0.463, 0.431, 0.400, 1.0];
const TR_WHITE: Rgba = [1.0, 1.0, 1.0, 0.5];

#[derive(PartialEq)]
enum BoardState {
	Idle,
	Moving,
	GameOver
}

pub struct PlayBoard {
	tiles: Vec<Vec<Tile>>,
	moving_tiles: Vec<MovingTile>,
	board_size: usize,
	state: BoardState,
	score: u32,
	highscore: u32
}

impl PlayBoard {
	pub const PADDED_TILE_SIZE: f64 = 110.0;

	pub fn new(board_size: usize) -> Self {
		let mut tiles = Vec::with_capacity(board_size);

		// Fill board with empty tiles
		for y in 0..board_size {
			tiles.push(Vec::with_capacity(board_size));

			for x in 0..board_size {
				tiles[y].push(Tile::empty(x as f64, y as f64));
			}
		}

		PlayBoard {
			tiles: tiles,
			moving_tiles: Vec::with_capacity(board_size.pow(2)),
			board_size: board_size,
			state: BoardState::Idle,
			score: 0,
			highscore: 0
		}
	}

	fn get_empty_tile(&self) -> Option<Vector> {
		// Create a result vector with a max capacity of board size ^ 2
		let mut res: Vec<Vector> = Vec::with_capacity(self.tiles.len().pow(2));

		// Search for empty tiles
		for y in 0..self.board_size {
			for x in 0..self.board_size {
				if self.tiles[y][x].value == 99 {
					res.push(Vector::new(x as f64, y as f64));
				}
			}
		}

		// Returns position of random empty tile (if it exists)
		match res.len() {
			0 => None,
			_ => Some(res.remove((rand::random::<f32>() * (res.len() - 1) as f32).round() as usize))
		}
	}

	// Spawn a tile at a random location
	pub fn spawn_tile(&mut self) {
		match self.get_empty_tile() {
			Some(pos) => {
				let mut rng = thread_rng();
				self.tiles[pos.y as usize][pos.x as usize].value = rng.gen_range(0, 2);
				self.tiles[pos.y as usize][pos.x as usize].scale = 1.1;
			},
			_ => ()
		}
	}

	fn spawn_moving_tile(&mut self, x: isize, y: isize, direction: &Direction, walker: isize, combine: bool) {
		let tiles = &mut self.tiles;

		// Get destination position
		let (dx, dy) = direction.displacement();
		let (walker_x, walker_y) = (
			(x + walker * dx) as usize,
			(y + walker * dy) as usize);

		// Get position as usize
		let ux = x as usize;
		let uy = y as usize;

		// Spawn a moving tile
		self.moving_tiles.push(MovingTile::new(
			x as f64,
			y as f64,
			walker_x as f64,
			walker_y as f64,
			tiles[uy][ux].value,
			combine
		));
		
		// Set destination tile to `taken` if it will be combined with
		if combine {
			tiles[walker_y][walker_x].taken = true;
		}

		// Set destination tile's `occupied` value to the tile's value so that other moving tiles can combine with it 
		tiles[walker_y][walker_x].occupied = tiles[uy][ux].value;

		// Reset current tile
		tiles[uy][ux].value = 99;
		tiles[uy][ux].occupied = 99;
	}

	pub fn slide(&mut self, direction: Direction) {
		if self.state != BoardState::Idle { return; }

		// Get vector displacement from direction
		let (dx, dy) = direction.displacement();

		for y in 0..self.board_size {
			for x in 0..self.board_size {
				let tiles = &self.tiles;

				// Get x and y position of tile
				let ix =
					if dx == 1 { (self.board_size - x - 1) as isize }
					else { x as isize };
				let iy =
					if dy == 1 { (self.board_size - y - 1) as isize }
					else { y as isize };


				// Skip if tile is empty
				if tiles[iy as usize][ix as usize].value == 99 { continue; }

				// Makes iteration through columns go in the opposite direction
				let mut walker: isize = 0;
				let mut combine = false;

				// Find the max walking distance for the walker
				let max_walk =
					if direction.is_negative() {
						cmp::max(ix * dx.abs(), iy * dy.abs())
					}
					else {
						self.board_size as isize - cmp::max(ix * dx, iy * dy) - 1
					};
				
				// Check if the tile to the left is occupied or not in a loop
				while walker < max_walk {
					walker += 1;

					let (walker_x, walker_y) = (
						(ix + walker * dx) as usize,
						(iy + walker * dy) as usize);
					
					// Check if the tile is not empty
					if tiles[walker_y][walker_x].value != 99 || tiles[walker_y][walker_x].occupied != 99 {
						// If both tiles have the same value and the tile is not `taken`, enable the combine flag
						if (tiles[walker_y][walker_x].value == tiles[iy as usize][ix as usize].value || 
							tiles[walker_y][walker_x].occupied == tiles[iy as usize][ix as usize].value)
						    && !tiles[walker_y][walker_x].taken {
							combine = true;
						}
						// Move walker back 1 step
						else {
							walker -= 1;
						}

						// Break from loop if a tile is found
						break;
					}
				}

				// Check if the tile can move to another location
				if walker != 0 {
					self.spawn_moving_tile(ix, iy, &direction, walker, combine);
				}
			}
		}

		// Set board state to `moving` so that no more inputs are applied
		self.state = BoardState::Moving;
	}

	fn game_over(&self) -> bool {
		for y in 0..self.board_size {
			for x in 0..self.board_size {
				// Check for a match to the right
				if x < self.board_size - 1 {
					if self.tiles[y][x].value == self.tiles[y][x + 1].value {
						return false;
					}
				}

				// Check for a match below
				if y < self.board_size - 1 {
					if self.tiles[y][x].value == self.tiles[y + 1][x].value {
						return false;
					}
				}
			}
		}

		true
	}

	pub fn render(&mut self, render_ctx: &mut RenderContext, ctx: Context, gl: &mut G2d) {
		// Render tiles
		for row in &mut self.tiles {
			for tile in row {
				tile.render(render_ctx, ctx, gl);
			}
		}

		// Render moving tiles
		for tile in &mut self.moving_tiles {
			tile.render(render_ctx, ctx, gl);
		}

		// Get text transform
		let transform = ctx.transform
			.trans(245.0, 28.0);

		// Render current score
		text(TEXT_COLOR, 22, &format!("Score: {:?}", self.score), &mut render_ctx.glyphs.brandon_blk, transform, gl)
			.expect("Failed to draw text");

		// Render high score
		text(TEXT_COLOR, 22, &format!("High score: {:?}", self.highscore), &mut render_ctx.glyphs.brandon_blk, transform.trans(280.0, 0.0), gl)
			.expect("Failed to draw text");

		// Draw `GameOver` overlay and text
		if self.state == BoardState::GameOver {
			// Draw overlay
			rectangle(TR_WHITE, [0.0, 0.0, render_ctx.window_size[0], render_ctx.window_size[1]], ctx.transform, gl);

			// Get text transform
			let transform = ctx.transform
				.trans(render_ctx.window_size[0] / 2.0 - 80.0, 200.0);

			// "GAME OVER" banner
			text(TEXT_COLOR, 35, "Game Over!", &mut render_ctx.glyphs.brandon_blk, transform, gl)
				.expect("Failed to draw text");

			// Retry banner
			multi_line_text(
				TEXT_COLOR,
				25,
				&format!("            Score: {:?}\n\nRetry by pressing SPACE", self.score),
				&mut render_ctx.glyphs.brandon_blk,
				transform.trans(-35.0, 60.0),
				gl
			);
		}
	}

	pub fn update(&mut self, _args: &UpdateArgs) {
		// Update all moving tiles
		for tile in &mut self.moving_tiles {
			// Check if the moving tile is finished moving
			if tile.is_finished() {
				let destination = &mut self.tiles[tile.position.y as usize][tile.position.x as usize];

				if tile.combine {
					destination.value = tile.value + 1;
					destination.scale = 1.2;

					// Increment score
					self.score += 2u32.pow(tile.value as u32 + 2);
				}
				else {
					destination.value = tile.value;
				}

				destination.occupied = destination.value;
				destination.taken = false;
			}
		}

		let old_length = self.moving_tiles.len();

		// Remove all moving tiles that have finished
		self.moving_tiles.retain(|tile| !tile.is_finished());

		// Set board state to idle if no more tiles are moving
		let new_length = self.moving_tiles.len();
		if new_length == 0 && self.state == BoardState::Moving {
			self.state = BoardState::Idle;

			// Spawn a new random tile if the tiles have moved
			if old_length != new_length {
				self.spawn_tile();
			}

			// Check if the game is over
			if let None = self.get_empty_tile() {
				if self.game_over() {
					self.state = BoardState::GameOver;

					// Set new highscore (if applicable)
					self.highscore = cmp::max(self.score, self.highscore);
				}
			}
		}
	}

	pub fn reset(&mut self) {
		// Set board to empty tiles
		for y in 0..self.board_size {
			for x in 0..self.board_size {
				self.tiles[y][x].reset();
			}
		}

		// Reset score and state
		self.score = 0;
		self.state = BoardState::Idle;

		// Spawn 1 random tile
		self.spawn_tile();
	}

	pub fn board_size(&self) -> usize {
		self.board_size
	}
}
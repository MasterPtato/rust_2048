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
	pub board_size: usize,
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

	fn spawn_moving_tile(&mut self, x: usize, y: usize, walker: i32, combine: bool) {
		let row = &mut self.tiles[y];

		// Spawn a moving tile
		self.moving_tiles.push(MovingTile::new(
			x as f64,
			y as f64,
			walker as f64,
			y as f64,
			row[x].value,
			combine
		));
		
		// Set destination tile to `taken` if it will be combined with
		if combine {
			row[walker as usize].taken = true;
		}

		// Set destination tile's `occupied` value to the tile's value so that other moving tiles can combine with it 
		row[walker as usize].occupied = row[x].value;

		// Reset current tile
		row[x].value = 99;
		row[x].occupied = 99;
	}

	fn spawn_moving_tile_vertical(&mut self, x: usize, y: usize, walker: i32, combine: bool) {
		let tiles = &mut self.tiles;

		// Spawn a moving tile
		self.moving_tiles.push(MovingTile::new(
			x as f64,
			y as f64,
			x as f64,
			walker as f64,
			tiles[y][x].value,
			combine
		));
		
		// Set destination tile to `taken` if it will be combined with
		if combine {
			tiles[walker as usize][x].taken = true;
		}

		// Set destination tile's `occupied` value to the tile's value so that other moving tiles can combine with it 
		tiles[walker as usize][x].occupied = tiles[y][x].value;

		// Reset current tile
		tiles[y][x].value = 99;
		tiles[y][x].occupied = 99;
	}

	pub fn slide(&mut self, direction: Direction) {
		if self.state != BoardState::Idle { return; }

		match direction {
			// Slide playing board to the left
			Direction::Left => 
				for y in 0..self.board_size {
					for x in 0..self.board_size {
						let row = &mut self.tiles[y];

						// Skip if tile is empty
						if row[x].value == 99 { continue; }

						let mut walker = x as i32;
						let mut combine = false;
						
						// Check if the tile to the left is occupied or not in a loop
						while walker > 0 {
							walker -= 1;

							if row[walker as usize].value != 99 || row[walker as usize].occupied != 99 {
								// If both tiles have the same value and the tile is not `taken`, enable the combine flag
								if (row[walker as usize].value == row[x].value || row[walker as usize].occupied == row[x].value)
								   && !row[walker as usize].taken {
									combine = true;
								}
								// Move walker back 1 step
								else {
									walker += 1;
								}

								// Break from loop if a tile is found
								break;
							}
						}

						// Check if the tile can move to another location
						if walker as usize != x {
							self.spawn_moving_tile(x, y, walker, combine);
						}
					}
				},
			// Slide playing board to the right
			Direction::Right => 
				for y in 0..self.board_size {
					for x2 in 0..self.board_size {
						let row = &mut self.tiles[y];

						// Makes iteration through rows go in the opposite direction
						let x = self.board_size - x2 - 1;

						// Skip if tile is empty
						if row[x].value == 99 { continue; }

						let mut walker = x as i32;
						let mut combine = false;
						
						// Check if the tile to the right is occupied or not in a loop
						while walker < self.board_size as i32 - 1 {
							walker += 1;

							let walker_u = walker as usize;
							if row[walker_u].value != 99 || row[walker_u].occupied != 99 {
								// If both tiles have the same value and the tile is not `taken`, enable the combine flag
								if (row[walker_u].value == row[x].value || row[walker_u].occupied == row[x].value)
								   && !row[walker_u].taken {
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
						if walker as usize != x {
							self.spawn_moving_tile(x, y, walker, combine);
						}
					}
				},
			// Slide playing board upwards
			Direction::Up => 
				for y in 0..self.board_size {
					for x in 0..self.board_size {
						let tiles = &self.tiles;

						// Skip if tile is empty
						if tiles[y][x].value == 99 { continue; }

						let mut walker = y as i32;
						let mut combine = false;
						
						// Check if the tile to the left is occupied or not in a loop
						while walker > 0 {
							walker -= 1;

							let walker_u = walker as usize;
							if tiles[walker_u][x].value != 99 || tiles[walker_u][x].occupied != 99 {
								// If both tiles have the same value and the tile is not `taken`, enable the combine flag
								if (tiles[walker_u][x].value == tiles[y][x].value || tiles[walker_u][x].occupied == tiles[y][x].value)
								   && !tiles[walker_u][x].taken {
									combine = true;
								}
								// Move walker back 1 step
								else {
									walker += 1;
								}

								// Break from loop if a tile is found
								break;
							}
						}

						// Check if the tile can move to another location
						if walker as usize != y {
							self.spawn_moving_tile_vertical(x, y, walker, combine);
						}
					}
				},
			// Slide playing board downwards
			Direction::Down => 
				for y2 in 0..self.board_size {
					for x in 0..self.board_size {
						let tiles = &self.tiles;

						// Makes iteration through columns go in the opposite direction
						let y = self.board_size - y2 - 1;

						// Skip if tile is empty
						if tiles[y][x].value == 99 { continue; }

						let mut walker = y as i32;
						let mut combine = false;
						
						// Check if the tile to the left is occupied or not in a loop
						while walker < self.board_size as i32 - 1 {
							walker += 1;

							let walker_u = walker as usize;
							if tiles[walker_u][x].value != 99 || tiles[walker_u][x].occupied != 99 {
								// If both tiles have the same value and the tile is not `taken`, enable the combine flag
								if (tiles[walker_u][x].value == tiles[y][x].value || tiles[walker_u][x].occupied == tiles[y][x].value)
								   && !tiles[walker_u][x].taken {
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
						if walker as usize != y {
							self.spawn_moving_tile_vertical(x, y, walker, combine);
						}
					}
				}
		}

		// Set board state to `moving` so that no more inputs are applied
		self.state = BoardState::Moving;
	}

	fn game_over(&self) -> bool {
		for y in 0..(self.board_size - 1) {
			for x in 0..self.board_size {
				// Check for a match to the right
				if x < self.board_size - 1 {
					if self.tiles[y][x].value == self.tiles[y][x + 1].value {
						return false;
					}
				}

				// Check for a match below
				if self.tiles[y][x].value == self.tiles[y + 1][x].value {
					return false;
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
}
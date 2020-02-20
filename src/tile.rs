use piston::input::{UpdateArgs};
use piston_window::{Context, G2d};
use graphics::*;

use crate::playboard::PlayBoard;
use crate::utils::{Vector, RenderContext};

#[derive(Debug)]
pub struct Tile {
	position: Vector,
	pub value: usize,
	pub occupied: usize,
	pub taken: bool
}

impl Tile {
	#[allow(dead_code)]
	pub fn new(x: f64, y: f64, value: usize) -> Self {
		Tile {
			position: Vector::new(x, y),
			value: value,
			occupied: value,
			taken: false
		}
	}

	// Create a new empty tile
	pub fn empty(x: f64, y: f64) -> Self {
		Tile {
			position: Vector::new(x, y),
			value: 99,
			occupied: 99,
			taken: false
		}
	}

	pub fn render(&mut self, render_ctx: &mut RenderContext, ctx: Context, gl: &mut G2d) {
		// Get transform for drawing
		let transform = ctx.transform
			.trans(self.position.x * PlayBoard::PADDED_TILE_SIZE, self.position.y * PlayBoard::PADDED_TILE_SIZE)
			.trans(render_ctx.window_size[0] / 2.0, render_ctx.window_size[1] / 2.0)
			.trans(render_ctx.board_size[0], render_ctx.board_size[1])
			.trans(-48.0, -48.0);

		// Check if this tile is an empty one
		if self.value == 99 || self.value == 98 {
			// Draw empty tile
			if let Some(texture) = &render_ctx.textures.empty_tile {
				image(texture, transform, gl);
			}
		}
		else {
			// Draw tile
			image(&render_ctx.textures.nums[self.value], transform, gl);
		}
	}
}

#[derive(Debug)]
pub struct MovingTile {
	pub position: Vector,
	new_position: Vector,
	pub value: usize,
	pub combine: bool
}

impl MovingTile {
	pub fn new(x: f64, y: f64, new_x: f64, new_y: f64, value: usize, combine: bool) -> Self {
		MovingTile {
			position: Vector::new(x, y),
			new_position: Vector::new(new_x, new_y),
			value: value,
			combine: combine
		}
	}

	pub fn render(&mut self, render_ctx: &mut RenderContext, ctx: Context, gl: &mut G2d) {
		// Move the position of the tile to its new position
		let move_dt = render_ctx.dt * 32.0;

		// Move X
		if (self.position.x - self.new_position.x).abs() > move_dt {
			if self.position.x < self.new_position.x {
				self.position.x += move_dt;
			}
			else {
				self.position.x -= move_dt;
			}
		}
		else {
			self.position.x = self.new_position.x;
		}

		// Move Y
		if (self.position.y - self.new_position.y).abs() > move_dt {
			if self.position.y < self.new_position.y {
				self.position.y += move_dt;
			}
			else {
				self.position.y -= move_dt;
			}
		}
		else {
			self.position.y = self.new_position.y;
		}

		// Get transform for drawing
		let transform = ctx.transform
			.trans(self.position.x * PlayBoard::PADDED_TILE_SIZE, self.position.y * PlayBoard::PADDED_TILE_SIZE)
			.trans(render_ctx.window_size[0] / 2.0, render_ctx.window_size[1] / 2.0)
			.trans(render_ctx.board_size[0], render_ctx.board_size[1])
			.trans(-48.0, -48.0);


		// Draw tile
		image(&render_ctx.textures.nums[self.value], transform, gl);
	}

	pub fn is_finished(&self) -> bool {
		self.position.x == self.new_position.x && self.position.y == self.new_position.y
	}
}
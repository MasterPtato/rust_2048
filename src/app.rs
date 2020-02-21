use piston::input::{RenderEvent, Event, UpdateArgs, UpdateEvent, PressEvent, ReleaseEvent, Button};
use piston_window::{PistonWindow, Button::Keyboard, Key};
use piston::event_loop::{EventSettings, Events, EventLoop};
use graphics::*;

use crate::playboard::PlayBoard;
use crate::utils::{
	FRAME_COUNT_FOR_AVG,
	RenderContext,
	Rgba,
	GlyphsStorage,
	Direction,
	KeyMap,
	multi_line_text
};

const BG_COLOR: Rgba = [0.733, 0.678, 0.627, 1.0];
const TEXT_COLOR: Rgba = [0.463, 0.431, 0.400, 1.0];

pub struct App {
	window: PistonWindow,
	board: PlayBoard,
	keys: KeyMap
}

impl App {
	pub fn new(window: PistonWindow) -> Self {
		App {
			window: window,
			board: PlayBoard::new(4),
			keys: KeyMap::new()
		}
	}

	fn render(&mut self, e: &Event, render_ctx: &mut RenderContext) {
		// Used for averaging FPS
		render_ctx.avg.push(render_ctx.dt);
		if render_ctx.avg.len() >= FRAME_COUNT_FOR_AVG {
			render_ctx.avg.remove(0);
		}

		render_ctx.dt = render_ctx.avg.iter().sum::<f64>() / render_ctx.avg.len() as f64;

		let board = &mut self.board;

		self.window.draw_2d(e, |ctx, gl, device| {
			// Clear the screen
			clear(BG_COLOR, gl);

			// Get text transform
			let transform = ctx.transform
				.trans(60.0, 70.0);

			// Render title
			text(TEXT_COLOR, 50, "2048", &mut render_ctx.glyphs.brandon_blk, transform, gl)
				.expect("Failed to draw text");

			let transform = ctx.transform
				.trans(15.0, 95.0);

			// Render instructions
			multi_line_text(
				TEXT_COLOR,
				25,
				"Combine the tiles by\nsliding the board with\nWASD or arrow keys",
				&mut render_ctx.glyphs.brandon_blk,
				transform,
				gl
			);

			// Render the playing board
			board.render(render_ctx, ctx, gl);

			// Update glyphs before rendering
			render_ctx.glyphs.fira_code_reg.factory.encoder.flush(device);
			render_ctx.glyphs.brandon_blk.factory.encoder.flush(device);
		});
	}

	fn update(&mut self, args: &UpdateArgs) {
		// Update playing board
		self.board.update(args);
	}

	fn key_press(&mut self, args: &Button) {
		// Slide board based on key press
		match *args {
			Keyboard(Key::Left) | Keyboard(Key::A) => {
				// Only slide if the key has been released prior to being pressed, prevents holding key
				if self.keys.left.released {
					self.keys.left.released = false;

					self.board.slide(Direction::Left);
				}
			},
			Keyboard(Key::Right) | Keyboard(Key::D) => {
				if self.keys.right.released {
					self.keys.right.released = false;

					self.board.slide(Direction::Right);
				}
			},
			Keyboard(Key::Up) | Keyboard(Key::W) => {
				if self.keys.up.released {
					self.keys.up.released = false;

					self.board.slide(Direction::Up);
				}
			},
			Keyboard(Key::Down) | Keyboard(Key::S) => {
				if self.keys.down.released {
					self.keys.down.released = false;

					self.board.slide(Direction::Down);
				}
			},
			// Reset game on SPACEBAR press
			Keyboard(Key::Space) => {
				self.board.reset();
			},
			_ => ()
		}
	}

	fn key_release(&mut self, args: &Button) {
		// Set pressed key to released
		match *args {
			Keyboard(Key::Left)  | Keyboard(Key::A) => self.keys.left.released = true,
			Keyboard(Key::Right) | Keyboard(Key::D) => self.keys.right.released = true,
			Keyboard(Key::Up)    | Keyboard(Key::W) => self.keys.up.released = true,
			Keyboard(Key::Down)  | Keyboard(Key::S) => self.keys.down.released = true,
			_ => ()
		}
	}

	pub fn init(&mut self) {
		// Load ttf fonts
		let fcreg = self.window.load_font("./assets/FiraCode-Regular.ttf").unwrap();
		let bblk = self.window.load_font("./assets/Brandon_blk.ttf").unwrap();

		// Create a new render context
		let mut render_ctx = RenderContext::new(&mut self.window, GlyphsStorage {
			fira_code_reg: fcreg,
			brandon_blk: bblk
		});

		// Store the board size offset in the render context
		render_ctx.board_size = [self.board.board_size as f64 * -15.0, self.board.board_size as f64 * -41.0];

		// Start event handler at 60 UPS
		let mut events = Events::new(EventSettings::new().ups(60));

		// Spawn 1 tile on the board
		self.board.spawn_tile();

		// Initiate events
    	while let Some(e) = events.next(&mut self.window) {
		// while let Some(e) = self.window.next() {
			if let Some(args) = e.render_args() {
				// Update render context
				render_ctx.dt = args.ext_dt;
				render_ctx.window_size = args.window_size;

				// Render app
				self.render(&e, &mut render_ctx);
			}

			if let Some(args) = e.update_args() {
				// Update app
				self.update(&args);
			}

			if let Some(args) = e.press_args() {
				// Register key presses to app
				self.key_press(&args);
			}
			if let Some(args) = e.release_args() {
				// Register key releases to app
				self.key_release(&args);
			}
		}
	}
}
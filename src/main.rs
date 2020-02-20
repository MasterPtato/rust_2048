use opengl_graphics::OpenGL;
use piston_window::{WindowSettings, PistonWindow};

mod tile;
mod playboard;
mod utils;
mod app;

use app::App;

fn main() {
	// Create a window
	let window_res: Result<PistonWindow, Box<_>> = WindowSettings::new("2048 game", [700, 500])
		.graphics_api(OpenGL::V3_2)
		.exit_on_esc(true)
		.resizable(false)
		.samples(0)
		.build();

	if let Ok(window) = window_res {
		// Create a new app
		let mut app = App::new(window);

		// Begin app
		app.init();
	}
	else {
		println!("Failed to create window. Aborting.");
	}
}
use piston_window::{PistonWindow, Glyphs, TextureSettings, G2dTexture, Texture, Flip, G2d};
use graphics::*;

pub const FRAME_COUNT_FOR_AVG: usize = 60;

pub type Rgba = [f32; 4];

pub struct TextureStorage {
	pub nums: Vec<G2dTexture>,
	pub empty_tile: Option<G2dTexture>
}

impl TextureStorage {
	fn new() -> Self {
		TextureStorage {
			nums: Vec::with_capacity(8),
			empty_tile: None
		}
	}
}

pub enum Direction {
	Left,
	Right,
	Up,
	Down
}

pub struct ToggleKey {
	pub pressed: bool,
	pub released: bool
}
impl ToggleKey {
	fn new() -> Self {
		ToggleKey {
			pressed: false,
			released: true
		}
	}
}

pub struct KeyMap {
	pub left: ToggleKey,
	pub right: ToggleKey,
	pub up: ToggleKey,
	pub down: ToggleKey
}
impl KeyMap {
	pub fn new() -> Self {
		KeyMap {
			left: ToggleKey::new(),
			right: ToggleKey::new(),
			up: ToggleKey::new(),
			down: ToggleKey::new()
		}
	}
}

#[derive(Debug)]
pub struct Vector {
	pub x: f64,
	pub y: f64
}
impl Vector {
	pub fn new(x: f64, y: f64) -> Self {
		Vector {
			x: x,
			y: y
		}
	}
}

pub struct GlyphsStorage {
	pub fira_code_reg: Glyphs,
	pub brandon_blk: Glyphs
}

pub struct RenderContext {
	pub window_size: [f64; 2],
	pub board_size: [f64; 2],
	pub glyphs: GlyphsStorage,
	pub dt: f64,
	pub avg: Vec<f64>,
	pub textures: TextureStorage
}

impl RenderContext {
	pub fn new(window: &mut PistonWindow, glyphs: GlyphsStorage) -> Self {
		RenderContext {
			window_size: [0.0, 0.0],
			board_size: [0.0, 0.0],
			glyphs: glyphs,
			dt: 0.0,
			avg: Vec::with_capacity(FRAME_COUNT_FOR_AVG),
			textures: RenderContext::create_textures(window)
		}
	}

	fn create_textures(window: &mut PistonWindow) -> TextureStorage {
		let mut texture_storage = TextureStorage::new();

		// Create texture making tools
		let mut ctx = window.create_texture_context();
		let settings = TextureSettings::new();

		// Load each tile texture
		for i in 1..=17 {
			if let Ok(texture) = Texture::from_path(&mut ctx, format!("./assets/t{:?}.png", 2i32.pow(i)), Flip::None, &settings) {
				texture_storage.nums.push(texture);
			}
		}

		// Load blank tile texture
		if let Ok(texture) = Texture::from_path(&mut ctx, "./assets/tempty.png", Flip::None, &settings) {
			texture_storage.empty_tile = Some(texture);
		}

		texture_storage
	}
}

// Draw text as multiple lines by splitting by '\n' and drawing each split
pub fn multi_line_text(color: Rgba, font_size: u32, input_text: &str, glyphs: &mut Glyphs, transform: [[f64; 3]; 2], gl: &mut G2d) {
	// Get separate lines of text
	let text_lines: Vec<&str> = input_text.split('\n').collect();

	let mut new_transform = transform;

	// Draw each separate text line
	for line in text_lines {
		text(color, font_size, line, glyphs, new_transform, gl)
			.expect("Failed to draw text");

		// Move line down
		new_transform = new_transform
			.trans(0.0, font_size as f64);
	}
}
use crate::render::Renderable;
use humantime::Duration as HDuration;
use piston_window::*;
use std::time::{Duration, Instant};

pub struct UIState {
	pub ups: i16,
	pub fps: i16,
	pub cache: Glyphs,

	last_second: Instant,
	wip_updates: i16,
	wip_frames: i16,

	time_scale: u64,
	seconds_per_second: u64,

	time_text: TextCache,
	fps_text: TextCache,
	ups_text: TextCache,
}

impl<'a> UIState {
	pub fn new(window: &mut PistonWindow) -> UIState {
		let assets = find_folder::Search::ParentsThenKids(3, 3)
			.for_folder("assets")
			.unwrap();
		let cache = window
			.load_font(assets.join("fonts/DejaVuSansMono.ttf"))
			.unwrap();

		let mut state = UIState {
			last_second: Instant::now(),
			ups: 0,
			fps: 0,
			wip_updates: 0,
			wip_frames: 0,
			cache,
			time_scale: 0,
			seconds_per_second: 0,

			time_text: TextCache::new([1.0; 4], 14, "?/s"),
			fps_text: TextCache::new([1.0; 4], 14, "FPS: -"),
			ups_text: TextCache::new([1.0; 4], 14, "UPS: -"),
		};

		state.update_text();
		state
	}

	pub fn track_timescale(&mut self, seconds_per_second: u64, time_scale: u64) {
		self.seconds_per_second = seconds_per_second;
		self.time_scale = time_scale;

		self.update_text();
	}

	pub fn track_update(&mut self) {
		if self.last_second.elapsed() >= Duration::from_secs(1) {
			self.reset_tracking();
		}
		self.wip_updates += 1;
	}

	pub fn track_frame(&mut self) {
		if self.last_second.elapsed() >= Duration::from_secs(1) {
			self.reset_tracking();
		}
		self.wip_frames += 1;
	}

	fn reset_tracking(&mut self) {
		self.ups = self.wip_updates;
		self.wip_updates = 0;
		self.fps = self.wip_frames;
		self.wip_frames = 0;
		self.update_text();

		self.last_second = Instant::now()
	}

	fn update_text(&mut self) {
		self.fps_text.text = format!("FPS: {}", self.fps);
		self.ups_text.text = format!("UPS: {}", self.ups);

		self.time_text.text = format!("{}/s", self.derive_relative_time());
	}

	fn derive_relative_time(&self) -> HDuration {
		if self.time_scale == 0 {
			return Duration::from_secs(0).into();
		}

		let real_sps = self.seconds_per_second as f64 * self.ups as f64 / self.time_scale as f64;

		// prevent accuracy stutter for < 2 FPS
		if real_sps.fract().abs() < 2.0 / 60.0 {
			return Duration::from_secs_f64(real_sps.trunc()).into();
		}

		// <rel>/s @ actual update/s / updates / s
		Duration::from_secs_f64(real_sps).into()
	}
}

impl Renderable<()> for UIState {
	fn render(&self, _: &(), _context: &Context, _graphics: &mut G2d) {
		panic!("Cannot run UI state without mutable rendering.");
	}

	fn render_mut(&mut self, _world: &(), context: &Context, graphics: &mut G2d) {
		let vp = context.get_view_size();
		let font_size = 14.0;
		let bottom_right_trans = Context::new_abs(vp[0], vp[1])
			.transform
			.trans(vp[0] - 100.0, vp[1] - (font_size + 1.0) * 2.0);

		let bottom_left_trans = Context::new_abs(vp[0], vp[1])
			.transform
			.trans(0.0, vp[1] - font_size + 1.0);

		self.time_text
			.draw(&mut self.cache, &context, bottom_left_trans, graphics);
		self.fps_text
			.draw(&mut self.cache, &context, bottom_right_trans, graphics);
		self.ups_text.draw(
			&mut self.cache,
			&context,
			bottom_right_trans.trans(0.0, 15.0),
			graphics,
		);
	}
}

struct TextCache {
	pub color: [f32; 4],
	pub size: u32,
	pub pos: [f64; 2],
	pub text: String,
	pub obj: Text,
}

impl TextCache {
	pub fn new(color: [f32; 4], size: u32, initial: &str) -> TextCache {
		TextCache {
			color,
			size,
			pos: [0.0; 2],
			text: initial.to_owned(),
			obj: Text::new_color(color, size),
		}
	}

	pub fn draw<C, G>(
		&self,
		cache: &mut C,
		context: &Context,
		transform: graphics::math::Matrix2d,
		g: &mut G,
	) where
		C: graphics::character::CharacterCache,
		G: Graphics<Texture = C::Texture>,
		C::Error: std::fmt::Debug,
	{
		self.obj
			.draw(self.text.as_str(), cache, &context.draw_state, transform, g)
			.unwrap();
	}
}

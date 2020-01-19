mod bodies;
mod render;
mod ui;

use crate::render::Renderable;
use bodies::*;
use log::{debug, info, trace};
use nalgebra::{Matrix3, Point2, Projective2, Vector2};
use piston_window::*;
use pretty_env_logger;
use std::convert::TryInto;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::time::Duration;
use structopt::StructOpt;

const SECONDS_PER_SECOND_MAX: u64 = 60 * 60; // hour
const SECONDS_PER_SECOND_MIN: u64 = 1;

fn main() {
	pretty_env_logger::init();

	debug!("main - initializing window");
	let mut window: PistonWindow = WindowSettings::new("Rusty Bodys", [640, 480])
		.exit_on_esc(true)
		.automatic_close(true)
		.resizable(true)
		.samples(8)
		.build()
		.unwrap();

	let opt = Opt::from_args();

	debug!("main - initializing world");
	let mut world = World::new_from_json(read_to_string(opt.input).expect("Error opening JSON."))
		.expect("Failed parsing world.");

	let mut viewport_transform = recalculate_transform(window.size().into());
	let mut pos = Point2::new(0.0, 0.0);
	let mut panning = false;

	let mut time_scale = opt.base_ticks;
	window.set_event_settings(window.events.get_event_settings().ups(time_scale));
	let mut seconds_per_second = 1;

	let mut ui = ui::UIState::new(&mut window);
	ui.track_timescale(seconds_per_second, time_scale);

	debug!("main - beginning loop");
	while let Some(event) = window.next() {
		event.mouse_cursor(|new_pos| pos = Point2::from(new_pos));
		event.button(|args| {
			if let Button::Mouse(MouseButton::Left) = args.button {
				if args.state == ButtonState::Press {
					debug!("loop - panning start");
					panning = true;
				} else {
					debug!("loop - panning end");
					panning = false;
				}
			}
		});

		// if panning, hook the relative mouse movements to the pan variable;
		if panning {
			event.mouse_relative(|delta| {
				viewport_transform
					.matrix_mut_unchecked()
					.append_translation_mut(&Vector2::from(delta));
				trace!(
					"loop - pan - new transform: {}",
					viewport_transform.matrix()
				);
			});
		}

		event.mouse_scroll(|dir| {
			debug!(
				"loop - zoom - old transform: {}",
				viewport_transform.matrix()
			);
			debug!("loop - zoom - pos: {}", pos);

			let zoom = if dir[1] > 0.0 { 1.25 } else { 1.0 / 1.25 };

			let point_project = pos;
			let vector = Vector2::new(point_project.x, point_project.y);
			debug!("loop - zoom - projected point: {}", vector);
			viewport_transform = Projective2::from_matrix_unchecked(
				viewport_transform
					.matrix_mut_unchecked()
					.append_translation(&-vector)
					.append_scaling(zoom)
					.append_translation(&vector),
			);
			debug!(
				"loop - zoom - new transform: {}",
				viewport_transform.matrix()
			);
		});

		event.text(|s| {
			if s == "+" {
				if seconds_per_second < SECONDS_PER_SECOND_MAX {
					seconds_per_second *= 2;
					if seconds_per_second > SECONDS_PER_SECOND_MAX {
						seconds_per_second = SECONDS_PER_SECOND_MAX;
					}
					info!("loop - seconds per second now {}", seconds_per_second);
				} else {
					time_scale = (time_scale as f64 * 1.5_f64) as u64;
					info!("loop - time scale *= 1.5 ('{}'), now {}", s, time_scale + 1);
				}
			}

			if s == "-" && time_scale > 0 {
				if time_scale == 60 && seconds_per_second > SECONDS_PER_SECOND_MIN {
					{
						seconds_per_second /= 2;
						if seconds_per_second < SECONDS_PER_SECOND_MIN {
							seconds_per_second = SECONDS_PER_SECOND_MIN;
						}
					}
					info!("loop - seconds per second now {}", seconds_per_second);
				} else {
					time_scale = (time_scale as f64 / 1.5_f64) as u64;
					info!("loop - time scale /= 1.5 ('{}'), now {}", s, time_scale - 1);
				}
			}

			window.set_event_settings(window.events.get_event_settings().ups(time_scale));
			ui.track_timescale(seconds_per_second, time_scale);
		});

		event.after_render(|_| ui.track_frame());

		event.update(|args| {
			let elapsed = seconds_per_second as f64 * args.dt;
			/*
			let elapsed = if args.dt < min_step {
				min_step
			} else {
				args.dt
			};
			*/

			// In-order physics state of next frame
			let frames = world
				.entities
				.iter()
				.enumerate()
				.map(|(i, e)| {
					let (l, r) = world.entities.split_at(i);
					e.tick(l.iter().chain(r).collect::<Vec<_>>(), elapsed)
				})
				.collect::<Vec<_>>();

			// apply frames
			world
				.entities
				.iter_mut()
				.zip(frames)
				.map(|(e, f)| e.set(f))
				.for_each(drop); // drain to evaluate lazy iter

			ui.track_update();
		});

		window.draw_2d(&event, |context, graphics, device| {
			clear([0.1, 0.1, 0.1, 1.0], graphics);
			graphics.clear_stencil(0);

			/*
			if sec.elapsed().as_secs_f32() < 1.0 {
				fps += 1;
			} else {
				last_fps = fps;
				sec = Instant::now();
				fps = 0;
			}
			*/

			let ctx = context.append_transform(matrix_to_array(viewport_transform.matrix()));

			for e in &world.entities {
				e.render(&world, &ctx, graphics);
			}

			ui.render_mut(&(), &ctx, graphics);
			ui.cache.factory.encoder.flush(device);
		});
	}
}

#[inline]
fn matrix_to_array(t: &Matrix3<f64>) -> [[f64; 3]; 2] {
	[
		[*t.get(0).unwrap(), *t.get(3).unwrap(), *t.get(6).unwrap()],
		[*t.get(1).unwrap(), *t.get(4).unwrap(), *t.get(7).unwrap()],
	]
}

#[inline]
fn recalculate_transform(size: [f64; 2]) -> Projective2<f64> {
	let window_scale = Vector2::from(get_window_scale(size));
	Projective2::from_matrix_unchecked(Matrix3::new_nonuniform_scaling(&window_scale))
}

#[inline]
fn get_window_scale(size: [f64; 2]) -> [f64; 2] {
	let fb_dims = [5_000_000_000.0 * get_window_ratio(size), 5_000_000_000.0];

	[size[0] / fb_dims[0], size[1] / fb_dims[1]]
}

#[inline]
fn get_window_ratio(size: [f64; 2]) -> f64 {
	size[0] / size[1]
}

#[derive(Debug, StructOpt)]
#[structopt(name = "rusty-planets", about = "Simple orbital simulator.")]
struct Opt {
	/// Set the base number of ticks per second. More ticks leads to a more accurate simulation.
	// we don't want to name it "speed", need to look smart
	#[structopt(short = "t", long = "ticks", default_value = "60")]
	base_ticks: u64,

	/// Input file
	#[structopt(parse(from_os_str), default_value = "sol.json")]
	input: PathBuf,
}

mod bodies;
use bodies::*;
use log::debug;
use nalgebra::{Matrix3, Point2, Projective2, Vector2};
use piston_window::*;
use pretty_env_logger;
use std::convert::TryInto;
use std::time::{Duration, Instant};

fn main() {
    pretty_env_logger::init();

    debug!("main - initializing window");
    let mut window: PistonWindow = WindowSettings::new("Rusty Planets", [640, 480])
        .exit_on_esc(true)
        .automatic_close(true)
        .resizable(true)
        .samples(8)
        .build()
        .unwrap();

    debug!("main - initializing world");
    let mut world = World { entities: vec![] };

    debug!("main - adding entities");
    world.entities.push(Star::new());
    world.entities.push(Planet::new_stable_orbit(PlanetParams {
        parent: &world.entities[0],
        name: "Earth",
        height: 149.6e6,
        mass: 5.9722e24,
        diameter: 12742.0,
        color: [0.2, 0.2, 1.0, 1.0],
        id: world.entities.len(),
    }));

    world.entities.push(Planet::new_stable_orbit(
        // moon
        PlanetParams {
            parent: &world.entities[1],
            name: "Luna",
            height: 0.384e6,
            mass: 0.073e24,
            diameter: 3475.0,
            color: [1.0; 4],
            id: world.entities.len(),
        },
    ));

    world.entities.push(Planet::new_stable_orbit(PlanetParams {
        parent: &world.entities[0],
        name: "Mercury",
        height: 58_000_000.0,
        mass: 3.285e23,
        diameter: 4879.4,
        color: [1.0, 0.4, 0.4, 1.0],
        id: world.entities.len(),
    }));

    world.entities.push(Planet::new_stable_orbit(PlanetParams {
        parent: &world.entities[0],
        name: "Venus",
        height: 108_490_000.0,
        mass: 4.867e24,
        diameter: 12104.0,
        color: [1.0, 1.0, 0.1, 1.0],
        id: world.entities.len(),
    }));

    world.entities.push(Planet::new_stable_orbit(PlanetParams {
        parent: &world.entities[0],
        name: "Mars",
        height: 227.9e6,
        mass: 0.642e24,
        diameter: 6792.0,
        color: [1.0, 0.1, 0.1, 1.0],
        id: world.entities.len(),
    }));

    world.entities.push(Planet::new_stable_orbit(PlanetParams {
        parent: &world.entities[0],
        name: "Jupiter",
        height: 778.6e6,
        mass: 1898e24,
        diameter: 142_984.0,
        color: [1.0, 0.8, 0.0, 1.0],
        id: world.entities.len(),
    }));

    world.entities.push(Planet::new_stable_orbit(PlanetParams {
        parent: &world.entities[0],
        name: "Saturn",
        height: 1433.5e6,
        mass: 568e24,
        diameter: 120_536.0,
        color: [1.0, 0.5, 0.0, 1.0],
        id: world.entities.len(),
    }));

    world.entities.push(Planet::new_stable_orbit(PlanetParams {
        parent: &world.entities[0],
        name: "Neptune",
        height: 2872.5e6,
        mass: 86.8e24,
        diameter: 51_118.0,
        color: [0.45, 0.45, 0.7, 1.0],
        id: world.entities.len(),
    }));

    world.entities.push(Planet::new_stable_orbit(PlanetParams {
        parent: &world.entities[0],
        name: "Uranus",
        height: 4495.1e6,
        mass: 102.0e24,
        diameter: 49_528.0,
        color: [0.4, 0.4, 0.9, 1.0],
        id: world.entities.len(),
    }));
    let mut sec = Instant::now();
    let mut fps = 0;
    let mut last_fps = 0;
    let mut time_scale = window.events.get_event_settings().ups;

    let mut viewport_transform = recalculate_transform(window.size().into());
    let mut pos = Point2::new(0.0, 0.0);
    let mut panning = false;
    let min_step = (Duration::from_secs(1) / time_scale.try_into().unwrap()).as_secs_f64();

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
                debug!(
                    "loop - pan - old transform: {}",
                    viewport_transform.matrix()
                );
                viewport_transform
                    .matrix_mut_unchecked()
                    .append_translation_mut(&Vector2::from(delta));
                debug!(
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

            let zoom = if dir[1] > 0.0 { 1.5 } else { 1.0 / 1.5 };

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
                debug!("loop - time scale +1 ('{}'), now {}", s, time_scale + 1);
                time_scale *= 2;
            }

            if s == "-" && time_scale > 0 {
                time_scale /= 2;
                debug!("loop - time scale -1 ('{}'), now {}", s, time_scale - 1);
            }
            window.set_event_settings(window.events.get_event_settings().ups(time_scale));
        });

        event.update(|args| {
            let elapsed = if args.dt < min_step {
                min_step
            } else {
                args.dt
            };

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
        });

        window.draw_2d(&event, |context, graphics, _device| {
            clear([0.1, 0.1, 0.1, 1.0], graphics);
            graphics.clear_stencil(0);

            if sec.elapsed().as_secs_f32() < 1.0 {
                fps += 1;
            } else {
                last_fps = fps;
                sec = Instant::now();
                fps = 0;
            }

            let ctx = context.append_transform(matrix_to_array(viewport_transform.matrix()));

            for e in &world.entities {
                e.render(&world, &ctx, &viewport_transform, graphics);
            }
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

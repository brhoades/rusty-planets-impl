mod bodies;
use piston_window::*;
use bodies::{World,Planet,Star};
use std::time::{Instant,Duration};
use nalgebra::{Point3, Point2, Vector2, Vector3, Matrix, Matrix3, U3, U2, Projective2};
use pretty_env_logger;
use log::{debug};

fn main() {
    pretty_env_logger::init();

    debug!("main - initializing window");
    let mut window: PistonWindow =
        WindowSettings::new("Rusty Planets", [640, 480])
        .exit_on_esc(true)
        .automatic_close(true)
        .resizable(true)
        .samples(8)
        .build()
        .unwrap();

    debug!("main - initializing world");
    let mut world = World{entities: vec!()};

    debug!("main - adding entities");
    world.entities.push(Star::new());
    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            149.6e6,
            5.9722e24,
            12742.0,
            [0.2, 0.2, 1.0, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit( // moon
            &world.entities[1],
            0.384e6,
            0.073e24,
            3475.0,
            [1.0; 4],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            58_000_000.0,
            3.285e23,
            4879.4,
            [1.0, 0.4, 0.4, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            108_490_000.0,
            4.867e24,
            12104.0,
            [1.0, 1.0, 0.1, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            227.9e6,
            0.642e24,
            6792.0,
            [1.0, 0.1, 0.1, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            778.6e6,
            1898e24,
            142_984.0,
            [1.0, 0.8, 0.0, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            1433.5e6,
            568e24,
            120_536.0,
            [1.0, 0.5, 0.0, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            2872.5e6,
            86.8e24,
            51_118.0,
            [0.45, 0.45, 0.7, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            4495.1e6,
            102.0e24,
            49_528.0,
            [0.4, 0.4, 0.9, 1.0],
        )
    );
    let mut sec = Instant::now();
    let mut last = Instant::now();
    let mut fps = 0;
    let mut last_fps = 0;
    let mut time_scale = 1;


    let mut viewport_transform = recalculate_transform(window.size().into());
    let mut pos = Point2::new(0.0, 0.0);
    let mut panning = false;

    debug!("main - beginning loop");
    while let Some(event) = window.next() {
        event.mouse_cursor(|new_pos| pos = Point2::from(new_pos));
        event.button(|args| {
            match args.button {
                Button::Mouse(MouseButton::Left) => {
                    if args.state == ButtonState::Press {
                        println!("loop - panning start");
                        panning = true;
                    } else {
                        println!("loop - panning end");
                        panning = false;
                    }
                },
                _ => ()
            }
        });

        // if panning, hook the relative mouse movements to the pan variable;
        if panning {
            event.mouse_relative(|delta| {
                debug!("loop - pan - old transform: {}", viewport_transform.matrix());
                viewport_transform.matrix_mut_unchecked().append_translation_mut(&Vector2::from(delta));
                debug!("loop - pan - new transform: {}", viewport_transform.matrix());
            });
        }

        event.mouse_scroll(|dir| {
            debug!("loop - zoom - old transform: {}", viewport_transform.matrix());
            debug!("loop - zoom - pos: {}", pos);

            let zoom = if dir[1] > 0.0 {
                1.5
            } else {
                1.0/1.5
            };

            let point_project = pos;
            let vector = Vector2::new(point_project.x, point_project.y);
            debug!("loop - zoom - projected point: {}", vector);
            viewport_transform = Projective2::from_matrix_unchecked(
                viewport_transform
                    .matrix_mut_unchecked()
                    .append_translation(&-vector)
                    .append_scaling(zoom)
                    .append_translation(&vector)
            );
            debug!("loop - zoom - new transform: {}", viewport_transform.matrix());
        });

        event.text(|s| {
            if s == "+" {
                debug!("loop - time scale +1 ('{}'), now {}", s, time_scale + 1);
                time_scale += 1;
            }

            if s == "-" {
                if time_scale > 0 {
                    debug!("loop - time scale -1 ('{}'), now {}", s, time_scale - 1);
                    time_scale -= 1;
                }
            }
        });

        let elapsed = last.elapsed() * time_scale;

        // In-order physics state of next frame
        let frames = world.entities
            .iter()
            .map(|e| e.tick(&world, elapsed))
            .collect::<Vec<_>>();

        // apply frames
        world.entities
            .iter_mut()
            .zip(frames)
            .map(|(e, f)| e.set(f))
            .for_each(drop); // drain to evaluate lazy iter

        window.draw_2d(&event, |context, graphics, _device| {
            clear([0.1, 0.1, 0.1, 1.0], graphics);

            if sec.elapsed().as_secs_f32() < 1.0 {
                fps += 1;
            } else {
                last_fps = fps;
                sec = Instant::now();
                fps = 0;
            }

            let ctx = context.append_transform(matrix_to_array(viewport_transform.matrix()));

            for e in &world.entities {
                e.render(&ctx, graphics);
            }
        });

        last = Instant::now();
    }
}

#[inline]
fn matrix_to_array(t: &Matrix3<f64>) -> [[f64; 3]; 2] {
    [[t.get(0).unwrap().clone(), t.get(3).unwrap().clone(), t.get(6).unwrap().clone()], [t.get(1).unwrap().clone(), t.get(4).unwrap().clone(), t.get(7).unwrap().clone()]]
}

#[inline]
fn recalculate_transform(size: [f64; 2]) -> Projective2<f64> {
    let window_scale = Vector2::from(get_window_scale(size));
    Projective2::from_matrix_unchecked(Matrix3::new_nonuniform_scaling(&window_scale))
}

#[inline]
fn get_window_scale(size: [f64; 2]) -> [f64; 2] {
    let fb_dims = [5_000_000_000.0 * get_window_ratio(size), 5_000_000_000.0];

    return [size[0] / fb_dims[0], size[1] / fb_dims[1]];
}

#[inline]
fn get_window_ratio(size: [f64; 2]) -> f64 {
    return size[0] / size[1];
}

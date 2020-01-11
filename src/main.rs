mod bodies;
use piston_window::*;
use bodies::{World,Planet,Star};
use std::time::{Instant,Duration};

const STEP: Duration = Duration::from_millis(1000/60);

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Rusty Planets", [640, 480])
        .exit_on_esc(true)
        .automatic_close(true)
        .resizable(true)
        .build()
        .unwrap();

    let mut world = World{entities: vec!()};

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
    let mut last = Instant::now();
    let mut scale = get_window_scale(window.size());
    let mut time_scale = 1;

    while let Some(event) = window.next() {
        let mut zoom = None;
        match &event {
            Event::Input(Input::Move(Motion::MouseScroll(pos)), opt) => {
                zoom = Some(pos);
            },
            Event::Input(Input::Text(s), opt) => {
                if s == "+" {
                    time_scale += 1;
                }

                if s == "-" {
                    time_scale -= 1;
                }
            },
            _ => (),
        }

        std::thread::sleep(STEP);
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
            match zoom {
                Some(p) => {
                    context.trans(p[0], p[1]);
                    scale[0] += p[0].signum() * 1.28e-4;
                    scale[1] += p[1].signum() * 1.28e-4;
                },
                None => (),
            }
            let ctx = context.scale(scale[0], scale[1]);

            for e in &world.entities {
                e.render(&ctx, graphics);
            }
        });

        last = Instant::now();
    }
}

#[inline]
fn get_window_scale(size: Size) -> [f64; 2] {
    return [size.width / 5_000_000_000.0, size.height / 5_000_000_000.0];
}

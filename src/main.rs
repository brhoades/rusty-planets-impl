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
        .build()
        .unwrap();

    let mut world = World{entities: vec!()};
    world.entities.push(Star::new(window.size()));
    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            100.0,
            0.1,
            1.0,
            5.0,
            [0.0, 0.0, 1.0, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[1],
            10.0,
            0.0,
            0.001,
            2.5,
            [1.0; 4],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            30.0,
            0.9,
            1.0,
            2.5,
            [1.0, 0.4, 0.4, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            60.0,
            0.7,
            1.0,
            4.0,
            [1.0, 1.0, 0.1, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            150.0,
            0.2,
            1.0,
            4.0,
            [1.0, 0.1, 0.1, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            200.0,
            0.8,
            10.0,
            8.0,
            [1.0, 0.8, 0.0, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            275.0,
            0.4,
            8.0,
            6.0,
            [1.0, 0.5, 0.0, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            350.0,
            0.1,
            4.0,
            4.0,
            [0.45, 0.45, 0.7, 1.0],
        )
    );

    world.entities.push(
        Planet::new_stable_orbit(
            &world.entities[0],
            425.0,
            0.5,
            4.0,
            4.0,
            [0.4, 0.4, 0.9, 1.0],
        )
    );
    let mut last = Instant::now();

    while let Some(event) = window.next() {
        std::thread::sleep(STEP);
        let elapsed = last.elapsed();

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
            for e in &world.entities {
                e.render(&context, graphics);
            }
        });

        last = Instant::now();
    }
}

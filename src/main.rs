mod planet;
use piston_window::*;
use planet::{World,PhysicsFrame,Planet,Star};

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Rusty Planets", [640, 480])
        .exit_on_esc(true).build().unwrap();

    let mut world = World{entities: vec!()};
    world.entities.push(Star::new(window.size()));

    while let Some(event) = window.next() {
        // In-order physics state of next frame
        let frames = world.entities
            .iter()
            .map(|e| e.tick(&world))
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

            // world was moved in, now move it out of the closure
            // (try removing this)
        });
    }
}

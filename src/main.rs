mod planet;
use piston_window::*;
use planet::{World,PhysicsFrame};

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Hello Piston!", [640, 480])
        .exit_on_esc(true).build().unwrap();

    let mut world = World{entities: vec!()};

    while let Some(event) = window.next() {
        world = window.draw_2d(&event, |context, graphics, _device| {
            clear([0.1, 0.1, 0.1, 1.0], graphics);

            let frames = world.entities.iter().map(|e| e.tick(&world)).collect::<Vec<PhysicsFrame>>();

            for (i, e) in world.entities.iter_mut().enumerate() {
                e.set(&frames[i]);
            }

            for e in &world.entities {
                e.render(&context, graphics);
            }

            // world was moved in and now out of closure
            world
        }).expect("Error rendering world.");
    }
}

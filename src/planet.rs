use piston_window::*;

pub trait Renderable {
    fn render(&self, ctx: &Context, graphics: &mut G2d);
}

pub trait PhysicsBody {
    fn position(&self) -> Motion;
    fn tick(&self, w: &World) -> PhysicsFrame;
    fn set(&mut self, f: PhysicsFrame);
}

pub type PhysicsFrame = Motion;

pub struct Motion {
    velocity: [f64; 2],
    position: [f64; 2],
}

pub trait Entity: PhysicsBody + Renderable {
    fn name(&self) -> &'static str;
}

pub struct World {
    pub entities: Vec<Box<dyn Entity>>
}

pub struct Planet {
    velocity: [f64; 2],
    position: [f64; 2],
    color: [f32; 4],
    size: f64,
}

impl Planet {
    // makes a new planet that's (theoretically) stable around other at height at a period (% from 0 degrees).
    fn new_stable_orbit(other: &Box<dyn PhysicsBody>, height: f64, period: f64, size: f64, color: [f32; 4]) -> Self {
        let oPos = other.position();

        Planet{
            velocity: [1.0, 0.0],
            position: [oPos.position[0] + height, oPos.position[1] + height],
            color,
            size,
        }
    }
}

fn get_centered_square_extents(center: &[f64; 2], size: f64) -> [f64; 4] {
    return ellipse::centered([center[0], center[1], size / 2.0, size / 2.0]);
}

impl Renderable for Planet {
    fn render(&self, context: &Context, graphics: &mut G2d) {
        let extents = ellipse::circle(self.position[0], self.position[1], self.size);
        rectangle(
            self.color,
            extents,
            context.transform,
            graphics
        );
    }
}

impl PhysicsBody for Planet {
    fn tick(&self, w: &World) -> PhysicsFrame {
        PhysicsFrame {
            velocity: self.velocity,
            position: self.position,
        }
    }

    fn set(&mut self, f: PhysicsFrame) {
        self.position = f.position;
        self.velocity = f.velocity;
    }

    fn position(&self) -> Motion {
        Motion {
            position: self.position,
            velocity: [0.0, 0.0],
        }
    }
}

pub struct Star {
    position: [f64; 2],
    color: [f32; 4],
    size: f64,
}

impl Star {
    pub fn new(window_size: Size) -> Box<Star> {
        Box::new(Star{
            position: [window_size.width / 2.0, window_size.height / 2.0],
            color: [1.0, 0.5, 0.5, 1.0],
            size: 10.0,
        })
    }
}

impl PhysicsBody for Star {
    // Let's pretend the star doesn't move
    fn tick(&self, _w: &World) -> PhysicsFrame {
        PhysicsFrame {
            velocity: [0.0, 0.0],
            position: self.position,
        }
    }

    fn set(&mut self, _f: PhysicsFrame) {}

    fn position(&self) -> Motion {
        Motion {
            position: self.position,
            velocity: [0.0, 0.0],
        }
    }
}

impl Renderable for Star {
    fn render(&self, context: &Context, graphics: &mut G2d) {
        let extents = ellipse::circle(self.position[0], self.position[1], self.size);
        rectangle(
            self.color,
            extents,
            context.transform,
            graphics
        );
    }
}

impl Entity for Star {
    fn name(&self) -> &'static str {
        "Star"
    }
}

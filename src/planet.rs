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
    mass: f64,
}

impl Planet {
    // makes a new planet that's (theoretically) stable around other at height at a period (% from 0 degrees).
    pub fn new_stable_orbit(other: &Box<dyn Entity>, height: f64, period: f64, mass: f64, size: f64, color: [f32; 4]) -> Box<Self> {
        let o_pos = other.position();
        let mu = 6.67430e-11*size;
        let v = (mu*(2.0 / height - 1.0 / height)).sqrt();

        Box::new(Planet{
            velocity: [v, 0.0],
            position: [o_pos.position[0] + height / 2.0, o_pos.position[1] + height],
            color,
            size,
            mass,
        })
    }
}

impl Entity for Planet {
    fn name(&self) -> &'static str {
        "planet"
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
            position: [self.position[0] + self.velocity[0], self.position[1] + self.velocity[1]],
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
    mass: f64,
    size: f64,
}

impl Star {
    pub fn new(window_size: Size) -> Box<Star> {
        Box::new(Star{
            position: [window_size.width / 2.0, window_size.height / 2.0],
            color: [1.0, 0.5, 0.5, 1.0],
            mass: 1.989e30,
            size: 20.0,
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

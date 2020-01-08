use piston_window::*;
use std::time::Duration;
use nalgebra::Point2;

pub trait Renderable {
    fn render(&self, ctx: &Context, graphics: &mut G2d);
}

pub trait PhysicsBody {
    fn motion(&self) -> Motion;
    fn mass(&self) -> f64;

    fn tick(&self, _w: &World, d: Duration) -> PhysicsFrame;
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

// const G: f64 = 6.67430e-11;
 const G: f64 = 0.001;

impl Planet {
    // makes a new planet that's (theoretically) stable around other at height at a period (% from 0 degrees).
    pub fn new_stable_orbit(other: &Box<dyn Entity>, height: f64, period: f64, mass: f64, size: f64, color: [f32; 4]) -> Box<Self> {
        let o_pos = other.motion();
        let mu = G * other.mass();
        let v = (mu / height).sqrt();

        Box::new(Planet{
            velocity: [0.0, v],
            position: [o_pos.position[0] + height, o_pos.position[1] + height / 2.0],
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
    fn tick(&self, w: &World, d: Duration) -> PhysicsFrame {
        let mut dv = [0.0; 2];

        for e in &w.entities {
            let o_mass = e.mass();
            let pos = Point2::from(self.motion().position);
            let o_pos = Point2::from(e.motion().position);
            let vec = pos - o_pos;
            let dist = vec.norm_squared();

            let g_mass = - G * o_mass * self.mass();

            let force = g_mass / dist * vec.normalize();

            if force[0].is_finite() {
                println!("x: {:?}", force);
                dv[0] += force[0];
            }

            if force[1].is_finite() {
                println!("y: {:?}", force);
                dv[1] += force[1];
            }
        }

        PhysicsFrame {
            velocity: [self.velocity[0] + dv[0], self.velocity[1] + dv[1]],
            position: [self.position[0] + self.velocity[0], self.position[1] + self.velocity[1]],
        }
    }

    fn set(&mut self, f: PhysicsFrame) {
        self.position = f.position;
        self.velocity = f.velocity;
    }

    fn motion(&self) -> Motion {
        Motion {
            position: self.position,
            velocity: [0.0, 0.0],
        }
    }

    fn mass(&self) -> f64 {
        self.mass
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
            mass: 1000.0,
            size: 20.0,
        })
    }
}

impl PhysicsBody for Star {
    // Let's pretend the star doesn't move
    fn tick(&self, _w: &World, d: Duration) -> PhysicsFrame {
        PhysicsFrame {
            velocity: [0.0, 0.0],
            position: self.position,
        }
    }

    fn set(&mut self, _f: PhysicsFrame) {}

    fn motion(&self) -> Motion {
        Motion {
            position: self.position,
            velocity: [0.0, 0.0],
        }
    }

    fn mass(&self) -> f64 {
        self.mass
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

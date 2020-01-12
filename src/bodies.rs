use piston_window::*;
use std::time::Duration;
use nalgebra::{RealField,Point2,Vector2,Projective2};
use rand::prelude::*;
use log::{debug};

pub trait Renderable {
    fn render(&self, context: &Context, transform: &Projective2<f64>, graphics: &mut G2d);
}

pub trait PhysicsBody {
    fn motion(&self) -> Motion;
    fn mass(&self) -> f64;

    fn tick(&self, _w: &World, d: Duration) -> PhysicsFrame;
    fn set(&mut self, f: PhysicsFrame);
}

pub type PhysicsFrame = Motion;

#[derive(Debug)]
pub struct Motion {
    velocity: Vector2<f64>,
    position: Point2<f64>,
}

pub trait Entity: PhysicsBody + Renderable {
    fn id(&self) -> u32;
    fn name(&self) -> &'static str;
}

pub struct World {
    pub entities: Vec<Box<dyn Entity>>
}

#[derive(Debug)]
pub struct Planet {
    velocity: Vector2<f64>,
    position: Point2<f64>,
    color: [f32; 4],
    size: f64,
    mass: f64,
    id: u32,
}

const G: f64 = 6.67430e-11;

const X_SIZE: f64 = 5_000_000_000.0;
const Y_SIZE: f64 = X_SIZE;

impl Planet {
    // makes a new planet that's (theoretically) stable around other at height at a period (% from 0 degrees).
    pub fn new_stable_orbit(other: &Box<dyn Entity>, height: f64, mass: f64, size: f64, color: [f32; 4]) -> Box<Self> {
        let o_pos = other.motion().position;
        let period: f64 = rand::thread_rng().gen_range(0.0, 2.0);
        let orbit_vec = nalgebra::Rotation2::new(f64::pi() * period) * Vector2::from([height, height / 2.0]);
        let position = Point2::from(o_pos) + orbit_vec;

        let mu = G * (other.mass() + mass);
        let f_vec = o_pos - position;
        let velocity = (mu / f_vec.norm()).sqrt() * (nalgebra::Rotation2::new(-f64::frac_pi_2()) * f_vec.normalize());

        // add parent velocity
        let velocity = other.motion().velocity + velocity;

        Box::new(Planet{
            id: rand::thread_rng().gen(),
            velocity,
            position,
            color,
            size,
            mass,
        })
    }
}

impl Entity for Planet {
    fn id(&self) -> u32 {
        return self.id
    }

    fn name(&self) -> &'static str {
        "planet"
    }
}

impl Renderable for Planet {
    fn render(&self, context: &Context, transform: &Projective2<f64>, graphics: &mut G2d) {
        let scale = transform.matrix().get(0).unwrap();
        let scale_zoom = 5_000_000_000.0 * scale;
        let mut ellipse = Ellipse::new(self.color);
        let size = if scale_zoom > 100_000.0 {
            if self.size < 10_000_000.0 {
                self.size * 10.0
            } else {
                self.size
            }
        } else if self.size < 10_000_000.0 {
            ellipse = Ellipse::new_border(self.color, 0.45 / scale);
            5.0 / scale // statically sized placeholder
        } else {
            self.size
        };

        let extents = ellipse::circle(self.position[0], self.position[1], size);


        graphics.ellipse(&ellipse, extents, &context.draw_state, context.transform);
    }
}

impl PhysicsBody for Planet {
    fn tick(&self, w: &World, d: Duration) -> PhysicsFrame {
        let mut dv: Vector2<f64> = Vector2::from([0.0; 2]);

        for e in &w.entities {
            if e.id() == self.id() {
                continue;
            }

            let pos = self.motion().position;
            let o_pos = e.motion().position;
            let mass = e.mass() + self.mass();

            let vec = o_pos - pos;
            let r_sq = vec.norm_squared();

            let force = (G * mass) / r_sq * vec.normalize();
            dv += force;
        }

        PhysicsFrame {
            velocity: self.velocity + dv * d.as_secs_f64(),
            position: self.position + self.velocity * d.as_secs_f64(),
        }
    }

    fn set(&mut self, f: PhysicsFrame) {
        self.position = f.position;
        self.velocity = f.velocity;
    }

    fn motion(&self) -> Motion {
        Motion {
            position: self.position,
            velocity: self.velocity,
        }
    }

    fn mass(&self) -> f64 {
        self.mass
    }
}

pub struct Star {
    position: Point2<f64>,
    color: [f32; 4],
    mass: f64,
    size: f64,
}

impl Star {
    pub fn new() -> Box<Star> {
        Box::new(Star{
            position: Point2::from([X_SIZE / 2.0, Y_SIZE / 2.0]),
            color: [1.0, 1.0, 0.8, 1.0],
            mass: 1.989e30,
            size: 50_000_000.0, // size: 1_391_000.0,
        })
    }
}

impl PhysicsBody for Star {
    // Let's pretend the star doesn't move
    fn tick(&self, _w: &World, _d: Duration) -> PhysicsFrame {
        PhysicsFrame {
            velocity: Vector2::from([0.0, 0.0]),
            position: self.position,
        }
    }

    fn set(&mut self, _f: PhysicsFrame) {}

    fn motion(&self) -> Motion {
        Motion {
            position: self.position,
            velocity: Vector2::from([0.0, 0.0]),
        }
    }

    fn mass(&self) -> f64 {
        self.mass
    }
}

impl Renderable for Star {
    fn render(&self, context: &Context, transform: &Projective2<f64>, graphics: &mut G2d) {
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
    fn id(&self) -> u32 {
        return 1
    }

    fn name(&self) -> &'static str {
        "Star"
    }
}

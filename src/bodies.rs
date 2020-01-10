use piston_window::*;
use std::time::Duration;
use nalgebra::{RealField,Point2,Vector2};
use rand::prelude::*;

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
const SMALL_SIZE_SCALE_FACTOR: f64 = 15_000_000.0 / 12_756.0;
const MED_SIZE_SCALE_FACTOR: f64 = 30_000_000.0 / 49_528.0;
const LARGE_SIZE_SCALE_FACTOR: f64 = 45_000_000.0 / 142_984.0;

// Scale size. 100M km is the visability for the star.
// around 10M km gets you a pixel. Scale by fraction of max size getting 50M.
fn scale_size(size_in: f64) -> f64 {
    if size_in < 15_000.0 {
        return size_in * SMALL_SIZE_SCALE_FACTOR;
    }

    if size_in < 50_000.0 {
        return size_in * MED_SIZE_SCALE_FACTOR;
    }

    return size_in * LARGE_SIZE_SCALE_FACTOR;
}

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
        // let velocity = other.motion().velocity + v_vec;

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
    fn render(&self, context: &Context, graphics: &mut G2d) {
        let extents = ellipse::circle(self.position[0], self.position[1], scale_size(self.size));

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
    fn id(&self) -> u32 {
        return 1
    }

    fn name(&self) -> &'static str {
        "Star"
    }
}

use log::debug;
use nalgebra::{Point2, Projective2, RealField, Rotation2, Vector2};
use piston_window::*;
use rand::prelude::*;
use std::time::Duration;

pub trait Renderable {
    fn render(
        &self,
        world: &World,
        context: &Context,
        transform: &Projective2<f64>,
        graphics: &mut G2d,
    );
}

pub trait PhysicsBody {
    fn motion(&self) -> Motion;
    fn mass(&self) -> f64;
    fn size(&self) -> f64;

    fn tick(&self, others: Vec<&Box<dyn Entity>>, d: Duration) -> PhysicsFrame;
    fn set(&mut self, f: PhysicsFrame);
}

pub type PhysicsFrame = Motion;

#[derive(Debug)]
pub struct Motion {
    velocity: Vector2<f64>,
    position: Point2<f64>,
}

pub trait Entity: PhysicsBody + Renderable {
    fn id(&self) -> usize;
    fn name(&self) -> &'static str;
}

pub struct World {
    pub entities: Vec<Box<dyn Entity>>,
}

#[derive(Debug)]
pub struct Planet {
    velocity: Vector2<f64>,
    position: Point2<f64>,
    color: [f32; 4],
    size: f64,
    mass: f64,
    parent_id: usize,
    name: &'static str,
    id: usize,
}

pub struct PlanetParams<'a> {
    pub parent: &'a Box<dyn Entity>,
    pub color: [f32; 4],
    pub diameter: f64,
    pub mass: f64,
    pub height: f64,
    pub name: &'static str,
    pub id: usize,
}

const G: f64 = 6.67430e-11;

const X_SIZE: f64 = 5_000_000_000.0;
const Y_SIZE: f64 = X_SIZE;

impl Planet {
    // makes a new planet that's (theoretically) stable around other at height at a period (% from 0 degrees).
    pub fn new_stable_orbit(params: PlanetParams) -> Box<Self> {
        let parent = params.parent;
        let o_pos = parent.motion().position;
        let period: f64 = rand::thread_rng().gen_range(0.0, 2.0);
        let orbit_vec = nalgebra::Rotation2::new(f64::pi() * period)
            * Vector2::from([params.height, params.height / 2.0]);
        let position = Point2::from(o_pos) + orbit_vec;

        let mu = G * (parent.mass() + params.mass);
        let f_vec = o_pos - position;
        let velocity = (mu / f_vec.norm()).sqrt()
            * (nalgebra::Rotation2::new(-f64::frac_pi_2()) * f_vec.normalize());

        // add parent velocity
        let velocity = parent.motion().velocity + velocity;

        Box::new(Planet {
            id: params.id,
            parent_id: parent.id(),
            velocity,
            position,
            color: params.color,
            size: params.diameter,
            mass: params.mass,
            name: params.name,
        })
    }

    fn get_offsets(
        &self,
        world: &World,
        transform: [[f64; 3]; 2],
        world_transform: &Projective2<f64>,
        size: f64,
    ) -> ([[f32; 4]; 3], [[[f64; 3]; 2]; 3]) {
        // cheater shadow from the parent.
        let parent = &world.entities.get(self.parent_id).unwrap();
        let vec = self.position - parent.motion().position;
        let unit_vec: Vector2<f64> = vec.normalize();
        let wrot: Rotation2<f64> =
            Rotation2::from_matrix(&world_transform.matrix().fixed_resize(2.0));

        // highlights reduce with the inverse square of the distance
        //let strength = parent.size() / (f64::pi() * 4.0 * vec.norm_squared());
        let strength_f =
            |a| 2.0 / 3.0 * ((1.0 - a / f64::pi()) * f64::cos(a) + 1.0 / f64::pi() * f64::sin(a));
        let strength = strength_f(f64::pi() / 2.0);
        debug!("Strength for: {}, {}", self.name(), strength);

        (
            [
                self.color,
                [1.0, 1.0, 1.0, strength as f32],
                [0.0, 0.0, 0.0, strength as f32],
            ],
            [
                transform,
                transform.trans_pos(-0.75 * size * unit_vec),
                transform.trans_pos(0.75 * size * unit_vec),
            ],
        )
    }

    fn render_scaled(
        &self,
        world: &World,
        context: &Context,
        world_transform: &Projective2<f64>,
        graphics: &mut G2d,
        scale: f64,
    ) {
        let extents = ellipse::circle(0.0, 0.0, 5.0 / scale); // statically sized placeholder
        let transform = context.transform.trans(self.position[0], self.position[1]);

        Rectangle::new([0.0, 0.0, 0.0, 1.0]).draw(
            extents,
            &DrawState::new_clip(),
            transform,
            graphics,
        );

        let (colors, transes) = self.get_offsets(world, transform, world_transform, 5.0 / scale);
        colors
            .iter()
            .zip(transes.iter())
            .into_iter()
            .map(|(color, trans)| {
                Rectangle::new(*color).draw(
                    extents,
                    &DrawState::new_inside(),
                    trans.clone(),
                    graphics,
                )
            })
            .for_each(drop);
    }

    fn render_real(
        &self,
        world: &World,
        context: &Context,
        world_transform: &Projective2<f64>,
        graphics: &mut G2d,
    ) {
        let transform = context.transform.trans(self.position[0], self.position[1]);
        let size = if self.size < 10_000_000.0 {
            self.size * 10.0
        } else {
            self.size
        };

        let extents = ellipse::circle(0.0, 0.0, size);
        Ellipse::new([0.0, 0.0, 0.0, 1.0]).draw(
            extents,
            &DrawState::new_clip(),
            transform,
            graphics,
        );

        let (colors, transes) = self.get_offsets(world, transform, world_transform, size);
        colors
            .iter()
            .zip(transes.iter())
            .map(|(color, trans)| {
                Rectangle::new(*color).draw(
                    extents,
                    &DrawState::new_inside(),
                    trans.clone(),
                    graphics,
                )
            })
            .for_each(drop);
    }
}

impl Entity for Planet {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> &'static str {
        "planet"
    }
}

impl Renderable for Planet {
    fn render(
        &self,
        world: &World,
        context: &Context,
        transform: &Projective2<f64>,
        graphics: &mut G2d,
    ) {
        let scale = transform.matrix().get(0).unwrap();
        let scale_zoom = 5_000_000_000.0 * scale;
        if scale_zoom > 100_000.0 {
            self.render_real(world, &context, &transform, graphics);
        } else if self.size < 10_000_000.0 {
            self.render_scaled(world, &context, &transform, graphics, scale.clone());
        } else {
            self.render_real(&world, context, &transform, graphics);
        };
    }
}

impl PhysicsBody for Planet {
    fn tick(&self, others: Vec<&Box<dyn Entity>>, d: Duration) -> PhysicsFrame {
        let mut dv: Vector2<f64> = Vector2::from([0.0; 2]);

        for e in others {
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

    fn size(&self) -> f64 {
        self.size
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
        Box::new(Star {
            position: Point2::from([X_SIZE / 2.0, Y_SIZE / 2.0]),
            color: [1.0, 1.0, 0.8, 1.0],
            mass: 1.989e30,
            size: 50_000_000.0, // size: 1_391_000.0,
        })
    }
}

impl PhysicsBody for Star {
    // Let's pretend the star doesn't move
    fn tick(&self, _others: Vec<&Box<dyn Entity>>, _d: Duration) -> PhysicsFrame {
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

    fn size(&self) -> f64 {
        self.size
    }
}

impl Renderable for Star {
    fn render(
        &self,
        _world: &World,
        context: &Context,
        transform: &Projective2<f64>,
        graphics: &mut G2d,
    ) {
        let extents = ellipse::circle(self.position[0], self.position[1], self.size);

        rectangle(self.color, extents, context.transform, graphics);
    }
}

impl Entity for Star {
    fn id(&self) -> usize {
        0 // TODO: this will break some day
    }

    fn name(&self) -> &'static str {
        "Star"
    }
}

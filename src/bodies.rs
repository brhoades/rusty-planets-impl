use log::debug;
use nalgebra::{Point2, RealField, Vector2};
use piston_window::*;
use rand::prelude::*;
use serde::Deserialize;
use std::convert::TryInto;

pub trait Renderable {
    fn render(&self, world: &World, context: &Context, graphics: &mut G2d);
}

pub trait PhysicsBody {
    fn physics_data(&self) -> PhysicsData;

    fn tick(&self, others: Vec<&Box<dyn Entity>>, dt: f64) -> PhysicsFrame;
    fn set(&mut self, f: PhysicsFrame);
}

pub struct PhysicsFrame {
    velocity: Vector2<f64>,
    position: Point2<f64>,
}

#[derive(Debug)]
pub struct PhysicsData {
    velocity: Vector2<f64>,
    position: Point2<f64>,
    mass: f64,
    size: f64,
}

pub trait Entity: PhysicsBody + Renderable {
    fn id(&self) -> usize;
    fn name(&self) -> String;
}

pub struct World {
    pub entities: Vec<Box<dyn Entity>>,
}

#[derive(Debug)]
pub struct Body {
    velocity: Vector2<f64>,
    position: Point2<f64>,
    color: [f32; 4],
    size: f64,
    mass: f64,
    parent_id: usize,
    name: String,
    id: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WorldParams {
    stars: Vec<BodyParams>,
    planets: Vec<BodyParams>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BodyParams {
    #[serde(skip)]
    id: usize,
    #[serde(skip)]
    parent_id: usize,
    color: [f32; 4],
    diameter: f64,
    mass: f64,
    height: f64,
    name: String,
    #[serde(default)]
    children: Option<Vec<BodyParams>>,
}

const G: f64 = 6.67430e-11;

const X_SIZE: f64 = 5_000_000_000.0;
const Y_SIZE: f64 = X_SIZE;

impl Body {
    // makes a new planet that's (theoretically) stable around other at height at a period (% from 0 degrees).
    pub fn new_stable_orbit<'a>(
        parent_physics: &PhysicsData,
        params: BodyParams,
    ) -> Box<dyn Entity> {
        let o_pos = parent_physics.position;
        let period: f64 = rand::thread_rng().gen_range(0.0, 2.0);
        let orbit_vec =
            nalgebra::Rotation2::new(f64::pi() * period) * Vector2::from([params.height, 0.0]);
        let position = o_pos + orbit_vec;

        let mu = G * (parent_physics.mass + params.mass);
        let f_vec = o_pos - position;
        let velocity = (mu / f_vec.norm()).sqrt()
            * (nalgebra::Rotation2::new(-f64::frac_pi_2()) * f_vec.normalize());
        debug!("{} velocity: {}", params.name, velocity);

        // add parent velocity
        let velocity = parent_physics.velocity + velocity;

        Box::new(Body {
            id: params.id,
            parent_id: params.parent_id,
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
        size: f64,
    ) -> ([[f32; 4]; 3], [[[f64; 3]; 2]; 3]) {
        // cheater shadow from the parent.
        let parent = &world.entities.get(self.parent_id).unwrap();
        let vec = self.position - parent.physics_data().position;
        let unit_vec: Vector2<f64> = vec.normalize();
        /*
        let wrot: Rotation2<f64> =
            Rotation2::from_matrix(&world_transform.matrix().fixed_resize(2.0));
        */

        // highlights reduce with the inverse square of the distance
        //let strength = parent.size() / (f64::pi() * 4.0 * vec.norm_squared());
        /*
        let strength_f =
            |a| 2.0 / 3.0 * ((1.0 - a / f64::pi()) * f64::cos(a) + 1.0 / f64::pi() * f64::sin(a));
        let strength = strength_f(f64::pi() / 2.0);
        */
        let strength = 1.0 - (vec.norm().log(10.0) / 6.0e10_f64.log(10.0));
        // debug!("Strength for: {}, {}", self.name(), strength);

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

    fn render_scaled(&self, world: &World, context: &Context, graphics: &mut G2d, scale: f64) {
        let extents = ellipse::circle(0.0, 0.0, 10.0 / scale); // statically sized placeholder
        let transform = context.transform.trans(self.position[0], self.position[1]);

        Rectangle::new([0.0, 0.0, 0.0, 1.0]).draw(
            extents,
            &DrawState::new_clip(),
            transform,
            graphics,
        );

        let (colors, transes) = self.get_offsets(world, transform, 5.0 / scale);
        colors
            .iter()
            .zip(transes.iter())
            .map(|(color, trans)| {
                Rectangle::new(*color).draw(extents, &DrawState::new_inside(), *trans, graphics)
            })
            .for_each(drop);
    }

    fn render_real(&self, world: &World, context: &Context, graphics: &mut G2d) {
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

        let (colors, transes) = self.get_offsets(world, transform, size);
        colors
            .iter()
            .zip(transes.iter())
            .map(|(color, trans)| {
                Ellipse::new(*color).draw(extents, &DrawState::new_inside(), *trans, graphics)
            })
            .for_each(drop);
    }
}

impl Entity for Body {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Renderable for Body {
    fn render(&self, world: &World, context: &Context, graphics: &mut G2d) {
        let scale = context.transform[0][0] * 1e3; // this transform differs from nalgebra by 1e3
        if is_visible(self.size, context.transform[0][0]) {
            self.render_real(world, context, graphics);
        } else if self.size < 10_000_000.0 {
            self.render_scaled(world, context, graphics, scale);
        };
    }
}

impl PhysicsBody for Body {
    fn tick(&self, others: Vec<&Box<dyn Entity>>, dt: f64) -> PhysicsFrame {
        let mut dv: Vector2<f64> = Vector2::from([0.0; 2]);
        let this_physics_data = self.physics_data();
        let pos = this_physics_data.position;
        let id = self.id();

        for e in others {
            if e.id() == id {
                continue;
            }

            let physics = e.physics_data();
            let o_pos = physics.position;
            let mass = physics.mass + this_physics_data.mass;

            let vec = o_pos - pos;
            let r_sq = vec.norm_squared();

            dv += G * mass / r_sq * vec.normalize();
        }

        PhysicsFrame {
            velocity: self.velocity + dv * dt,
            position: self.position + self.velocity * dt,
        }
    }

    fn set(&mut self, f: PhysicsFrame) {
        self.position = f.position;
        self.velocity = f.velocity;
    }

    fn physics_data(&self) -> PhysicsData {
        PhysicsData {
            position: self.position,
            velocity: self.velocity,
            mass: self.mass,
            size: self.size,
        }
    }
}

pub struct Star {
    id: usize,
    name: String,
    velocity: Vector2<f64>,
    position: Point2<f64>,
    color: [f32; 4],
    mass: f64,
    size: f64,
}

impl Star {
    pub fn new_from_params(star: BodyParams) -> Box<dyn Entity> {
        Box::new(Star {
            id: star.id,
            name: star.name,
            position: Point2::from([X_SIZE / 2.0, Y_SIZE / 2.0]),
            velocity: Vector2::from([0.0; 2]),
            color: star.color,
            mass: star.mass,
            size: star.diameter,
        })
    }
}

impl PhysicsBody for Star {
    // Let's pretend the star doesn't move
    fn tick(&self, _others: Vec<&Box<dyn Entity>>, _dt: f64) -> PhysicsFrame {
        PhysicsFrame {
            velocity: Vector2::from([0.0, 0.0]),
            position: self.position,
        }
    }

    fn set(&mut self, _f: PhysicsFrame) {}

    fn physics_data(&self) -> PhysicsData {
        PhysicsData {
            position: self.position,
            velocity: self.velocity,
            mass: self.mass,
            size: self.size,
        }
    }
}

impl Renderable for Star {
    fn render(&self, _world: &World, context: &Context, graphics: &mut G2d) {
        let scale = context.transform[0][0] * 1e3; // this transform differs from nalgebra by 1e3

        let extents = if scale > 100_000.0 / 5_000_000_000.0 {
            ellipse::circle(self.position[0], self.position[1], self.size)
        } else {
            ellipse::circle(self.position[0], self.position[1], 25.0 / scale) // statically sized placeholder
        };

        rectangle(self.color, extents, context.transform, graphics);
    }
}

impl Entity for Star {
    fn id(&self) -> usize {
        self.id
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl World {
    pub fn new_from_json(input: String) -> Result<World, serde_json::error::Error> {
        let world_params: WorldParams = serde_json::from_str(&input)?;

        let stars = world_params
            .stars
            .into_iter()
            .enumerate()
            .map(|(i, mut p)| {
                p.id = i.try_into().unwrap();
                Star::new_from_params(p)
            })
            .collect::<Vec<_>>();
        let star = stars.get(0).unwrap();
        let mut offset = stars.len();
        let planets = world_params
            .planets
            .into_iter()
            .flat_map(|mut p| {
                p.id = offset;
                let parent_id = offset;
                let children = p.children.unwrap_or_else(|| vec![]);
                p.children = None;
                let this = Body::new_stable_orbit(&star.physics_data(), p);
                let parent_motion = this.physics_data();

                children
                    .into_iter()
                    .map(move |mut c| {
                        offset += 1;
                        c.id = offset;
                        c.parent_id = parent_id.clone();
                        Body::new_stable_orbit(&parent_motion, c)
                    })
                    .chain(vec![this].into_iter())
            })
            .collect::<Vec<_>>();

        Ok(World {
            entities: stars
                .into_iter()
                .chain(planets.into_iter())
                .collect::<Vec<_>>(),
        })
    }
}

#[inline]
fn is_visible(absolute_size: f64, zoom: f64) -> bool {
    // at 1.0 / 50_000.0 zoom, 7k is roughly 1 pixel
    debug!(
        "absolute_size: {}, zoom: {}, together: {}",
        absolute_size,
        zoom,
        absolute_size * zoom
    );
    absolute_size * zoom > 0.001
}

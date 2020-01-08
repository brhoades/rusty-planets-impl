use piston_window::*;

pub trait Renderable {
    fn render(&self, ctx: &Context, graphics: &mut G2d);
}

pub trait PhysicsBody {
    fn tick(&self, w: &World) -> PhysicsFrame;
    fn set(&mut self, f: &PhysicsFrame);
}

pub struct PhysicsFrame {
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
    color: [f64; 4],
    size: [f64; 2],
}

/*
impl Planet {
    fn new(pos: [f64; 2], vel: [f64; 2]) -> Self {
        Planet{
            color:
        }
    }
}
*/

impl Renderable for Planet {
    fn render(&self, context: &Context, graphics: &mut G2d) {
        rectangle([1.0, 0.0, 0.0, 1.0],
                  [0.0, 0.0, 100.0, 100.0],
                  context.transform,
                  graphics);
    }
}

impl PhysicsBody for Planet {
    fn tick(&self, w: &World) -> PhysicsFrame {
        PhysicsFrame {
            velocity: self.velocity,
            position: self.position,
        }
    }

    fn set(&mut self, f: &PhysicsFrame) {
        self.position = f.position;
        self.velocity = f.velocity;
    }
}

pub struct Star {
}

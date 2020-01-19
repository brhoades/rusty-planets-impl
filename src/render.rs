use piston_window::*;

pub trait Renderable<T> {
	fn render(&self, world: &T, context: &Context, graphics: &mut G2d);
	fn render_mut(&mut self, world: &T, context: &Context, graphics: &mut G2d);
}

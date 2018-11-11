#[macro_use]
extern crate specs_derive;

use ggez::{
    conf,
    event::{self, MouseButton},
    graphics::{self, Point2, Rect},
    Context, GameResult,
};
use ncollide2d::math::Vector;
use specs::prelude::*;

type Float = f32;
type Vec = Vector<Float>;

#[derive(Default)]
struct Bvh();

#[derive(Component, Debug)]
struct Position {
    vec: Vec,
    dirty: bool,
}
impl Position {
    fn new(vec: Vec) -> Position {
        Position { vec, dirty: false }
    }
}

#[derive(Component, Debug)]
enum Geometry {
    Circle(f32),
    Square(f32),
}

#[derive(Component, Debug)]
struct Velocity(Vec);

struct VelocitySys;
impl<'a> System<'a> for VelocitySys {
    type SystemData = (ReadStorage<'a, Velocity>, WriteStorage<'a, Position>);
    fn run(&mut self, (vel, mut pos): Self::SystemData) {
        for (vel, pos) in (&vel, &mut pos).join() {
            pos.vec.x += vel.0.x * 0.05;
            pos.vec.y += vel.0.y * 0.05;
        }
    }
}

struct RenderSys<'a>(&'a mut Context);
impl<'a> System<'a> for RenderSys<'a> {
    type SystemData = (ReadStorage<'a, Position>, ReadStorage<'a, Geometry>);
    fn run(&mut self, (pos, geometry): Self::SystemData) {
        for (pos, geometry) in (&pos, &geometry).join() {
            match geometry {
                Geometry::Circle(radius) => {
                    graphics::circle(
                        self.0,
                        graphics::DrawMode::Line(1.0),
                        Point2::new(pos.vec.x, pos.vec.y),
                        *radius,
                        0.1,
                    )
                    .unwrap();
                }
                Geometry::Square(length) => {
                    graphics::rectangle(
                        self.0,
                        graphics::DrawMode::Line(1.0),
                        Rect::new(pos.vec.x, pos.vec.y, *length, *length),
                    )
                    .unwrap();
                }
            }
        }
    }
}

struct MainState {
    world: World,
}
impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Velocity>();
        world.register::<Geometry>();
        let state = MainState { world };
        Ok(state)
    }
}
impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let mut dispatcher = DispatcherBuilder::new()
            .with(VelocitySys, "velocity", &[])
            .build();
        dispatcher.dispatch(&mut self.world.res);
        self.world.maintain();
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        let mut render_sys = RenderSys(ctx);
        render_sys.run_now(&self.world.res);
        graphics::present(ctx);
        Ok(())
    }
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        x: i32,
        y: i32,
    ) {
        let velocity = Velocity(Vec::new(
            rand::random::<f32>() * 4.0 - 2.0,
            rand::random::<f32>() * 4.0 - 2.0,
        ));
        let position = Position::new(Vec::new(x as f32, y as f32));
        let random = rand::random::<f32>();
        let geometry = match random {
            _ if random < 0.5 => Geometry::Circle(10.0),
            _ => Geometry::Square(20.0),
        };
        self.world
            .create_entity()
            .with(geometry)
            .with(velocity)
            .with(position)
            .build();
    }
}

fn main() {
    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("specs-bvh", "ggez", c).unwrap();
    let state = &mut MainState::new(ctx).unwrap();
    if let Err(e) = event::run(ctx, state) {
        println!("Error encountered: {}", e);
    } else {
        println!("Game exited cleanly.");
    }
}

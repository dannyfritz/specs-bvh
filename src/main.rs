#[macro_use]
extern crate specs_derive;

use ggez::{
    conf, event,
    graphics::{self, Point2},
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
    type SystemData = ReadStorage<'a, Position>;
    fn run(&mut self, pos: Self::SystemData) {
        for pos in (pos).join() {
            graphics::circle(
                self.0,
                graphics::DrawMode::Line(1.0),
                Point2::new(pos.vec.x, pos.vec.y),
                10.0,
                0.1,
            )
            .unwrap();
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
        world
            .create_entity()
            .with(Velocity(Vec::new(1.0, 1.0)))
            .with(Position::new(Vec::new(1.0, 1.0)))
            .build();
        let s = MainState { world };
        Ok(s)
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

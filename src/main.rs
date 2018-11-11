#[macro_use]
extern crate specs_derive;

use ggez::{
    conf,
    event::{self, MouseButton},
    graphics::{self, Color, Point2, Rect},
    Context, GameResult,
};
use ncollide2d::{
    bounding_volume::{aabb, HasBoundingVolume, AABB},
    math::{Isometry, Vector},
    partitioning::{DBVTLeaf, DBVT, BoundingVolumeInterferencesCollector},
    shape::{Ball, Cuboid},
};
use specs::prelude::*;

type Float = f32;
type Vec = Vector<Float>;

const WHITE: Color = graphics::Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
const RED: Color = graphics::Color {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

struct Bvh(DBVT<Float, (), AABB<Float>>);
impl Bvh {
    fn new() -> Bvh {
        Bvh(DBVT::new())
    }
}

#[derive(Component, Debug)]
struct Collider {
    colliding: bool,
}
impl Collider {
    fn new() -> Collider {
        Collider { colliding: false }
    }
}

#[derive(Component, Debug)]
struct Position {
    vec: Vec,
}
impl Position {
    fn new(vec: Vec) -> Position {
        Position { vec }
    }
}

#[derive(Component, Debug)]
enum Geometry {
    Circle(f32),
    Square(f32),
}

#[derive(Component, Debug)]
struct Velocity {
    vec: Vec,
}
impl Velocity {
    fn new(vec: Vec) -> Velocity {
        Velocity { vec }
    }
}

struct VelocitySys;
impl<'a> System<'a> for VelocitySys {
    type SystemData = (ReadStorage<'a, Velocity>, WriteStorage<'a, Position>);
    fn run(&mut self, (vel, mut pos): Self::SystemData) {
        for (vel, pos) in (&vel, &mut pos).join() {
            pos.vec.x += vel.vec.x * 0.05;
            pos.vec.y += vel.vec.y * 0.05;
        }
    }
}

struct BvhSys;
impl<'a> System<'a> for BvhSys {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Geometry>,
        WriteExpect<'a, Bvh>,
    );
    fn run(&mut self, (pos, geometry, mut bvh): Self::SystemData) {
        *bvh = Bvh::new();
        for (pos, geometry) in (&pos, &geometry).join() {
            let shape: Box<HasBoundingVolume<Float, AABB<Float>>> = match geometry {
                Geometry::Circle(radius) => Box::new(Ball::new(*radius)),
                Geometry::Square(length) => {
                    Box::new(Cuboid::new(Vec::new(length / 2.0, length / 2.0)))
                }
            };
            let iso = Isometry::new(pos.vec, 0.0);
            let bv = aabb(shape.as_ref(), &iso);
            let leaf = DBVTLeaf::new(bv, ());
            bvh.0.insert(leaf);
        }
    }
}

#[derive(Default)]
struct CollideSys;
impl<'a> System<'a> for CollideSys {
    type SystemData = (
        ReadExpect<'a, Bvh>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Geometry>,
        WriteStorage<'a, Collider>,
    );
    fn run(&mut self, (bvh, pos, geometry, mut collider): Self::SystemData) {
        for (pos, geometry, collider) in (&pos, &geometry, &mut collider).join() {
            let shape: Box<HasBoundingVolume<Float, AABB<Float>>> = match geometry {
                Geometry::Circle(radius) => Box::new(Ball::new(*radius)),
                Geometry::Square(length) => {
                    Box::new(Cuboid::new(Vec::new(length / 2.0, length / 2.0)))
                }
            };
            let iso = Isometry::new(pos.vec, 0.0);
            let bv = aabb(shape.as_ref(), &iso);
            let mut collisions = vec![];
            bvh.0.visit(&mut BoundingVolumeInterferencesCollector::new(&bv, &mut collisions));
            if collisions.len() > 1 { collider.colliding = true };
        }
    }
}

struct CleanSys;
impl<'a> System<'a> for CleanSys {
    type SystemData = (WriteStorage<'a, Collider>);
    fn run(&mut self, mut collider: Self::SystemData) {
        for collider in (&mut collider).join() {
            collider.colliding = false;
        }
    }
}

struct RenderSys<'a> {
    ctx: &'a mut Context,
}
impl<'a> System<'a> for RenderSys<'a> {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Geometry>,
        ReadStorage<'a, Collider>,
    );
    fn run(&mut self, (pos, geometry, collider): Self::SystemData) {
        for (pos, geometry, collider) in (&pos, &geometry, &collider).join() {
            let color = if collider.colliding { RED } else { WHITE };
            graphics::set_color(self.ctx, color).unwrap();
            match geometry {
                Geometry::Circle(radius) => {
                    graphics::circle(
                        self.ctx,
                        graphics::DrawMode::Line(1.0),
                        Point2::new(pos.vec.x, pos.vec.y),
                        *radius,
                        0.1,
                    )
                    .unwrap();
                }
                Geometry::Square(length) => {
                    graphics::rectangle(
                        self.ctx,
                        graphics::DrawMode::Line(1.0),
                        Rect::new(pos.vec.x - length / 2.0, pos.vec.y - length / 2.0, *length, *length),
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
        world.register::<Collider>();
        world.add_resource(Bvh::new());
        world
            .create_entity()
            .with(Position::new(Vec::new(100.0, 100.0)))
            .with(Geometry::Circle(20.0))
            .with(Collider::new())
            .build();
        let state = MainState { world };
        Ok(state)
    }
}
impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let mut dispatcher = DispatcherBuilder::new()
            .with(CleanSys, "clean", &[])
            .with(VelocitySys, "velocity", &["clean"])
            .with(BvhSys, "bvh", &["velocity"])
            .with(CollideSys, "collide", &["bvh"])
            .build();
        dispatcher.dispatch(&mut self.world.res);
        self.world.maintain();
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        let mut render_sys = RenderSys { ctx };
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
        let speed = 16.0;
        let velocity = Velocity::new(Vec::new(
            rand::random::<f32>() * speed - speed / 2.0,
            rand::random::<f32>() * speed - speed / 2.0,
        ));
        let position = Position::new(Vec::new(x as f32, y as f32));
        let random = rand::random::<f32>();
        let geometry = match random {
            _ if random < 0.5 => Geometry::Circle(rand::random::<f32>() * 20.0 + 10.0),
            _ => Geometry::Square(rand::random::<f32>() * 40.0 + 20.0),
        };
        self.world
            .create_entity()
            .with(geometry)
            .with(velocity)
            .with(position)
            .with(Collider::new())
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

extern crate specs;

use specs::{Component, VecStorage, ReadStorage, RunNow, System, World};

#[derive(Debug)]
struct Position {
    x: f32,
    y: f32
}

impl Component for Position {
    type Storage = VecStorage<Position>;
}

#[derive(Debug)]
struct Velocity {
    x: f32,
    y: f32
}

impl Component for Velocity {
    type Storage = VecStorage<Velocity>;
}

struct LoggingSystem;

impl<'a> System<'a> for LoggingSystem {
    type SystemData = ReadStorage<'a, Position>;

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        for position in data.join() {
            println!("Hello, {:?}", &position);
        }
    }
}

fn main() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();

    let ball = world.create_entity()
        .with(Position {x: 4.0, y: 7.0})
        .with(Velocity {x: 0.5, y: 0.5})
        .build();

    let mut logging_system = LoggingSystem;
    logging_system.run_now(&world.res);
}

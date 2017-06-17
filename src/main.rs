extern crate specs;

use specs::{Component, DispatcherBuilder, VecStorage, ReadStorage, WriteStorage, System, World};
use std::{thread, time};

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

struct UpdatePositionSystem;

impl<'a> System<'a> for UpdatePositionSystem {
    type SystemData = (ReadStorage<'a, Velocity>, WriteStorage<'a, Position>);

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        let (vel, mut pos) = data;

        for (vel, pos) in (&vel, &mut pos).join() {
            pos.x += vel.x * 0.05;
            pos.y += vel.y * 0.05;
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
    let mut update_position_system = UpdatePositionSystem;

    let mut dispatcher = DispatcherBuilder::new()
        .add(logging_system, "logging_system", &[])
        .add(update_position_system, "update_position_system", &["logging_system"])
        .build();

    let sleep_duration = time::Duration::from_millis(1000);
    loop {
        dispatcher.dispatch(&mut world.res);
        thread::sleep(sleep_duration);
    }
}

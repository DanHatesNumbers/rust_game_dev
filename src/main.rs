extern crate specs;
extern crate time;

use specs::{Component, DispatcherBuilder, Fetch, VecStorage, ReadStorage, WriteStorage, System, World};
use std::{thread};


#[derive(Debug)]
struct Position {
    x: f64,
    y: f64
}

impl Component for Position {
    type Storage = VecStorage<Position>;
}

#[derive(Debug)]
struct Velocity {
    x: f64,
    y: f64
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
    type SystemData = (Fetch<'a, DeltaTime>,
        ReadStorage<'a, Velocity>,
        WriteStorage<'a, Position>);

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        let (delta, vel, mut pos) = data;

        let delta = delta.0.num_milliseconds() as f64;

        for (vel, pos) in (&vel, &mut pos).join() {
            pos.x += vel.x * delta;
            pos.y += vel.y * delta;
        }
    }
}

struct DeltaTime(time::Duration);

fn main() {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Velocity>();
    world.add_resource(DeltaTime(time::Duration::from_std(std::time::Duration::new(0, 0)).unwrap()));

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

    let sleep_duration = std::time::Duration::from_millis(1000);
    let mut last_update = std::time::SystemTime::now();
    loop {
        {
            let mut delta = world.write_resource::<DeltaTime>();
            let last_update_elapsed = last_update.elapsed().unwrap();
            println!("{:?}", last_update_elapsed);
            *delta = DeltaTime(time::Duration::from_std(last_update_elapsed).unwrap());
        }

        dispatcher.dispatch(&mut world.res);
        last_update = std::time::SystemTime::now();
        thread::sleep(sleep_duration);
    }
}

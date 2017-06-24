extern crate specs;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use specs::{Component, DispatcherBuilder, Fetch, FetchMut, VecStorage, ReadStorage, WriteStorage, System, World};
use std::path;
use std::f64::consts::PI as PI;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture};

struct Player;

impl Component for Player {
    type Storage = VecStorage<Player>;
}

#[derive(Debug)]
struct Rotation {
    radians: f64
}

impl Component for Rotation {
    type Storage = VecStorage<Rotation>;
}

#[derive(Debug)]
struct RotationalVelocity {
    radians: f64
}

impl Component for RotationalVelocity {
    type Storage = VecStorage<RotationalVelocity>;
}

struct Sprite {
    image: graphics::Image,
    texture: Texture
}

impl Component for Sprite {
    type Storage = VecStorage<Sprite>;
}

#[derive(Clone)]
enum PressedStatus {
    Pressed,
    Released
}

#[derive(Clone)]
struct ButtonStatus {
    button: Button,
    status: PressedStatus
}

impl Component for ButtonStatus {
    type Storage = VecStorage<ButtonStatus>;
}

struct PendingUpdates(Vec<ButtonStatus>);

struct InputHandlingSystem;

impl<'a> System<'a> for InputHandlingSystem {
    type SystemData = (FetchMut<'a, PendingUpdates>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, RotationalVelocity>);

    fn run(&mut self, data: Self::SystemData) {
        use piston::input::keyboard;
        use specs::Join;

        let (mut pending_updates, player, mut rotational_velocity) = data;
        let (player, mut rotational_velocity) = (&player, &mut rotational_velocity).join().next().unwrap();


        for pending_update in &pending_updates.0 {
            match pending_update.button {
                Button::Keyboard(key) => {
                    match key {
                        Key::Left => {
                            rotational_velocity.radians = 0.0 - rotational_velocity.radians;
                        },
                        Key::Right => {
                            rotational_velocity.radians = 0.0 + rotational_velocity.radians;
                        }
                        _ => ()
                    }
                },
                _ => ()
            }
        }

        pending_updates.0.clear();
    }
}

struct UpdateRotationSystem;

impl<'a> System<'a> for UpdateRotationSystem {
    type SystemData = (Fetch<'a, UpdateArgs>,
        ReadStorage<'a, RotationalVelocity>,
        WriteStorage<'a, Rotation>);

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;

        let (update_args, rotational_velocity, mut rotation) = data;

        let delta = update_args.dt;

        for (rotational_velocity, rotation) in (&rotational_velocity, &mut rotation).join() {
            rotation.radians += rotational_velocity.radians * delta;
            if(rotation.radians > 2.0 * PI) {
                rotation.radians = 0.0;
            }
            else if(rotation.radians < -2.0 * PI) {
                rotation.radians = 0.0
            }
        }
    }
}

struct RenderSystem;

impl<'a> System<'a> for RenderSystem {
    type SystemData = (FetchMut<'a, GlGraphics>,
        Fetch<'a, RenderArgs>,
        ReadStorage<'a, Rotation>,
        ReadStorage<'a, Sprite>);

    fn run(&mut self, data: Self::SystemData) {
        use specs::Join;
        use graphics::*;

        let (mut gl, render_args, rotation, sprite) = data;

        let white = [1.0, 1.0, 1.0, 1.0];
        gl.draw(render_args.viewport(), |context, gl| {
            graphics::clear(white, gl);

            for (rotation, sprite) in (&rotation, &sprite).join() {
                let transform = context.transform
                    .trans((render_args.width / 2) as f64, (render_args.height / 2) as f64)
                    .rot_rad(rotation.radians)
                    .trans(-(sprite.image.rectangle.unwrap()[2] / 2.0), -(sprite.image.rectangle.unwrap()[3] / 2.0));

                sprite.image.draw(&sprite.texture, &draw_state::DrawState::default(), transform, gl);
            }
        });
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new(
        "spinning_tux",
        [800, 600]
    )
    .opengl(opengl)
    .exit_on_esc(true)
    .build()
    .unwrap();

    let gl = GlGraphics::new(opengl);

    let mut world = World::new();
    world.register::<Rotation>();
    world.register::<RotationalVelocity>();
    world.register::<Sprite>();
    world.register::<Player>();
    world.add_resource(gl);
    world.add_resource(PendingUpdates(Vec::new()));

    let texture = Texture::from_path(path::Path::new("Tux.png")).unwrap();

    let tux = world.create_entity()
        .with(Rotation {
            radians: 0.0
        })
        .with(RotationalVelocity {
            radians: 2.0
        })
        .with(Sprite {
            image: graphics::Image::new().rect(graphics::rectangle::square(0.0, 0.0, 145.0)),
            texture: texture
        })
        .with(Player)
        .build();

    let input_handling_system = InputHandlingSystem;
    let update_rotation_system = UpdateRotationSystem;
    let render_system = RenderSystem;

    let mut update_dispatcher = DispatcherBuilder::new()
        .add(input_handling_system, "input_handling_system", &[])
        .add(update_rotation_system, "update_rotation_system", &["input_handling_system"])
        .build();

    let mut render_dispatcher = DispatcherBuilder::new()
        .add_thread_local(render_system)
        .build();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(p) = e.press_args() {
            let mut pending_updates = world.write_resource::<PendingUpdates>();
            pending_updates.0.push(ButtonStatus {button: p, status: PressedStatus::Pressed});
        }
        if let Some(r) = e.release_args() {
            let mut pending_updates = world.write_resource::<PendingUpdates>();
            pending_updates.0.push(ButtonStatus {button: r, status: PressedStatus::Released});
        }
        if let Some(u) = e.update_args() {
            world.add_resource(u);
            update_dispatcher.dispatch(&mut world.res);
        }
        if let Some(r) = e.render_args() {
            world.add_resource(r);
            render_dispatcher.dispatch(&mut world.res);
        }
    }
}

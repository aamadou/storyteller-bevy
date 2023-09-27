use bevy::prelude::*;

mod people;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, people::Plugin))
        .add_systems(Startup, hello_world)
        .run();
}

fn hello_world() {
    println!("Hello, world!");
}

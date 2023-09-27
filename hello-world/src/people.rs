use bevy::{
    app::{Startup, Update},
    prelude::{App, Commands, Component, Query, With},
};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        #[rustfmt::skip]
        app.add_systems(Startup, startup)
            .add_systems(Update, greet);
    }
}

#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

impl Name {
    fn new<S: Into<String>>(name: S) -> Self {
        Self(name.into())
    }
}

fn startup(mut cmds: Commands) {
    cmds.spawn((Person, Name::new("Elaina Proctor")));
    cmds.spawn((Person, Name::new("Renzo Hume")));
    cmds.spawn((Person, Name::new("Zayna Nieves")));
}

fn greet(q: Query<&Name, With<Person>>) {
    for name in &q {
        println!("Hello, {}!", name.0);
    }
}

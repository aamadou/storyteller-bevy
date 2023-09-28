use std::time::Duration;

use bevy::{
    prelude::{
        info, App, Commands, Component, Query, Res, ResMut, Resource, Startup, Update, With,
    },
    time::{Time, TimerMode},
};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Timer(bevy::time::Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, startup)
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

#[derive(Resource)]
struct Timer(bevy::time::Timer);

impl Timer {
    fn tick(&mut self, delta: Duration) -> &bevy::time::Timer {
        self.0.tick(delta)
    }
}

fn startup(mut cmds: Commands) {
    cmds.spawn((Person, Name::new("Elaina Proctor")));
    cmds.spawn((Person, Name::new("Renzo Hume")));
    cmds.spawn((Person, Name::new("Zayna Nieves")));
}

fn greet(time: Res<Time>, mut timer: ResMut<Timer>, q: Query<&Name, With<Person>>) {
    if timer.tick(time.delta()).just_finished() {
        for name in &q {
            info!("Hello, {}!", name.0);
        }
    }
}

use bevy::prelude::*;

fn main() {
    App::new()
        .add_systems(Startup, add_people)
        .add_systems(Update, (hello_world, greet_people))
        .run();
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

fn add_people(mut cmds: Commands) {
    cmds.spawn((Person, Name::new("Elaina Proctor")));
    cmds.spawn((Person, Name::new("Renzo Hume")));
    cmds.spawn((Person, Name::new("Zayna Nieves")));
}

fn hello_world() {
    println!("Hello, world!");
}

fn greet_people(q: Query<&Name, With<Person>>) {
    for name in &q {
        println!("Hello, {}!", name.0);
    }
}

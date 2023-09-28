use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

pub struct UiLayerPlugin;

impl Plugin for UiLayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, layout)
            .add_systems(Update, update);
    }
}

#[derive(Component)]
pub struct FramerateCounter;

fn layout(mut cmd: Commands) {
    cmd.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            padding: UiRect::all(Val::Px(16.)),
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        parent.spawn((
            FramerateCounter,
            TextBundle::from_section(
                "0 fps",
                TextStyle {
                    color: Color::GREEN,
                    font_size: 14.,
                    ..default()
                },
            )
            .with_text_alignment(TextAlignment::Right)
            .with_style(Style {
                // TODO: This doesn't seem to have any effect
                width: Val::Percent(100.),
                ..default()
            }),
        ));
    });
}

fn update(
    time: Res<Time>,
    diag: Res<DiagnosticsStore>,
    mut counter: Query<&mut Text, With<FramerateCounter>>,
) {
    if let Some(fps_diag) = diag.get(FrameTimeDiagnosticsPlugin::FPS) {
        let fps = if let Some(fps_smoothed) = fps_diag.smoothed() {
            fps_smoothed
        } else {
            let ms = time.delta().as_micros() as f64 / 1000.;
            1000. / ms
        };

        counter.single_mut().sections[0].value = format!("{fps:.2} fps");
    }
}

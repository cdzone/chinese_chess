use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "中国象棋".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .run();
}

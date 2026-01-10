//! 中国象棋客户端入口

use bevy::prelude::*;
use bevy::window::WindowResolution;
use chess_client::ChessClientPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "中国象棋".into(),
                        resolution: WindowResolution::new(1280, 720),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(ChessClientPlugin)
        .add_systems(Startup, setup_camera)
        .add_systems(Update, handle_window_resize)
        .run();
}

/// 设置摄像机
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// 处理窗口大小变化
fn handle_window_resize(
    windows: Query<&Window>,
    mut layout: ResMut<chess_client::board::BoardLayout>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let new_layout = chess_client::board::BoardLayout::from_window_size(
        window.width(),
        window.height(),
    );

    // 只在布局变化时更新
    if (new_layout.cell_size - layout.cell_size).abs() > 1.0 {
        *layout = new_layout;
    }
}

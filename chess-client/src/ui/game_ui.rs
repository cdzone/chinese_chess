//! 游戏界面 UI

use bevy::prelude::*;

use super::{ButtonAction, GameUiMarker, UiMarker, button_style, NORMAL_BUTTON, HOVERED_BUTTON, PRESSED_BUTTON};
use crate::game::{ClientGame, GameEvent};

/// 计时器显示组件
#[derive(Component)]
pub struct TimerDisplay {
    pub is_red: bool,
}

/// 棋谱显示组件
#[derive(Component)]
pub struct MoveHistoryDisplay;

/// 暂停按钮文字标记
#[derive(Component)]
pub struct PauseButtonText;

/// 设置游戏 UI
pub fn setup_game_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 右侧面板
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                bottom: Val::Px(20.0),
                width: Val::Px(280.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            UiMarker,
            GameUiMarker,
        ))
        .with_children(|parent| {
            // 对手信息区
            spawn_player_info(parent, &asset_server, "对手", false);

            // 棋谱区域
            spawn_move_history(parent, &asset_server);

            // 玩家信息区
            spawn_player_info(parent, &asset_server, "玩家", true);

            // 按钮区
            spawn_game_buttons(parent, &asset_server);
        });
}

/// 生成玩家信息区
fn spawn_player_info(parent: &mut ChildBuilder, asset_server: &AssetServer, name: &str, is_red: bool) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(10.0)),
            margin: UiRect::bottom(Val::Px(10.0)),
            ..default()
        })
        .with_children(|parent| {
            // 玩家名称
            parent.spawn((
                Text::new(name),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(if is_red {
                    Color::srgb(0.9, 0.3, 0.3)
                } else {
                    Color::srgb(0.3, 0.3, 0.3)
                }),
            ));

            // 计时器
            parent.spawn((
                Text::new("10:00"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TimerDisplay { is_red },
            ));
        });
}

/// 生成棋谱区域
fn spawn_move_history(parent: &mut ChildBuilder, asset_server: &AssetServer) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::vertical(Val::Px(10.0)),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("棋谱记录"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // 走法列表容器
            parent.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                MoveHistoryDisplay,
            ));
        });
}

/// 生成游戏按钮
fn spawn_game_buttons(parent: &mut ChildBuilder, asset_server: &AssetServer) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            // 悔棋按钮
            spawn_game_button(parent, asset_server, "悔棋", ButtonAction::Undo);
            // 认输按钮
            spawn_game_button(parent, asset_server, "认输", ButtonAction::Resign);
            // 暂停按钮（带标记以便动态更新文字）
            spawn_pause_button(parent, asset_server);
            // 保存按钮
            spawn_game_button(parent, asset_server, "保存", ButtonAction::SaveGame);
        });
}

/// 生成游戏按钮
fn spawn_game_button(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    text: &str,
    action: ButtonAction,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
            action,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 生成暂停按钮（带标记以便动态更新文字）
fn spawn_pause_button(parent: &mut ChildBuilder, asset_server: &AssetServer) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
            ButtonAction::Pause,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("暂停"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                PauseButtonText,
            ));
        });
}

/// 清理游戏 UI
pub fn cleanup_game_ui(mut commands: Commands, query: Query<Entity, With<GameUiMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// 更新计时器显示
pub fn update_timer_display(
    game: Res<ClientGame>,
    mut query: Query<(&mut Text, &TimerDisplay)>,
) {
    if !game.is_changed() {
        return;
    }

    for (mut text, timer) in query.iter_mut() {
        let time_ms = if timer.is_red {
            game.red_time_ms
        } else {
            game.black_time_ms
        };

        let minutes = time_ms / 60000;
        let seconds = (time_ms % 60000) / 1000;
        **text = format!("{:02}:{:02}", minutes, seconds);
    }
}

/// 更新暂停按钮文字
pub fn update_pause_button_text(
    game: Res<ClientGame>,
    mut query: Query<&mut Text, With<PauseButtonText>>,
) {
    if !game.is_changed() {
        return;
    }

    for mut text in query.iter_mut() {
        **text = if game.is_paused {
            "继续".to_string()
        } else {
            "暂停".to_string()
        };
    }
}

/// 更新棋谱显示
pub fn update_move_history(
    game: Res<ClientGame>,
    mut commands: Commands,
    query: Query<Entity, With<MoveHistoryDisplay>>,
    asset_server: Res<AssetServer>,
) {
    if !game.is_changed() {
        return;
    }

    for entity in query.iter() {
        // 清除旧内容
        commands.entity(entity).despawn_descendants();

        // 添加新内容
        commands.entity(entity).with_children(|parent| {
            for (i, record) in game.move_history.iter().enumerate() {
                let move_num = i / 2 + 1;
                let is_red = i % 2 == 0;

                if is_red {
                    // 红方走法（带回合数）
                    parent.spawn((
                        Text::new(format!("{}. {}", move_num, record.notation)),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.7, 0.7)),
                    ));
                } else {
                    // 黑方走法
                    parent.spawn((
                        Text::new(format!("   {}", record.notation)),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                }
            }
        });
    }
}

/// 处理游戏按钮点击
pub fn handle_game_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_events: EventWriter<GameEvent>,
    game: Res<ClientGame>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                match action {
                    ButtonAction::Undo => {
                        game_events.send(GameEvent::RequestUndo);
                    }
                    ButtonAction::Resign => {
                        game_events.send(GameEvent::Resign);
                    }
                    ButtonAction::Pause => {
                        if game.is_paused {
                            game_events.send(GameEvent::ResumeGame);
                        } else {
                            game_events.send(GameEvent::PauseGame);
                        }
                    }
                    ButtonAction::SaveGame => {
                        // TODO: 保存游戏
                        tracing::info!("Save game clicked");
                    }
                    _ => {}
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

/// 设置游戏结束 UI
pub fn setup_game_over_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            UiMarker,
            GameUiMarker,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|parent| {
                    // 结果文字
                    parent.spawn((
                        Text::new("游戏结束"),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(30.0)),
                            ..default()
                        },
                    ));

                    // 返回主菜单按钮
                    parent
                        .spawn((
                            Button,
                            button_style(),
                            BackgroundColor(NORMAL_BUTTON),
                            ButtonAction::BackToMenu,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("返回主菜单"),
                                TextFont {
                                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // 再来一局按钮
                    parent
                        .spawn((
                            Button,
                            button_style(),
                            BackgroundColor(NORMAL_BUTTON),
                            ButtonAction::PlayAgain,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("再来一局"),
                                TextFont {
                                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                                    font_size: 24.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

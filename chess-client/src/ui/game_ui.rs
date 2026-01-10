//! 游戏界面 UI

use bevy::prelude::*;

use super::{ButtonAction, GameUiMarker, UiMarker, NORMAL_BUTTON, HOVERED_BUTTON, PRESSED_BUTTON};
use crate::game::{AiThinkingState, ClientGame, GameEvent};
use crate::network::NetworkEvent;
use crate::GameState;

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

/// AI 思考指示器标记
#[derive(Component)]
pub struct AiThinkingIndicator;

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

    // AI 思考指示器（棋盘中央）
    // P2 修复：使用 flexbox 居中而不是 left/top 百分比
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
            Visibility::Hidden,
            UiMarker,
            GameUiMarker,
            AiThinkingIndicator,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                    BorderRadius::all(Val::Px(8.0)),
                ))
                .with_children(|inner| {
                    inner.spawn((
                        Text::new("AI 思考中..."),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                    ));
                });
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
                flex_shrink: 1.0,  // 允许收缩
                min_height: Val::Px(50.0),  // 最小高度
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
            flex_shrink: 0.0,  // 防止按钮区域被压缩
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

        // 无限时间模式（本地 PvE）显示 "--:--"
        if time_ms > 24 * 60 * 60 * 1000 {
            // 超过 24 小时视为无限
            **text = "--:--".to_string();
        } else {
            let minutes = time_ms / 60000;
            let seconds = (time_ms % 60000) / 1000;
            **text = format!("{:02}:{:02}", minutes, seconds);
        }
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

/// 更新 AI 思考指示器
pub fn update_ai_thinking_indicator(
    thinking_state: Res<AiThinkingState>,
    mut query: Query<&mut Visibility, With<AiThinkingIndicator>>,
) {
    for mut visibility in &mut query {
        *visibility = if thinking_state.is_thinking {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// 设置游戏结束 UI
pub fn setup_game_over_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game: Res<ClientGame>,
) {
    // 根据游戏结果确定显示内容
    let (title, title_color, subtitle) = match game.is_player_win() {
        Some(true) => ("恭喜获胜！", Color::srgb(1.0, 0.84, 0.0), get_win_reason(&game)),
        Some(false) => ("很遗憾，您输了", Color::srgb(0.8, 0.3, 0.3), get_win_reason(&game)),
        None => {
            if game.game_result.is_some() {
                ("握手言和", Color::srgb(0.7, 0.7, 0.7), get_draw_reason(&game))
            } else {
                ("游戏结束", Color::WHITE, "".to_string())
            }
        }
    };

    // 统计信息
    let stats = format!(
        "总回合数：{}    总步数：{}",
        game.total_rounds(),
        game.total_moves()
    );

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            UiMarker,
            GameOverUiMarker,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(50.0)),
                        min_width: Val::Px(400.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderRadius::all(Val::Px(12.0)),
                ))
                .with_children(|parent| {
                    // 主标题
                    parent.spawn((
                        Text::new(title),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(title_color),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // 副标题（胜负原因）
                    if !subtitle.is_empty() {
                        parent.spawn((
                            Text::new(subtitle),
                            TextFont {
                                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                            Node {
                                margin: UiRect::bottom(Val::Px(25.0)),
                                ..default()
                            },
                        ));
                    }

                    // 分隔线
                    parent.spawn((
                        Node {
                            width: Val::Percent(80.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    ));

                    // 统计信息
                    parent.spawn((
                        Text::new(stats),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        Node {
                            margin: UiRect::vertical(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    // 按钮容器
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Center,
                            column_gap: Val::Px(20.0),
                            margin: UiRect::top(Val::Px(10.0)),
                            ..default()
                        })
                        .with_children(|parent| {
                            // 返回主菜单按钮
                            spawn_result_button(
                                parent,
                                &asset_server,
                                "返回主菜单",
                                ButtonAction::BackToMenu,
                                Color::srgb(0.3, 0.3, 0.3),
                            );

                            // 再来一局按钮
                            spawn_result_button(
                                parent,
                                &asset_server,
                                "再来一局",
                                ButtonAction::PlayAgain,
                                Color::srgb(0.2, 0.5, 0.3),
                            );
                        });
                });
        });
}

/// 生成结果页按钮
fn spawn_result_button(
    parent: &mut ChildBuilder,
    asset_server: &AssetServer,
    text: &str,
    action: ButtonAction,
    bg_color: Color,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(160.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            BorderRadius::all(Val::Px(8.0)),
            action,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 获取胜负原因描述
fn get_win_reason(game: &ClientGame) -> String {
    use protocol::{GameResult, WinReason};
    
    match &game.game_result {
        Some(GameResult::RedWin(reason)) | Some(GameResult::BlackWin(reason)) => {
            match reason {
                WinReason::Checkmate => "将死对方".to_string(),
                WinReason::Resign => "对方认输".to_string(),
                WinReason::Timeout => "对方超时".to_string(),
                WinReason::Disconnect => "对方断线".to_string(),
            }
        }
        _ => "".to_string(),
    }
}

/// 获取和棋原因描述
fn get_draw_reason(game: &ClientGame) -> String {
    use protocol::{DrawReason, GameResult};
    
    match &game.game_result {
        Some(GameResult::Draw(reason)) => {
            match reason {
                DrawReason::Agreement => "双方同意和棋".to_string(),
                DrawReason::Stalemate => "无子可动".to_string(),
                DrawReason::Repetition => "三次重复局面".to_string(),
                DrawReason::FiftyMoves => "五十回合无吃子".to_string(),
            }
        }
        _ => "".to_string(),
    }
}

/// 游戏结束 UI 标记
#[derive(Component)]
pub struct GameOverUiMarker;

/// 清理游戏结束 UI
pub fn cleanup_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUiMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// 处理游戏结束界面的按钮点击
pub fn handle_game_over_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut game: ResMut<ClientGame>,
    mut network_events: EventWriter<NetworkEvent>,
    conn_handle: Res<crate::network::NetworkConnectionHandle>,
    settings: Res<crate::settings::GameSettings>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                match action {
                    ButtonAction::BackToMenu => {
                        tracing::info!("Back to menu clicked");
                        // 断开网络连接
                        conn_handle.connection.disconnect();
                        // 重置游戏状态
                        game.reset();
                        // 返回主菜单
                        game_state.set(GameState::Menu);
                    }
                    ButtonAction::PlayAgain => {
                        tracing::info!("Play again clicked");
                        // 保存之前的游戏模式
                        let game_mode = game.game_mode.clone();
                        // 断开旧连接（如果是在线模式）
                        if game_mode.as_ref().map_or(false, |m| m.is_online()) {
                            conn_handle.connection.disconnect();
                        }
                        // 重置游戏状态
                        game.reset();
                        
                        // 根据之前的游戏模式重新开始
                        match game_mode {
                            Some(crate::game::GameMode::LocalPvE { difficulty }) => {
                                // 本地 PvE：直接重新开始，使用设置中的时间限制
                                game.start_local_pve(difficulty);
                                let time_ms = settings.time_limit.to_millis();
                                game.red_time_ms = time_ms;
                                game.black_time_ms = time_ms;
                                game_state.set(GameState::Playing);
                            }
                            Some(crate::game::GameMode::OnlinePvE { difficulty, .. }) => {
                                // 在线 PvE：重新连接服务器
                                let initial_state = protocol::BoardState::initial();
                                game.start_game(
                                    initial_state,
                                    protocol::Side::Red,
                                    crate::game::GameMode::OnlinePvE { room_id: 0, difficulty },
                                );
                                game.red_time_ms = 600_000;
                                game.black_time_ms = 600_000;
                                game_state.set(GameState::Playing);
                                
                                // 使用设置中的服务器地址和昵称
                                network_events.send(NetworkEvent::Connect {
                                    addr: settings.server_address.clone(),
                                    nickname: settings.nickname.clone(),
                                });
                            }
                            _ => {
                                // PvP 模式返回主菜单
                                game_state.set(GameState::Menu);
                            }
                        }
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

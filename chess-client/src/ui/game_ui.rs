//! 游戏界面 UI

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use super::{ButtonAction, GameUiMarker, UiMarker, NORMAL_BUTTON, HOVERED_BUTTON, PRESSED_BUTTON};
use crate::game::{AiThinkingState, ClientGame, GameEvent};
#[cfg(feature = "llm")]
use crate::icons;
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

/// 棋谱滚动容器标记
#[derive(Component)]
pub struct MoveHistoryScrollContainer;

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
fn spawn_player_info(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer, name: &str, is_red: bool) {
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
fn spawn_move_history(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::vertical(Val::Px(10.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            ScrollPosition::default(),
            MoveHistoryScrollContainer,
            Interaction::default(),  // 需要 Interaction 来检测鼠标悬停
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
fn spawn_game_buttons(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
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
    parent: &mut ChildSpawnerCommands,
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
fn spawn_pause_button(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
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
        commands.entity(entity).despawn();
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
    query: Query<(Entity, Option<&Children>), With<MoveHistoryDisplay>>,
    asset_server: Res<AssetServer>,
) {
    if !game.is_changed() {
        return;
    }

    for (entity, children) in query.iter() {
        // 清除旧内容
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

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
    mut game_events: MessageWriter<GameEvent>,
    game: Res<ClientGame>,
    settings: Res<crate::settings::GameSettings>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                match action {
                    ButtonAction::Undo => {
                        game_events.write(GameEvent::RequestUndo);
                    }
                    ButtonAction::Resign => {
                        game_events.write(GameEvent::Resign);
                    }
                    ButtonAction::Pause => {
                        if game.is_paused {
                            game_events.write(GameEvent::ResumeGame);
                        } else {
                            game_events.write(GameEvent::PauseGame);
                        }
                    }
                    ButtonAction::SaveGame => {
                        save_current_game(&game, &settings);
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

/// 导出棋谱记录（LLM 友好格式）
fn export_game_record(game: &ClientGame, settings: &crate::settings::GameSettings) {
    use protocol::{GameRecord, MoveRecord as ProtoMoveRecord};
    use std::fs;

    // 确定玩家名称
    let red_player = settings.nickname.clone();
    let black_player = match &game.game_mode {
        Some(crate::game::GameMode::LocalPvE { difficulty }) => {
            format!("AI-{:?}", difficulty)
        }
        Some(crate::game::GameMode::OnlinePvE { difficulty, .. }) => {
            format!("AI-{:?}", difficulty)
        }
        _ => "对手".to_string(),
    };

    // 创建棋谱记录
    let mut record = GameRecord::new(red_player.clone(), black_player.clone());

    // 设置 AI 难度
    if let Some(difficulty) = game.game_mode.as_ref().and_then(|m| m.difficulty()) {
        record.set_ai_difficulty(&format!("{:?}", difficulty));
    }

    // 添加走法历史
    for mv in &game.move_history {
        record.add_move(ProtoMoveRecord::new(mv.from, mv.to, mv.notation.clone()));
    }

    // 设置游戏结果
    if let Some(ref result) = game.game_result {
        record.set_result(result.clone());
    }

    // 获取导出目录
    let export_dir = dirs::document_dir()
        .or_else(|| dirs::home_dir())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let export_dir = export_dir.join("chinese-chess-exports");

    // 确保目录存在
    if let Err(e) = fs::create_dir_all(&export_dir) {
        tracing::error!("创建导出目录失败: {}", e);
        return;
    }

    // 生成文件名
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

    // 导出 JSON 格式
    let json_filename = format!("{}_{}_vs_{}.json", timestamp, red_player, black_player);
    let json_path = export_dir.join(&json_filename);
    match record.to_json() {
        Ok(json) => {
            if let Err(e) = fs::write(&json_path, &json) {
                tracing::error!("写入 JSON 文件失败: {}", e);
            } else {
                tracing::info!("棋谱已导出: {:?}", json_path);
            }
        }
        Err(e) => {
            tracing::error!("序列化 JSON 失败: {}", e);
        }
    }

    // 导出 LLM 友好的文本格式
    let txt_filename = format!("{}_{}_vs_{}_llm.txt", timestamp, red_player, black_player);
    let txt_path = export_dir.join(&txt_filename);
    let llm_format = record.to_llm_format();
    if let Err(e) = fs::write(&txt_path, &llm_format) {
        tracing::error!("写入 LLM 文本文件失败: {}", e);
    } else {
        tracing::info!("LLM 格式棋谱已导出: {:?}", txt_path);
    }

    tracing::info!("棋谱导出完成，保存在: {:?}", export_dir);
}

/// 保存当前游戏
fn save_current_game(game: &ClientGame, settings: &crate::settings::GameSettings) {
    use crate::storage::StorageManager;
    use protocol::{GameRecord, MoveRecord as ProtoMoveRecord};

    // 获取当前游戏状态
    let Some(ref board_state) = game.game_state else {
        tracing::warn!("无法保存：游戏状态为空");
        return;
    };

    // 确定玩家名称
    let red_player = settings.nickname.clone();
    let black_player = match &game.game_mode {
        Some(crate::game::GameMode::LocalPvE { difficulty }) => {
            format!("AI-{:?}", difficulty)
        }
        Some(crate::game::GameMode::OnlinePvE { difficulty, .. }) => {
            format!("AI-{:?}", difficulty)
        }
        _ => "对手".to_string(),
    };

    // 创建棋谱记录
    let mut record = GameRecord::new(red_player.clone(), black_player.clone());

    // 设置 AI 难度
    if let Some(difficulty) = game.game_mode.as_ref().and_then(|m| m.difficulty()) {
        record.set_ai_difficulty(&format!("{:?}", difficulty));
    }

    // 添加走法历史
    for mv in &game.move_history {
        record.add_move(ProtoMoveRecord::new(mv.from, mv.to, mv.notation.clone()));
    }

    // 设置游戏结果
    if let Some(ref result) = game.game_result {
        record.set_result(result.clone());
    }

    // 保存棋局
    match StorageManager::new() {
        Ok(storage) => {
            match storage.save_game(
                &red_player,
                &black_player,
                &mut record,
                board_state,
                game.red_time_ms,
                game.black_time_ms,
            ) {
                Ok(filename) => {
                    tracing::info!("棋局已保存: {}", filename);
                }
                Err(e) => {
                    tracing::error!("保存棋局失败: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("初始化存储失败: {}", e);
        }
    }
}

/// 处理棋谱区域的鼠标滚轮滚动
pub fn handle_move_history_scroll(
    mut scroll_events: MessageReader<MouseWheel>,
    mut query: Query<(&Interaction, &mut ScrollPosition, &ComputedNode), With<MoveHistoryScrollContainer>>,
) {
    for event in scroll_events.read() {
        for (interaction, mut scroll_pos, computed) in &mut query {
            // 只在鼠标悬停时响应滚轮
            if *interaction != Interaction::Hovered && *interaction != Interaction::Pressed {
                continue;
            }
            
            let scroll_amount = match event.unit {
                bevy::input::mouse::MouseScrollUnit::Line => event.y * 20.0,
                bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
            };
            
            // 更新滚动位置（向上滚动减少 y，向下滚动增加 y）
            let new_y = (scroll_pos.0.y - scroll_amount).max(0.0);
            
            // 限制最大滚动距离（内容高度 - 可见高度）
            let max_scroll = (computed.content_size().y - computed.size().y).max(0.0);
            scroll_pos.0.y = new_y.min(max_scroll);
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

/// 处理分析结果区域的鼠标滚轮滚动
pub fn handle_analysis_scroll(
    mut scroll_events: MessageReader<MouseWheel>,
    mut query: Query<(&Interaction, &mut ScrollPosition, &ComputedNode), With<super::AnalysisScrollContainer>>,
) {
    for event in scroll_events.read() {
        for (interaction, mut scroll_pos, computed) in &mut query {
            // 只在鼠标悬停时响应滚轮
            if *interaction != Interaction::Hovered && *interaction != Interaction::Pressed {
                continue;
            }
            
            let scroll_amount = match event.unit {
                bevy::input::mouse::MouseScrollUnit::Line => event.y * 30.0,
                bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
            };
            
            // 更新滚动位置（向上滚动减少 y，向下滚动增加 y）
            let new_y = (scroll_pos.0.y - scroll_amount).max(0.0);
            
            // 限制最大滚动距离（内容高度 - 可见高度）
            let max_scroll = (computed.content_size().y - computed.size().y).max(0.0);
            scroll_pos.0.y = new_y.min(max_scroll);
        }
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
                            column_gap: Val::Px(15.0),
                            margin: UiRect::top(Val::Px(10.0)),
                            flex_wrap: FlexWrap::Wrap,
                            row_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            // 悔棋按钮（仅本地模式且有历史记录时显示）
                            if game.is_local() && game.can_undo_at_game_over() {
                                spawn_result_button(
                                    parent,
                                    &asset_server,
                                    "悔棋",
                                    ButtonAction::Undo,
                                    Color::srgb(0.5, 0.4, 0.2),
                                );
                            }

                            // AI 复盘分析按钮（仅在 llm feature 启用时显示）
                            #[cfg(feature = "llm")]
                            spawn_result_button(
                                parent,
                                &asset_server,
                                "AI 复盘",
                                ButtonAction::AiAnalysis,
                                Color::srgb(0.4, 0.3, 0.6),
                            );

                            // 返回主菜单按钮
                            spawn_result_button(
                                parent,
                                &asset_server,
                                "返回主菜单",
                                ButtonAction::BackToMenu,
                                Color::srgb(0.3, 0.3, 0.3),
                            );

                            // 导出棋谱按钮
                            spawn_result_button(
                                parent,
                                &asset_server,
                                "导出棋谱",
                                ButtonAction::ExportGame,
                                Color::srgb(0.3, 0.4, 0.5),
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
    parent: &mut ChildSpawnerCommands,
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
        commands.entity(entity).despawn();
    }
}

/// 处理游戏结束界面的按钮点击
#[allow(unused_variables, unused_mut)] // 部分参数仅在 llm feature 启用时使用
pub fn handle_game_over_buttons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut game: ResMut<ClientGame>,
    mut network_events: MessageWriter<NetworkEvent>,
    conn_handle: Res<crate::network::NetworkConnectionHandle>,
    settings: Res<crate::settings::GameSettings>,
    mut analysis_state: ResMut<super::AiAnalysisState>,
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
                    ButtonAction::ExportGame => {
                        export_game_record(&game, &settings);
                    }
                    #[cfg(feature = "llm")]
                    ButtonAction::AiAnalysis => {
                        tracing::info!("AI Analysis clicked");
                        // 设置分析状态
                        analysis_state.is_analyzing = true;
                        analysis_state.result = None;
                        analysis_state.error = None;
                        // 启动异步分析任务
                        start_ai_analysis(&game, &settings, commands.reborrow(), asset_server.clone());
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
                                network_events.write(NetworkEvent::Connect {
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
                    ButtonAction::Undo => {
                        tracing::info!("Undo at game over clicked");
                        if game.is_local() && game.local_undo() {
                            tracing::info!("终盘悔棋成功，返回游戏");
                            game_state.set(GameState::Playing);
                        } else {
                            tracing::warn!("终盘悔棋失败");
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

// ============================================================================
// LLM 分析功能（仅在启用 llm feature 时编译）
// ============================================================================

/// AI 分析任务组件
#[cfg(feature = "llm")]
#[derive(Component)]
pub struct AiAnalysisTask(pub bevy::tasks::Task<Result<chess_ai::llm::GameAnalysis, String>>);

/// 启动 AI 分析任务
#[cfg(feature = "llm")]
fn start_ai_analysis(
    game: &ClientGame,
    settings: &crate::settings::GameSettings,
    mut commands: Commands,
    asset_server: AssetServer,
) {
    use bevy::tasks::AsyncComputeTaskPool;

    // 准备分析所需的数据
    let Some(ref board_state) = game.game_state else {
        tracing::warn!("无法分析：游戏状态为空");
        return;
    };

    let state = board_state.clone();
    let move_history: Vec<protocol::Move> = game.move_history.iter()
        .map(|r| protocol::Move::new(r.from, r.to))
        .collect();

    // 确定玩家名称
    let red_player = settings.nickname.clone();
    let black_player = match &game.game_mode {
        Some(crate::game::GameMode::LocalPvE { difficulty }) => {
            format!("AI-{:?}", difficulty)
        }
        Some(crate::game::GameMode::OnlinePvE { difficulty, .. }) => {
            format!("AI-{:?}", difficulty)
        }
        _ => "对手".to_string(),
    };

    // 确定游戏结果
    let result = match &game.game_result {
        Some(protocol::GameResult::RedWin(reason)) => format!("红方胜（{:?}）", reason),
        Some(protocol::GameResult::BlackWin(reason)) => format!("黑方胜（{:?}）", reason),
        Some(protocol::GameResult::Draw(reason)) => format!("和棋（{:?}）", reason),
        None => "未知结果".to_string(),
    };

    // LLM 配置
    let llm_base_url = settings.llm_base_url.clone();
    let llm_model = settings.llm_model.clone();

    // 显示加载界面
    spawn_analysis_loading_ui(&mut commands, &asset_server);

    // 创建异步任务
    // 注意：Bevy 的 AsyncComputeTaskPool 不是 tokio runtime，
    // 所以需要在任务中创建独立的 tokio runtime 来执行 HTTP 请求
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move {
        tracing::info!("Starting AI analysis task");

        // 创建独立的 tokio runtime 来执行 HTTP 请求
        // 因为 Bevy 的 AsyncComputeTaskPool 不兼容 reqwest
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                tracing::error!("Failed to create tokio runtime: {}", e);
                return Err("无法创建异步运行时".to_string());
            }
        };

        rt.block_on(async {
            // 创建 LLM 引擎
            let config = chess_ai::llm::OllamaConfig {
                base_url: llm_base_url.clone(),
                model: llm_model,
                max_tokens: 4096,  // 分析需要更多 tokens
                temperature: 0.7,
                timeout_secs: 120,  // 分析可能需要更长时间
            };

            let mut engine = match chess_ai::llm::LlmEngine::new(config) {
                Ok(e) => e,
                Err(e) => {
                    tracing::error!("Failed to create LLM engine: {}", e);
                    return Err(format!("无法创建 LLM 引擎: {}", e));
                }
            };

            // P2 修复：先检查 LLM 服务是否可用，返回具体错误信息
            tracing::info!("Checking LLM service availability at {}", llm_base_url);
            if let Err(e) = engine.check_available().await {
                tracing::error!("LLM service check failed: {}", e);
                return Err(format!("LLM 服务不可用: {}", e));
            }
            tracing::info!("LLM service is available");

            // 添加走法历史
            for mv in &move_history {
                engine.add_move(*mv);
            }

            // 执行分析
            match engine.analyze_game(&state, &result, &red_player, &black_player).await {
                Ok(analysis) => {
                    tracing::info!("AI analysis completed successfully");
                    Ok(analysis)
                }
                Err(e) => {
                    tracing::error!("AI analysis failed: {}", e);
                    Err(format!("分析失败: {}", e))
                }
            }
        })
    });

    // 将任务添加为实体组件
    commands.spawn(AiAnalysisTask(task));
}

/// 直接生成加载界面（不使用系统）
#[cfg(feature = "llm")]
fn spawn_analysis_loading_ui(commands: &mut Commands, asset_server: &AssetServer) {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            UiMarker,
            super::AnalysisUiMarker,
            super::AnalysisLoadingMarker,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(50.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderRadius::all(Val::Px(12.0)),
                ))
                .with_children(|parent| {
                    // 标题
                    parent.spawn((
                        Text::new("AI 复盘分析"),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(30.0)),
                            ..default()
                        },
                    ));

                    // 加载动画
                    parent.spawn((
                        Text::new(format!("{} AI 正在分析对局...", icons::INFO)),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // 提示信息
                    parent.spawn((
                        Text::new("预计需要 10-30 秒"),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        Node {
                            margin: UiRect::bottom(Val::Px(30.0)),
                            ..default()
                        },
                    ));

                    // 取消按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.2, 0.2)),
                            BorderRadius::all(Val::Px(6.0)),
                            ButtonAction::CloseAnalysis,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("取消"),
                                TextFont {
                                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

/// 轮询 AI 分析任务结果
#[cfg(feature = "llm")]
pub fn poll_ai_analysis_task(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut analysis_state: ResMut<super::AiAnalysisState>,
    mut task_query: Query<(Entity, &mut AiAnalysisTask)>,
    loading_query: Query<Entity, With<super::AnalysisLoadingMarker>>,
) {
    for (entity, mut task) in &mut task_query {
        if task.0.is_finished() {
            // 获取结果
            let result = bevy::tasks::block_on(bevy::tasks::poll_once(&mut task.0));

            // 清理任务实体
            commands.entity(entity).despawn();

            // 清理加载界面
            for loading_entity in loading_query.iter() {
                commands.entity(loading_entity).despawn();
            }

            // 更新分析状态
            analysis_state.is_analyzing = false;

            match result {
                Some(Ok(analysis)) => {
                    // 对 LLM 返回的内容进行 Emoji 替换，确保字体兼容
                    let sanitized = icons::sanitize_analysis(analysis);
                    analysis_state.result = Some(sanitized);
                    // 显示结果界面
                    spawn_analysis_result_ui(&mut commands, &asset_server, &analysis_state);
                }
                Some(Err(error_msg)) => {
                    // P2 改进：显示具体错误信息
                    analysis_state.error = Some(error_msg.clone());
                    tracing::error!("AI analysis failed: {}", error_msg);
                    // 显示错误界面
                    spawn_analysis_error_ui(&mut commands, &asset_server, &error_msg);
                }
                None => {
                    analysis_state.error = Some("分析任务异常终止".to_string());
                    tracing::error!("AI analysis task returned None (task aborted?)");
                }
            }
        }
    }
}

/// 生成分析错误界面
#[cfg(feature = "llm")]
fn spawn_analysis_error_ui(
    commands: &mut Commands,
    asset_server: &AssetServer,
    error_msg: &str,
) {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            UiMarker,
            super::AnalysisUiMarker,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        max_width: Val::Px(500.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    BorderRadius::all(Val::Px(12.0)),
                ))
                .with_children(|parent| {
                    // 错误图标
                    parent.spawn((
                        Text::new(icons::ERROR),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 48.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.3, 0.3)),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));

                    // 标题
                    parent.spawn((
                        Text::new("分析失败"),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },
                    ));

                    // 错误信息
                    parent.spawn((
                        Text::new(error_msg),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        Node {
                            margin: UiRect::bottom(Val::Px(25.0)),
                            ..default()
                        },
                    ));

                    // 提示信息
                    parent.spawn((
                        Text::new("请确保 Ollama 服务正在运行"),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        Node {
                            margin: UiRect::bottom(Val::Px(25.0)),
                            ..default()
                        },
                    ));

                    // 关闭按钮
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(120.0),
                                height: Val::Px(40.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            BorderRadius::all(Val::Px(6.0)),
                            ButtonAction::CloseAnalysis,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("关闭"),
                                TextFont {
                                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

/// 直接生成分析结果界面
#[cfg(feature = "llm")]
fn spawn_analysis_result_ui(
    commands: &mut Commands,
    asset_server: &AssetServer,
    analysis_state: &super::AiAnalysisState,
) {
    let Some(ref analysis) = analysis_state.result else {
        return;
    };

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            UiMarker,
            super::AnalysisUiMarker,
            super::AnalysisResultMarker,
        ))
        .with_children(|parent| {
            // 主容器（可滚动）
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(700.0),
                        max_height: Val::Percent(90.0),
                        padding: UiRect::all(Val::Px(30.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
                    BorderRadius::all(Val::Px(12.0)),
                    ScrollPosition::default(),
                    Interaction::default(),
                    super::AnalysisScrollContainer,
                ))
                .with_children(|parent| {
                    // 标题栏
                    spawn_analysis_title_bar(parent, asset_server);

                    // 整体评分
                    spawn_analysis_overall_rating(parent, asset_server, &analysis.overall_rating);

                    // 开局评价
                    spawn_analysis_opening_review(parent, asset_server, &analysis.opening_review);

                    // 关键时刻
                    spawn_analysis_key_moments(parent, asset_server, &analysis.key_moments);

                    // 残局评价
                    spawn_analysis_endgame_review(parent, asset_server, &analysis.endgame_review);

                    // 不足与提升
                    spawn_analysis_weaknesses(parent, asset_server, &analysis.weaknesses);

                    // 改进建议
                    spawn_analysis_suggestions(parent, asset_server, &analysis.suggestions);

                    // 底部按钮
                    spawn_analysis_bottom_buttons(parent, asset_server);
                });
        });
}

/// 生成分析标题栏
#[cfg(feature = "llm")]
fn spawn_analysis_title_bar(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            margin: UiRect::bottom(Val::Px(20.0)),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("AI 复盘分析"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 关闭按钮
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(36.0),
                        height: Val::Px(36.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    BorderRadius::all(Val::Px(18.0)),
                    ButtonAction::CloseAnalysis,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(icons::CLOSE),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// 生成分析区域容器
#[cfg(feature = "llm")]
fn spawn_analysis_section<F>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    title: &str,
    content: F,
) where
    F: FnOnce(&mut ChildSpawnerCommands),
{
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(15.0)),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|parent| {
            // 区域标题
            parent.spawn((
                Text::new(format!("─ {} ─", title)),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.8, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(12.0)),
                    ..default()
                },
            ));

            content(parent);
        });
}

/// 生成整体评分区域
#[cfg(feature = "llm")]
fn spawn_analysis_overall_rating(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    rating: &chess_ai::llm::OverallRating,
) {
    spawn_analysis_section(parent, asset_server, "整体评分", |parent| {
        // 评分行
        spawn_analysis_rating_row(parent, asset_server, "红方棋力", rating.red_play_quality);
        spawn_analysis_rating_row(parent, asset_server, "黑方棋力", rating.black_play_quality);
        spawn_analysis_rating_row(parent, asset_server, "对局精彩度", rating.game_quality);

        // 总结
        parent.spawn((
            Text::new(&rating.summary),
            TextFont {
                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Node {
                margin: UiRect::top(Val::Px(10.0)),
                ..default()
            },
        ));
    });
}

/// 生成评分行
#[cfg(feature = "llm")]
fn spawn_analysis_rating_row(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    label: &str,
    score: f32,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            margin: UiRect::bottom(Val::Px(8.0)),
            ..default()
        })
        .with_children(|parent| {
            // 标签
            parent.spawn((
                Text::new(format!("{}: ", label)),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    width: Val::Px(100.0),
                    ..default()
                },
            ));

            // 星级
            let stars = chess_ai::llm::OverallRating::stars(score);
            parent.spawn((
                Text::new(stars),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.84, 0.0)),
            ));

            // 分数
            parent.spawn((
                Text::new(format!(" {:.1}/10", score)),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

/// 生成开局评价区域
#[cfg(feature = "llm")]
fn spawn_analysis_opening_review(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    review: &chess_ai::llm::OpeningReview,
) {
    spawn_analysis_section(parent, asset_server, "开局评价", |parent| {
        if let Some(ref name) = review.name {
            parent.spawn((
                Text::new(format!("开局: {}", name)),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.7, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));
        }

        parent.spawn((
            Text::new(format!("评价: {}", review.evaluation)),
            TextFont {
                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
        ));

        parent.spawn((
            Text::new(&review.comment),
            TextFont {
                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });
}

/// 生成关键时刻区域
#[cfg(feature = "llm")]
fn spawn_analysis_key_moments(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    moments: &[chess_ai::llm::KeyMoment],
) {
    spawn_analysis_section(parent, asset_server, "关键时刻", |parent| {
        if moments.is_empty() {
            parent.spawn((
                Text::new("无特别关键的时刻"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
            return;
        }

        for moment in moments {
            spawn_analysis_key_moment(parent, asset_server, moment);
        }
    });
}

/// 生成单个关键时刻
#[cfg(feature = "llm")]
fn spawn_analysis_key_moment(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    moment: &chess_ai::llm::KeyMoment,
) {
    let (icon, type_color) = match moment.moment_type {
        chess_ai::llm::MomentType::Brilliant => (icons::BRILLIANT, Color::srgb(1.0, 0.84, 0.0)),
        chess_ai::llm::MomentType::Mistake => (icons::MISTAKE, Color::srgb(0.9, 0.3, 0.3)),
        chess_ai::llm::MomentType::TurningPoint => (icons::TURNING_POINT, Color::srgb(0.3, 0.7, 0.9)),
    };

    let side_name = match moment.side {
        chess_ai::llm::MomentSide::Red => "红方",
        chess_ai::llm::MomentSide::Black => "黑方",
    };

    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.5)),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|parent| {
            // 标题行
            parent.spawn((
                Text::new(format!(
                    "{} 第{}步 {} {}",
                    icon, moment.move_number, side_name, moment.move_notation
                )),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(type_color),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            // 类型标签
            parent.spawn((
                Text::new(format!("[{}]", moment.moment_type.display_name())),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(type_color),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            // 分析
            parent.spawn((
                Text::new(&moment.analysis),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

/// 生成残局评价区域
#[cfg(feature = "llm")]
fn spawn_analysis_endgame_review(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    review: &chess_ai::llm::EndgameReview,
) {
    spawn_analysis_section(parent, asset_server, "残局评价", |parent| {
        parent.spawn((
            Text::new(format!("评价: {}", review.evaluation)),
            TextFont {
                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(5.0)),
                ..default()
            },
        ));

        parent.spawn((
            Text::new(&review.comment),
            TextFont {
                font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
    });
}

/// 生成不足与提升区域
#[cfg(feature = "llm")]
fn spawn_analysis_weaknesses(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    weaknesses: &chess_ai::llm::Weaknesses,
) {
    // 如果双方都没有不足，不显示此区域
    if weaknesses.red.is_empty() && weaknesses.black.is_empty() {
        return;
    }

    spawn_analysis_section(parent, asset_server, "不足与提升", |parent| {
        // 红方不足
        if !weaknesses.red.is_empty() {
            parent.spawn((
                Text::new("红方不足:"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.5, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            for weakness in &weaknesses.red {
                parent.spawn((
                    Text::new(format!("• {}", weakness)),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::new(Val::Px(15.0), Val::Px(0.0), Val::Px(0.0), Val::Px(3.0)),
                        ..default()
                    },
                ));
            }
        }

        // 黑方不足
        if !weaknesses.black.is_empty() {
            parent.spawn((
                Text::new("黑方不足:"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(15.0), Val::Px(5.0)),
                    ..default()
                },
            ));

            for weakness in &weaknesses.black {
                parent.spawn((
                    Text::new(format!("• {}", weakness)),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::new(Val::Px(15.0), Val::Px(0.0), Val::Px(0.0), Val::Px(3.0)),
                        ..default()
                    },
                ));
            }
        }
    });
}

/// 生成改进建议区域
#[cfg(feature = "llm")]
fn spawn_analysis_suggestions(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    suggestions: &chess_ai::llm::Suggestions,
) {
    spawn_analysis_section(parent, asset_server, "改进建议", |parent| {
        // 红方建议
        if !suggestions.red.is_empty() {
            parent.spawn((
                Text::new("给红方:"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.5, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            for suggestion in &suggestions.red {
                parent.spawn((
                    Text::new(format!("• {}", suggestion)),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::new(Val::Px(15.0), Val::Px(0.0), Val::Px(0.0), Val::Px(3.0)),
                        ..default()
                    },
                ));
            }
        }

        // 黑方建议
        if !suggestions.black.is_empty() {
            parent.spawn((
                Text::new("给黑方:"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(15.0), Val::Px(5.0)),
                    ..default()
                },
            ));

            for suggestion in &suggestions.black {
                parent.spawn((
                    Text::new(format!("• {}", suggestion)),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::new(Val::Px(15.0), Val::Px(0.0), Val::Px(0.0), Val::Px(3.0)),
                        ..default()
                    },
                ));
            }
        }

        if suggestions.red.is_empty() && suggestions.black.is_empty() {
            parent.spawn((
                Text::new("暂无具体建议"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        }
    });
}

/// 生成底部按钮
#[cfg(feature = "llm")]
fn spawn_analysis_bottom_buttons(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(15.0),
            margin: UiRect::top(Val::Px(20.0)),
            ..default()
        })
        .with_children(|parent| {
            // 关闭按钮
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(140.0),
                        height: Val::Px(45.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    BorderRadius::all(Val::Px(8.0)),
                    ButtonAction::CloseAnalysis,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("关闭"),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

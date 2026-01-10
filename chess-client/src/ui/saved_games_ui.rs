//! 保存的棋局列表 UI

use bevy::prelude::*;

use super::{UiMarker, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::game::{ClientGame, GameMode};
use crate::storage::{SavedGameInfo, StorageManager};
use crate::GameState;
use protocol::{Difficulty, Fen};

/// 保存棋局页面标记
#[derive(Component)]
pub struct SavedGamesMarker;

/// 棋局列表容器标记
#[derive(Component)]
pub struct SavedGamesListContainer;

/// 棋局项按钮动作
#[derive(Component, Clone, Debug)]
pub enum SavedGameAction {
    /// 加载棋局
    Load(String),
    /// 删除棋局
    Delete(String),
    /// 返回主菜单
    BackToMenu,
    /// 刷新列表
    Refresh,
}

/// 保存的棋局列表资源
#[derive(Resource, Default)]
pub struct SavedGamesList {
    pub games: Vec<SavedGameInfo>,
    pub selected: Option<String>,
    pub error_message: Option<String>,
}

/// 设置保存棋局列表页面
pub fn setup_saved_games(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut saved_games: ResMut<SavedGamesList>,
) {
    // 加载棋局列表
    match StorageManager::new() {
        Ok(storage) => match storage.list_saved_games() {
            Ok(games) => {
                saved_games.games = games;
                saved_games.error_message = None;
            }
            Err(e) => {
                saved_games.error_message = Some(format!("加载棋局列表失败: {}", e));
                saved_games.games.clear();
            }
        },
        Err(e) => {
            saved_games.error_message = Some(format!("初始化存储失败: {}", e));
            saved_games.games.clear();
        }
    }

    // 根容器
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            UiMarker,
            SavedGamesMarker,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("保存的棋局"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // 棋局列表容器
            parent
                .spawn((
                    Node {
                        width: Val::Px(700.0),
                        height: Val::Px(400.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(15.0)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
                    BorderRadius::all(Val::Px(8.0)),
                    SavedGamesListContainer,
                ))
                .with_children(|_parent| {
                    // 内容将在 update_saved_games_list 中动态生成
                });

            // 底部按钮栏
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                })
                .with_children(|parent| {
                    // 刷新按钮
                    spawn_action_button(parent, &asset_server, "刷新", SavedGameAction::Refresh);
                    // 返回按钮
                    spawn_action_button(parent, &asset_server, "返回", SavedGameAction::BackToMenu);
                });
        });
}

/// 生成操作按钮
fn spawn_action_button(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &str,
    action: SavedGameAction,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(150.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
            BorderRadius::all(Val::Px(6.0)),
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

/// 更新棋局列表显示
pub fn update_saved_games_list(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    saved_games: Res<SavedGamesList>,
    container_query: Query<(Entity, Option<&Children>), With<SavedGamesListContainer>>,
) {
    if !saved_games.is_changed() {
        return;
    }

    for (entity, children) in container_query.iter() {
        // 清除旧内容
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        // 生成新内容
        commands.entity(entity).with_children(|parent| {
            // 显示错误信息
            if let Some(ref error) = saved_games.error_message {
                parent.spawn((
                    Text::new(error.clone()),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.4, 0.4)),
                ));
                return;
            }

            // 空列表提示
            if saved_games.games.is_empty() {
                parent.spawn((
                    Text::new("暂无保存的棋局"),
                    TextFont {
                        font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    Node {
                        margin: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                ));
                return;
            }

            // 生成棋局列表项
            for game in &saved_games.games {
                spawn_game_item(parent, &asset_server, game);
            }
        });
    }
}

/// 生成棋局列表项
fn spawn_game_item(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    game: &SavedGameInfo,
) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
            BorderRadius::all(Val::Px(6.0)),
        ))
        .with_children(|parent| {
            // 左侧信息
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    // 对局名称
                    parent.spawn((
                        Text::new(game.display_name()),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // 详细信息
                    parent.spawn((
                        Text::new(format!(
                            "{} · {} 步",
                            game.formatted_time(),
                            game.move_count
                        )),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                });

            // 右侧按钮
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|parent| {
                    // 加载按钮
                    spawn_small_button(
                        parent,
                        asset_server,
                        "加载",
                        SavedGameAction::Load(game.game_id.clone()),
                        Color::srgb(0.2, 0.5, 0.3),
                    );

                    // 删除按钮
                    spawn_small_button(
                        parent,
                        asset_server,
                        "删除",
                        SavedGameAction::Delete(game.game_id.clone()),
                        Color::srgb(0.5, 0.2, 0.2),
                    );
                });
        });
}

/// 生成小按钮
fn spawn_small_button(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &str,
    action: SavedGameAction,
    bg_color: Color,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(70.0),
                height: Val::Px(35.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            BorderRadius::all(Val::Px(4.0)),
            action,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 清理保存棋局页面
pub fn cleanup_saved_games(mut commands: Commands, query: Query<Entity, With<SavedGamesMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 处理保存棋局页面的按钮点击
pub fn handle_saved_games_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &SavedGameAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut game: ResMut<ClientGame>,
    mut saved_games: ResMut<SavedGamesList>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        let base_color = match action {
            SavedGameAction::Load(_) => Color::srgb(0.2, 0.5, 0.3),
            SavedGameAction::Delete(_) => Color::srgb(0.5, 0.2, 0.2),
            _ => NORMAL_BUTTON,
        };

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                match action {
                    SavedGameAction::Load(game_id) => {
                        if let Err(e) = load_game(&game_id, &mut game) {
                            saved_games.error_message = Some(format!("加载失败: {}", e));
                        } else {
                            game_state.set(GameState::Playing);
                        }
                    }
                    SavedGameAction::Delete(game_id) => {
                        if let Err(e) = delete_game(&game_id) {
                            saved_games.error_message = Some(format!("删除失败: {}", e));
                        } else {
                            // 刷新列表
                            refresh_saved_games(&mut saved_games);
                        }
                    }
                    SavedGameAction::BackToMenu => {
                        game_state.set(GameState::Menu);
                    }
                    SavedGameAction::Refresh => {
                        refresh_saved_games(&mut saved_games);
                    }
                }
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = base_color.into();
            }
        }
    }
}

/// 加载棋局
fn load_game(game_id: &str, game: &mut ClientGame) -> anyhow::Result<()> {
    let storage = StorageManager::new()?;
    let record = storage.load_game(game_id)?;

    // 解析难度
    let difficulty = record
        .metadata
        .ai_difficulty
        .as_ref()
        .map(|d| match d.as_str() {
            "Easy" | "简单" => Difficulty::Easy,
            "Hard" | "困难" => Difficulty::Hard,
            _ => Difficulty::Medium,
        })
        .unwrap_or(Difficulty::Medium);

    // 重建游戏状态
    let mut board_state = Fen::parse(&record.initial_fen)
        .map_err(|e| anyhow::anyhow!("无效的 FEN 字符串: {:?}", e))?;

    // 重放所有走法
    for mv in &record.moves {
        if let (Some(from), Some(to)) = (mv.from_position(), mv.to_position()) {
            board_state.board.move_piece(from, to);
            board_state.switch_turn();
        }
    }

    // 初始化游戏
    game.start_game(
        board_state.clone(),
        protocol::Side::Red,
        GameMode::LocalPvE { difficulty },
    );

    // 恢复走法历史（使用 filter_map 一次性构建）
    game.move_history = record
        .moves
        .iter()
        .filter_map(|mv| {
            mv.from_position().and_then(|from| {
                mv.to_position().map(|to| crate::game::MoveRecord {
                    notation: mv.notation.clone(),
                    from,
                    to,
                })
            })
        })
        .collect();

    // 恢复时间
    if let Some(ref save_info) = record.save_info {
        game.red_time_ms = save_info.red_time_remaining_ms;
        game.black_time_ms = save_info.black_time_remaining_ms;
    }

    tracing::info!("棋局已加载: {}", game_id);
    Ok(())
}

/// 删除棋局
fn delete_game(game_id: &str) -> anyhow::Result<()> {
    let storage = StorageManager::new()?;
    storage.delete_game(game_id)
}

/// 刷新棋局列表
fn refresh_saved_games(saved_games: &mut SavedGamesList) {
    match StorageManager::new() {
        Ok(storage) => match storage.list_saved_games() {
            Ok(games) => {
                saved_games.games = games;
                saved_games.error_message = None;
            }
            Err(e) => {
                saved_games.error_message = Some(format!("加载棋局列表失败: {}", e));
            }
        },
        Err(e) => {
            saved_games.error_message = Some(format!("初始化存储失败: {}", e));
        }
    }
}

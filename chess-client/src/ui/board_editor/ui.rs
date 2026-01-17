//! 棋盘编辑器主 UI

use bevy::prelude::*;
use protocol::{BoardState, Side, Difficulty, Fen};

use super::{
    BoardEditorState, OpponentType,
    spawn_piece_palette, spawn_editor_board,
    validate_board,
};
use crate::game::{ClientGame, GameMode};
use crate::ui::{UiMarker, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};
use crate::GameState;

/// 编辑器页面标记
#[derive(Component)]
pub struct BoardEditorMarker;

/// 编辑器按钮动作
#[derive(Component, Clone, Debug)]
pub enum EditorAction {
    /// 返回
    Back,
    /// 开始对局
    StartGame,
    /// 保存布局
    SaveLayout,
    /// 确认警告继续
    ConfirmWarning,
    /// 取消警告
    CancelWarning,
}

/// 设置面板标记
#[derive(Component)]
pub struct SettingsPanelMarker;

/// 先手选择单选按钮
#[derive(Component, Clone)]
pub struct FirstTurnRadio(pub Side);

/// 对手类型单选按钮
#[derive(Component, Clone)]
pub struct OpponentTypeRadio(pub OpponentType);

/// AI 难度单选按钮
#[derive(Component, Clone)]
pub struct DifficultyRadio(pub Difficulty);

/// 执子方单选按钮
#[derive(Component, Clone)]
pub struct PlayerSideRadio(pub Side);

/// 验证消息容器
#[derive(Component)]
pub struct ValidationMessageContainer;

/// 警告对话框标记
#[derive(Component)]
pub struct WarningDialogMarker;

/// AI 设置容器（根据对手类型显示/隐藏）
#[derive(Component)]
pub struct AiSettingsContainer;

/// 设置棋盘编辑器 UI
pub fn setup_board_editor(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut editor_state: ResMut<BoardEditorState>,
) {
    // 重置编辑器状态
    *editor_state = BoardEditorState::default();

    // 根容器
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            UiMarker,
            BoardEditorMarker,
        ))
        .with_children(|parent| {
            // 顶部栏
            spawn_top_bar(parent, &asset_server);

            // 主内容区域
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Row,
                    padding: UiRect::all(Val::Px(20.0)),
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|parent| {
                    // 左侧：棋子面板
                    spawn_piece_palette(parent, &asset_server);

                    // 中间：棋盘
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_editor_board(parent, &asset_server);

                            // 提示文字
                            parent.spawn((
                                Text::new("点击棋盘放置棋子，右键删除棋子，ESC 取消选择"),
                                TextFont {
                                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                            ));
                        });

                    // 右侧：设置面板
                    spawn_settings_panel(parent, &asset_server);
                });

            // 底部验证消息
            spawn_validation_area(parent, &asset_server);
        });
}

/// 生成顶部栏
fn spawn_top_bar(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            // 返回按钮
            spawn_editor_button(parent, asset_server, "返回", EditorAction::Back);

            // 标题
            parent.spawn((
                Text::new("棋盘编辑器"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.6)),
            ));

            // 右侧按钮组
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_editor_button(parent, asset_server, "保存布局", EditorAction::SaveLayout);
                    spawn_editor_button(parent, asset_server, "开始对局", EditorAction::StartGame);
                });
        });
}

/// 生成设置面板
fn spawn_settings_panel(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn((
            Node {
                width: Val::Px(250.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(15.0)),
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
            BorderRadius::all(Val::Px(8.0)),
            SettingsPanelMarker,
        ))
        .with_children(|parent| {
            // 标题
            parent.spawn((
                Text::new("对局设置"),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.8, 0.6)),
            ));

            // 先手选择
            spawn_radio_group(
                parent,
                asset_server,
                "先手方",
                vec![
                    ("红方先手", FirstTurnRadio(Side::Red)),
                    ("黑方先手", FirstTurnRadio(Side::Black)),
                ],
            );

            // 对手类型
            spawn_radio_group(
                parent,
                asset_server,
                "对手类型",
                vec![
                    ("人机对战", OpponentTypeRadio(OpponentType::AI)),
                    ("双人本地", OpponentTypeRadio(OpponentType::LocalPvP)),
                ],
            );

            // AI 设置（仅人机模式显示）
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                    AiSettingsContainer,
                ))
                .with_children(|parent| {
                    // AI 难度
                    spawn_radio_group(
                        parent,
                        asset_server,
                        "AI 难度",
                        vec![
                            ("简单", DifficultyRadio(Difficulty::Easy)),
                            ("中等", DifficultyRadio(Difficulty::Medium)),
                            ("困难", DifficultyRadio(Difficulty::Hard)),
                        ],
                    );

                    // 执子方
                    spawn_radio_group(
                        parent,
                        asset_server,
                        "执子方",
                        vec![
                            ("执红", PlayerSideRadio(Side::Red)),
                            ("执黑", PlayerSideRadio(Side::Black)),
                        ],
                    );
                });
        });
}

/// 生成单选按钮组
fn spawn_radio_group<T: Component + Clone>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    label: &str,
    options: Vec<(&str, T)>,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(5.0),
            ..default()
        })
        .with_children(|parent| {
            // 标签
            parent.spawn((
                Text::new(label),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // 选项行
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(15.0),
                    ..default()
                })
                .with_children(|parent| {
                    for (text, component) in options {
                        spawn_radio_option(parent, asset_server, text, component);
                    }
                });
        });
}

/// 生成单选选项
fn spawn_radio_option<T: Component>(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &str,
    component: T,
) {
    parent
        .spawn((
            Button,
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(5.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            component,
        ))
        .with_children(|parent| {
            // 圆点
            parent.spawn((
                Node {
                    width: Val::Px(12.0),
                    height: Val::Px(12.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                BorderRadius::all(Val::Px(6.0)),
            ));

            // 文字
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

/// 生成验证消息区域
fn spawn_validation_area(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(40.0),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ValidationMessageContainer,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.4, 0.4)),
            ));
        });
}

/// 生成编辑器按钮
fn spawn_editor_button(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &str,
    action: EditorAction,
) {
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
            BackgroundColor(NORMAL_BUTTON),
            BorderRadius::all(Val::Px(6.0)),
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

/// 清理编辑器 UI
pub fn cleanup_board_editor(mut commands: Commands, query: Query<Entity, With<BoardEditorMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// 处理编辑器按钮点击
pub fn handle_editor_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &EditorAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state: ResMut<NextState<GameState>>,
    mut editor_state: ResMut<BoardEditorState>,
    mut game: ResMut<ClientGame>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                match action {
                    EditorAction::Back => {
                        game_state.set(GameState::SavedGames);
                    }
                    EditorAction::StartGame => {
                        // 验证棋局
                        let validation = validate_board(&editor_state.board, editor_state.first_turn);
                        editor_state.validation_errors = validation.errors.clone();
                        editor_state.validation_warnings = validation.warnings.clone();

                        if !validation.is_valid() {
                            // 有错误，不能开始
                            return;
                        }

                        if !validation.warnings.is_empty() {
                            // 有警告，显示确认对话框
                            editor_state.show_warning_dialog = true;
                            return;
                        }

                        // 开始对局
                        start_game_from_editor(&editor_state, &mut game);
                        game_state.set(GameState::Playing);
                    }
                    EditorAction::SaveLayout => {
                        save_editor_layout(&editor_state);
                    }
                    EditorAction::ConfirmWarning => {
                        editor_state.show_warning_dialog = false;
                        start_game_from_editor(&editor_state, &mut game);
                        game_state.set(GameState::Playing);
                    }
                    EditorAction::CancelWarning => {
                        editor_state.show_warning_dialog = false;
                    }
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

/// 从编辑器开始对局
fn start_game_from_editor(editor_state: &BoardEditorState, game: &mut ClientGame) {
    let board_state = BoardState::from_board(editor_state.board.clone(), editor_state.first_turn);
    
    // 保存初始 FEN 以便后续保存棋局
    let initial_fen = Fen::to_string(&board_state);

    let game_mode = match editor_state.opponent_type {
        OpponentType::AI => GameMode::LocalPvE {
            difficulty: editor_state.ai_difficulty,
        },
        OpponentType::LocalPvP => GameMode::LocalPvP,
    };

    tracing::info!("从编辑器开始对局: {:?}", game_mode);
    game.start_game_with_fen(board_state, editor_state.player_side, game_mode, initial_fen);
}

/// 保存编辑器布局
fn save_editor_layout(editor_state: &BoardEditorState) {
    use crate::storage::StorageManager;
    use protocol::{BoardState, GameRecord};

    let board_state = BoardState::from_board(editor_state.board.clone(), editor_state.first_turn);
    let fen = Fen::to_string(&board_state);

    // 创建一个只包含初始布局的棋谱记录
    let mut record = GameRecord::from_fen(
        "编辑器".to_string(),
        "布局".to_string(),
        fen,
    );

    // 根据对手类型设置 AI 难度
    if editor_state.opponent_type == OpponentType::AI {
        record.set_ai_difficulty(&format!("{:?}", editor_state.ai_difficulty));
    }

    // 保存
    match StorageManager::new() {
        Ok(storage) => {
            match storage.save_game(
                "编辑器",
                "布局",
                &mut record,
                &board_state,
                editor_state.player_side,
                600_000, // 10分钟默认时间
                600_000,
            ) {
                Ok(filename) => {
                    tracing::info!("布局已保存: {}", filename);
                }
                Err(e) => {
                    tracing::error!("保存布局失败: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::error!("初始化存储失败: {}", e);
        }
    }
}

/// 处理先手选择
pub fn handle_first_turn_radio(
    mut interaction_query: Query<
        (&Interaction, &FirstTurnRadio),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_state: ResMut<BoardEditorState>,
) {
    for (interaction, radio) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            editor_state.first_turn = radio.0;
        }
    }
}

/// 处理对手类型选择
pub fn handle_opponent_type_radio(
    mut interaction_query: Query<
        (&Interaction, &OpponentTypeRadio),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_state: ResMut<BoardEditorState>,
) {
    for (interaction, radio) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            editor_state.opponent_type = radio.0;
        }
    }
}

/// 处理 AI 难度选择
pub fn handle_difficulty_radio(
    mut interaction_query: Query<
        (&Interaction, &DifficultyRadio),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_state: ResMut<BoardEditorState>,
) {
    for (interaction, radio) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            editor_state.ai_difficulty = radio.0;
        }
    }
}

/// 处理执子方选择
pub fn handle_player_side_radio(
    mut interaction_query: Query<
        (&Interaction, &PlayerSideRadio),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_state: ResMut<BoardEditorState>,
) {
    for (interaction, radio) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            editor_state.player_side = radio.0;
        }
    }
}

/// 更新单选按钮显示状态
pub fn update_radio_buttons(
    editor_state: Res<BoardEditorState>,
    first_turn_query: Query<(&FirstTurnRadio, &Children)>,
    opponent_query: Query<(&OpponentTypeRadio, &Children)>,
    difficulty_query: Query<(&DifficultyRadio, &Children)>,
    player_side_query: Query<(&PlayerSideRadio, &Children)>,
    mut bg_query: Query<&mut BackgroundColor>,
) {
    if !editor_state.is_changed() {
        return;
    }

    // 更新先手选择
    for (radio, children) in &first_turn_query {
        if let Some(child) = children.iter().next() {
            if let Ok(mut bg) = bg_query.get_mut(child) {
                *bg = if radio.0 == editor_state.first_turn {
                    Color::srgb(0.3, 0.7, 0.3).into()
                } else {
                    Color::srgb(0.4, 0.4, 0.4).into()
                };
            }
        }
    }

    // 更新对手类型
    for (radio, children) in &opponent_query {
        if let Some(child) = children.iter().next() {
            if let Ok(mut bg) = bg_query.get_mut(child) {
                *bg = if radio.0 == editor_state.opponent_type {
                    Color::srgb(0.3, 0.7, 0.3).into()
                } else {
                    Color::srgb(0.4, 0.4, 0.4).into()
                };
            }
        }
    }

    // 更新 AI 难度
    for (radio, children) in &difficulty_query {
        if let Some(child) = children.iter().next() {
            if let Ok(mut bg) = bg_query.get_mut(child) {
                *bg = if radio.0 == editor_state.ai_difficulty {
                    Color::srgb(0.3, 0.7, 0.3).into()
                } else {
                    Color::srgb(0.4, 0.4, 0.4).into()
                };
            }
        }
    }

    // 更新执子方
    for (radio, children) in &player_side_query {
        if let Some(child) = children.iter().next() {
            if let Ok(mut bg) = bg_query.get_mut(child) {
                *bg = if radio.0 == editor_state.player_side {
                    Color::srgb(0.3, 0.7, 0.3).into()
                } else {
                    Color::srgb(0.4, 0.4, 0.4).into()
                };
            }
        }
    }
}

/// 更新 AI 设置容器可见性
pub fn update_ai_settings_visibility(
    editor_state: Res<BoardEditorState>,
    mut query: Query<&mut Visibility, With<AiSettingsContainer>>,
) {
    if !editor_state.is_changed() {
        return;
    }

    for mut visibility in &mut query {
        *visibility = if editor_state.opponent_type == OpponentType::AI {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// 更新验证消息显示
pub fn update_validation_display(
    editor_state: Res<BoardEditorState>,
    container_query: Query<&Children, With<ValidationMessageContainer>>,
    mut text_query: Query<(&mut Text, &mut TextColor)>,
) {
    if !editor_state.is_changed() {
        return;
    }

    for children in &container_query {
        for child in children.iter() {
            if let Ok((mut text, mut color)) = text_query.get_mut(child) {
                if !editor_state.validation_errors.is_empty() {
                    **text = editor_state.validation_errors.join(" | ");
                    *color = TextColor(Color::srgb(0.9, 0.3, 0.3));
                } else if !editor_state.validation_warnings.is_empty() {
                    **text = editor_state.validation_warnings.join(" | ");
                    *color = TextColor(Color::srgb(0.9, 0.7, 0.2));
                } else {
                    **text = String::new();
                }
            }
        }
    }
}

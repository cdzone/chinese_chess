//! 编辑器棋盘渲染

use bevy::prelude::*;
use protocol::{Position, Side};

use super::{BoardEditorState, piece_to_chinese};

/// 编辑器棋盘标记
#[derive(Component)]
pub struct EditorBoardMarker;

/// 编辑器棋盘格子标记
#[derive(Component)]
pub struct EditorBoardCell {
    pub x: u8,
    pub y: u8,
}

/// 编辑器棋子标记
#[derive(Component)]
pub struct EditorPiece {
    pub x: u8,
    pub y: u8,
}

/// 棋盘常量
const CELL_SIZE: f32 = 45.0;
const BOARD_PADDING: f32 = 20.0;
const LINE_COLOR: Color = Color::srgb(0.3, 0.2, 0.1);
const LINE_WIDTH: f32 = 1.5;

/// 生成编辑器棋盘
pub fn spawn_editor_board(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    // 棋盘实际绘制区域：8x9 个间隔（交叉点是棋子位置）
    let board_inner_width = CELL_SIZE * 8.0;
    let board_inner_height = CELL_SIZE * 9.0;
    let board_width = board_inner_width + BOARD_PADDING * 2.0;
    let board_height = board_inner_height + BOARD_PADDING * 2.0;

    parent
        .spawn((
            Node {
                width: Val::Px(board_width),
                height: Val::Px(board_height),
                padding: UiRect::all(Val::Px(BOARD_PADDING)),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::srgb(0.87, 0.72, 0.53)),
            BorderRadius::all(Val::Px(8.0)),
            EditorBoardMarker,
        ))
        .with_children(|parent| {
            // 棋盘线条层
            spawn_board_lines(parent);
            
            // 棋子交互层（覆盖在线条上）
            spawn_piece_grid(parent, asset_server);
        });
}

/// 生成棋盘线条
fn spawn_board_lines(parent: &mut ChildSpawnerCommands) {
    let board_inner_width = CELL_SIZE * 8.0;
    let board_inner_height = CELL_SIZE * 9.0;
    
    // 线条容器
    parent
        .spawn(Node {
            width: Val::Px(board_inner_width),
            height: Val::Px(board_inner_height),
            position_type: PositionType::Absolute,
            left: Val::Px(BOARD_PADDING),
            top: Val::Px(BOARD_PADDING),
            ..default()
        })
        .with_children(|parent| {
            // 横线 (10条)
            for i in 0..10 {
                let top = CELL_SIZE * i as f32 - LINE_WIDTH / 2.0;
                parent.spawn((
                    Node {
                        width: Val::Px(board_inner_width),
                        height: Val::Px(LINE_WIDTH),
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(top),
                        ..default()
                    },
                    BackgroundColor(LINE_COLOR),
                ));
            }

            // 竖线 - 左右边线贯穿
            parent.spawn((
                Node {
                    width: Val::Px(LINE_WIDTH),
                    height: Val::Px(board_inner_height),
                    position_type: PositionType::Absolute,
                    left: Val::Px(-LINE_WIDTH / 2.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(LINE_COLOR),
            ));
            parent.spawn((
                Node {
                    width: Val::Px(LINE_WIDTH),
                    height: Val::Px(board_inner_height),
                    position_type: PositionType::Absolute,
                    left: Val::Px(board_inner_width - LINE_WIDTH / 2.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(LINE_COLOR),
            ));

            // 竖线 - 中间线分为上下两段（楚河汉界）
            for i in 1..8 {
                let left = CELL_SIZE * i as f32 - LINE_WIDTH / 2.0;
                // 上半部分 (y: 5-9，对应顶部)
                parent.spawn((
                    Node {
                        width: Val::Px(LINE_WIDTH),
                        height: Val::Px(CELL_SIZE * 4.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(left),
                        top: Val::Px(0.0),
                        ..default()
                    },
                    BackgroundColor(LINE_COLOR),
                ));
                // 下半部分 (y: 0-4，对应底部)
                parent.spawn((
                    Node {
                        width: Val::Px(LINE_WIDTH),
                        height: Val::Px(CELL_SIZE * 4.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(left),
                        top: Val::Px(CELL_SIZE * 5.0),
                        ..default()
                    },
                    BackgroundColor(LINE_COLOR),
                ));
            }

            // 九宫格标记 - UI 不支持旋转，用边框标记九宫区域
            spawn_palace_markers(parent, board_inner_height);
        });
}

/// 生成九宫格标记（用边框表示九宫区域）
fn spawn_palace_markers(parent: &mut ChildSpawnerCommands, board_inner_height: f32) {
    let palace_width = CELL_SIZE * 2.0;
    let palace_height = CELL_SIZE * 2.0;
    let palace_left = CELL_SIZE * 3.0;
    
    // 黑方九宫区域边框（顶部）
    parent.spawn((
        Node {
            width: Val::Px(palace_width),
            height: Val::Px(palace_height),
            position_type: PositionType::Absolute,
            left: Val::Px(palace_left),
            top: Val::Px(0.0),
            border: UiRect::all(Val::Px(LINE_WIDTH)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.3, 0.2, 0.1, 0.3)),
        BackgroundColor(Color::NONE),
    ));

    // 红方九宫区域边框（底部）
    parent.spawn((
        Node {
            width: Val::Px(palace_width),
            height: Val::Px(palace_height),
            position_type: PositionType::Absolute,
            left: Val::Px(palace_left),
            top: Val::Px(board_inner_height - palace_height),
            border: UiRect::all(Val::Px(LINE_WIDTH)),
            ..default()
        },
        BorderColor::all(Color::srgba(0.3, 0.2, 0.1, 0.3)),
        BackgroundColor(Color::NONE),
    ));
}

/// 生成棋子交互网格
fn spawn_piece_grid(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    let board_inner_width = CELL_SIZE * 8.0;
    let board_inner_height = CELL_SIZE * 9.0;
    
    // 棋子网格容器（绝对定位，覆盖线条）
    parent
        .spawn(Node {
            width: Val::Px(board_inner_width + CELL_SIZE), // 9个交叉点
            height: Val::Px(board_inner_height + CELL_SIZE), // 10个交叉点
            position_type: PositionType::Absolute,
            left: Val::Px(BOARD_PADDING - CELL_SIZE / 2.0),
            top: Val::Px(BOARD_PADDING - CELL_SIZE / 2.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            // 生成 10 行（从 y=9 到 y=0）
            for y in (0..10).rev() {
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    })
                    .with_children(|parent| {
                        // 生成 9 列
                        for x in 0..9 {
                            spawn_board_cell(parent, asset_server, x, y);
                        }
                    });
            }
        });
}

/// 生成单个棋盘格子（交叉点）
fn spawn_board_cell(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer, x: u8, y: u8) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(CELL_SIZE),
                height: Val::Px(CELL_SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            EditorBoardCell { x, y },
        ))
        .with_children(|parent| {
            // 棋子占位符（初始为空）
            parent.spawn((
                Text::new(""),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::NONE),
                EditorPiece { x, y },
            ));
        });
}

/// 更新编辑器棋盘显示
pub fn update_editor_board(
    editor_state: Res<BoardEditorState>,
    mut piece_query: Query<(&EditorPiece, &mut Text, &mut TextColor)>,
) {
    if !editor_state.is_changed() {
        return;
    }

    for (editor_piece, mut text, mut text_color) in &mut piece_query {
        if let Some(pos) = Position::new(editor_piece.x, editor_piece.y) {
            if let Some(piece) = editor_state.board.get(pos) {
                **text = piece_to_chinese(piece).to_string();
                *text_color = if piece.side == Side::Red {
                    TextColor(Color::srgb(0.8, 0.1, 0.1))
                } else {
                    TextColor(Color::srgb(0.1, 0.1, 0.1))
                };
            } else {
                **text = String::new();
                *text_color = TextColor(Color::NONE);
            }
        }
    }
}

/// 处理棋盘格子点击
pub fn handle_board_cell_click(
    mut interaction_query: Query<
        (&Interaction, &EditorBoardCell, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut editor_state: ResMut<BoardEditorState>,
) {
    for (interaction, cell, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // 左键放置，右键删除
                if mouse_button.just_pressed(MouseButton::Left) {
                    if editor_state.selected_piece.is_some() {
                        editor_state.place_piece(cell.x, cell.y);
                    }
                }
            }
            Interaction::Hovered => {
                // 右键删除
                if mouse_button.just_pressed(MouseButton::Right) {
                    editor_state.remove_piece(cell.x, cell.y);
                }
                // 高亮显示
                *bg_color = Color::srgba(0.5, 0.8, 0.5, 0.3).into();
            }
            Interaction::None => {
                *bg_color = Color::NONE.into();
            }
        }
    }
}

/// 处理右键删除（单独系统，因为 Interaction 只触发一次）
pub fn handle_right_click_delete(
    mouse_button: Res<ButtonInput<MouseButton>>,
    interaction_query: Query<(&Interaction, &EditorBoardCell)>,
    mut editor_state: ResMut<BoardEditorState>,
) {
    if mouse_button.just_pressed(MouseButton::Right) {
        for (interaction, cell) in &interaction_query {
            if *interaction == Interaction::Hovered {
                editor_state.remove_piece(cell.x, cell.y);
                return;
            }
        }
    }
}

/// 处理 ESC 取消选择
pub fn handle_escape_deselect(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut editor_state: ResMut<BoardEditorState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        editor_state.deselect();
    }
}

//! 棋子选择面板

use bevy::prelude::*;
use protocol::{Piece, PieceType, Side};

use super::BoardEditorState;
use crate::ui::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};

/// 棋子面板标记
#[derive(Component)]
pub struct PiecePaletteMarker;

/// 棋子按钮标记
#[derive(Component)]
pub struct PieceButton(pub Piece);

/// 工具按钮动作
#[derive(Component, Clone, Debug)]
pub enum ToolAction {
    /// 清空棋盘
    Clear,
    /// 标准开局
    Initial,
    /// 仅将帅
    KingsOnly,
    /// 橡皮擦（删除模式）
    Eraser,
}

/// 橡皮擦模式标记
#[derive(Component)]
pub struct EraserModeIndicator;

/// 选中棋子指示器
#[derive(Component)]
pub struct SelectedPieceIndicator;

/// 生成棋子面板
pub fn spawn_piece_palette(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(10.0),
                width: Val::Px(180.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
            BorderRadius::all(Val::Px(8.0)),
            PiecePaletteMarker,
        ))
        .with_children(|parent| {
            // 红方棋子区域
            spawn_piece_section(parent, asset_server, Side::Red, "红方");

            // 分隔线
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
            ));

            // 黑方棋子区域
            spawn_piece_section(parent, asset_server, Side::Black, "黑方");

            // 分隔线
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
            ));

            // 工具区域
            spawn_tool_section(parent, asset_server);

            // 当前选中提示
            parent
                .spawn(Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("当前：无"),
                        TextFont {
                            font: asset_server.load("fonts/SourceHanSansSC-Regular.otf"),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        SelectedPieceIndicator,
                    ));
                });
        });
}

/// 生成棋子区域
fn spawn_piece_section(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    side: Side,
    title: &str,
) {
    // 标题
    parent.spawn((
        Text::new(title),
        TextFont {
            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
            font_size: 16.0,
            ..default()
        },
        TextColor(if side == Side::Red {
            Color::srgb(0.9, 0.3, 0.3)
        } else {
            Color::srgb(0.2, 0.2, 0.2)  // 深黑色，更明显
        }),
    ));

    // 棋子按钮网格
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(5.0),
            row_gap: Val::Px(5.0),
            ..default()
        })
        .with_children(|parent| {
            let pieces = [
                PieceType::King,
                PieceType::Advisor,
                PieceType::Bishop,
                PieceType::Rook,
                PieceType::Knight,
                PieceType::Cannon,
                PieceType::Pawn,
            ];

            for piece_type in pieces {
                let piece = Piece::new(piece_type, side);
                spawn_piece_button(parent, asset_server, piece);
            }
        });
}

/// 生成单个棋子按钮
fn spawn_piece_button(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer, piece: Piece) {
    let text = piece_to_chinese(piece);
    let color = if piece.side == Side::Red {
        Color::srgb(0.8, 0.2, 0.2)
    } else {
        Color::srgb(0.2, 0.2, 0.2)
    };

    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(40.0),
                height: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.9, 0.85, 0.7)),
            BorderRadius::all(Val::Px(20.0)),
            PieceButton(piece),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: 20.0,
                    ..default()
                },
                TextColor(color),
            ));
        });
}

/// 生成工具区域
fn spawn_tool_section(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent.spawn((
        Text::new("工具"),
        TextFont {
            font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.7, 0.7, 0.7)),
    ));

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(5.0),
            ..default()
        })
        .with_children(|parent| {
            spawn_tool_button(parent, asset_server, "清空棋盘", ToolAction::Clear);
            spawn_tool_button(parent, asset_server, "标准开局", ToolAction::Initial);
            spawn_tool_button(parent, asset_server, "仅将帅", ToolAction::KingsOnly);
            spawn_tool_button(parent, asset_server, "橡皮擦", ToolAction::Eraser);
        });
}

/// 生成工具按钮
fn spawn_tool_button(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    text: &str,
    action: ToolAction,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(NORMAL_BUTTON),
            BorderRadius::all(Val::Px(4.0)),
            action,
        ))
        .with_children(|parent| {
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

/// 棋子转中文
pub fn piece_to_chinese(piece: Piece) -> &'static str {
    match (piece.piece_type, piece.side) {
        (PieceType::King, Side::Red) => "帥",
        (PieceType::King, Side::Black) => "將",
        (PieceType::Advisor, Side::Red) => "仕",
        (PieceType::Advisor, Side::Black) => "士",
        (PieceType::Bishop, Side::Red) => "相",
        (PieceType::Bishop, Side::Black) => "象",
        (PieceType::Rook, _) => "車",
        (PieceType::Knight, Side::Red) => "馬",
        (PieceType::Knight, Side::Black) => "馬",
        (PieceType::Cannon, Side::Red) => "炮",
        (PieceType::Cannon, Side::Black) => "砲",
        (PieceType::Pawn, Side::Red) => "兵",
        (PieceType::Pawn, Side::Black) => "卒",
    }
}

/// 处理棋子按钮点击
pub fn handle_piece_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &PieceButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_state: ResMut<BoardEditorState>,
) {
    for (interaction, mut color, piece_button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                editor_state.select_piece(piece_button.0);
            }
            Interaction::Hovered => {
                *color = Color::srgb(1.0, 0.95, 0.8).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.9, 0.85, 0.7).into();
            }
        }
    }
}

/// 处理工具按钮点击
pub fn handle_tool_buttons(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &ToolAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_state: ResMut<BoardEditorState>,
) {
    for (interaction, mut color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                match action {
                    ToolAction::Clear => {
                        editor_state.clear();
                    }
                    ToolAction::Initial => {
                        editor_state.set_initial();
                    }
                    ToolAction::KingsOnly => {
                        editor_state.set_kings_only();
                    }
                    ToolAction::Eraser => {
                        editor_state.deselect();
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

/// 更新选中棋子指示器
pub fn update_selected_indicator(
    editor_state: Res<BoardEditorState>,
    mut query: Query<&mut Text, With<SelectedPieceIndicator>>,
) {
    if !editor_state.is_changed() {
        return;
    }

    for mut text in &mut query {
        **text = if let Some(piece) = editor_state.selected_piece {
            format!("当前：{}", piece_to_chinese(piece))
        } else {
            "当前：无".to_string()
        };
    }
}

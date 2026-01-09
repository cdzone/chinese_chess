//! 棋子渲染

use bevy::prelude::*;
use protocol::{Board, Piece, Side};

use super::{BoardLayout, BoardMarker};
use crate::theme::ColorTheme;

/// 棋子标记组件
#[derive(Component)]
pub struct PieceMarker;

/// 棋子位置组件
#[derive(Component)]
pub struct PiecePosition {
    pub x: u8,
    pub y: u8,
}

/// 生成所有棋子
pub fn spawn_pieces(
    commands: &mut Commands,
    board: &Board,
    layout: &BoardLayout,
    theme: &ColorTheme,
    asset_server: &AssetServer,
) {
    for (pos, piece) in board.all_pieces() {
        spawn_piece(commands, pos.x, pos.y, &piece, layout, theme, asset_server);
    }
}

/// 生成单个棋子
fn spawn_piece(
    commands: &mut Commands,
    x: u8,
    y: u8,
    piece: &Piece,
    layout: &BoardLayout,
    theme: &ColorTheme,
    asset_server: &AssetServer,
) {
    let screen_pos = layout.board_to_screen(x, y);
    let radius = layout.piece_radius;

    // 棋子底色和文字颜色
    let text_color = match piece.side {
        Side::Red => theme.red_piece_text,
        Side::Black => theme.black_piece_text,
    };

    // 棋子汉字
    let char = piece.display_char();

    // 创建棋子实体
    commands
        .spawn((
            // 棋子背景圆形
            Sprite {
                color: theme.piece_background,
                custom_size: Some(Vec2::splat(radius * 2.0)),
                ..default()
            },
            Transform::from_xyz(screen_pos.x, screen_pos.y, 10.0),
            PieceMarker,
            PiecePosition { x, y },
            BoardMarker,
        ))
        .with_children(|parent| {
            // 棋子边框
            parent.spawn((
                Sprite {
                    color: theme.piece_border,
                    custom_size: Some(Vec2::splat(radius * 2.0 + 4.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, -0.1),
            ));

            // 棋子文字
            parent.spawn((
                Text2d::new(char.to_string()),
                TextFont {
                    font: asset_server.load("fonts/SourceHanSansSC-Bold.otf"),
                    font_size: radius * 1.4,
                    ..default()
                },
                TextColor(text_color),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });
}

/// 棋子移动动画组件
#[derive(Component)]
pub struct PieceMoveAnimation {
    pub start: Vec2,
    pub end: Vec2,
    pub duration: f32,
    pub elapsed: f32,
}

impl PieceMoveAnimation {
    pub fn new(start: Vec2, end: Vec2, duration: f32) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: 0.0,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).min(1.0)
    }

    pub fn current_position(&self) -> Vec2 {
        // 使用 ease-out 缓动
        let t = self.progress();
        let eased = 1.0 - (1.0 - t).powi(3);
        self.start.lerp(self.end, eased)
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

/// 更新棋子移动动画
pub fn animate_pieces(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut PieceMoveAnimation)>,
) {
    for (entity, mut transform, mut animation) in query.iter_mut() {
        animation.elapsed += time.delta_secs();
        let pos = animation.current_position();
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;

        if animation.is_finished() {
            commands.entity(entity).remove::<PieceMoveAnimation>();
        }
    }
}

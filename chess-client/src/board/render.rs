//! 棋盘渲染

use bevy::prelude::*;
use protocol::Position;

use super::{BoardLayout, BoardMarker, HighlightMarker, HighlightType};
use crate::theme::ColorTheme;

/// 生成棋盘
pub fn spawn_board(commands: &mut Commands, layout: &BoardLayout, theme: &ColorTheme) {
    // 棋盘背景
    commands.spawn((
        Sprite {
            color: theme.board_background,
            custom_size: Some(Vec2::new(layout.width + 40.0, layout.height + 40.0)),
            ..default()
        },
        Transform::from_xyz(
            layout.origin.x + layout.width / 2.0,
            layout.origin.y + layout.height / 2.0,
            0.0,
        ),
        BoardMarker,
    ));

    // 绘制棋盘线条
    spawn_board_lines(commands, layout, theme);

    // 绘制九宫格斜线
    spawn_palace_diagonals(commands, layout, theme);

    // 绘制兵/卒位置标记
    spawn_position_markers(commands, layout, theme);
}

/// 绘制棋盘线条
fn spawn_board_lines(commands: &mut Commands, layout: &BoardLayout, theme: &ColorTheme) {
    let line_width = 2.0;

    // 横线 (10条)
    for y in 0..10 {
        let start = layout.board_to_screen(0, y);
        let end = layout.board_to_screen(8, y);
        spawn_line(commands, start, end, line_width, theme.board_lines);
    }

    // 竖线 - 上半部分 (9条，但边线贯穿)
    for x in 0..9 {
        if x == 0 || x == 8 {
            // 边线贯穿整个棋盘
            let start = layout.board_to_screen(x, 0);
            let end = layout.board_to_screen(x, 9);
            spawn_line(commands, start, end, line_width, theme.board_lines);
        } else {
            // 中间线分为上下两段（楚河汉界）
            let start1 = layout.board_to_screen(x, 0);
            let end1 = layout.board_to_screen(x, 4);
            spawn_line(commands, start1, end1, line_width, theme.board_lines);

            let start2 = layout.board_to_screen(x, 5);
            let end2 = layout.board_to_screen(x, 9);
            spawn_line(commands, start2, end2, line_width, theme.board_lines);
        }
    }
}

/// 绘制九宫格斜线
fn spawn_palace_diagonals(commands: &mut Commands, layout: &BoardLayout, theme: &ColorTheme) {
    let line_width = 2.0;

    // 红方九宫 (x: 3-5, y: 0-2)
    let red_palace = [
        (layout.board_to_screen(3, 0), layout.board_to_screen(5, 2)),
        (layout.board_to_screen(5, 0), layout.board_to_screen(3, 2)),
    ];

    for (start, end) in red_palace {
        spawn_line(commands, start, end, line_width, theme.board_lines);
    }

    // 黑方九宫 (x: 3-5, y: 7-9)
    let black_palace = [
        (layout.board_to_screen(3, 7), layout.board_to_screen(5, 9)),
        (layout.board_to_screen(5, 7), layout.board_to_screen(3, 9)),
    ];

    for (start, end) in black_palace {
        spawn_line(commands, start, end, line_width, theme.board_lines);
    }
}

/// 绘制兵/卒和炮位置的标记
fn spawn_position_markers(commands: &mut Commands, layout: &BoardLayout, theme: &ColorTheme) {
    let marker_size = layout.cell_size * 0.12;
    let offset = layout.cell_size * 0.15;

    // 炮位置
    let cannon_positions = [(1, 2), (7, 2), (1, 7), (7, 7)];

    // 兵/卒位置（不包括边上的）
    let pawn_positions = [
        (2, 3),
        (4, 3),
        (6, 3), // 红兵
        (2, 6),
        (4, 6),
        (6, 6), // 黑卒
    ];

    // 边上的兵/卒位置（只有一侧标记）
    let edge_pawn_positions = [(0, 3), (8, 3), (0, 6), (8, 6)];

    // 绘制完整标记（四角）
    for (x, y) in cannon_positions.iter().chain(pawn_positions.iter()) {
        let center = layout.board_to_screen(*x, *y);
        spawn_position_marker(commands, center, marker_size, offset, theme.board_lines, false);
    }

    // 绘制边缘标记（只有内侧）
    for (x, y) in edge_pawn_positions {
        let center = layout.board_to_screen(x, y);
        let is_left_edge = x == 0;
        spawn_edge_position_marker(
            commands,
            center,
            marker_size,
            offset,
            theme.board_lines,
            is_left_edge,
        );
    }
}

/// 绘制位置标记（四角的小折线）
fn spawn_position_marker(
    commands: &mut Commands,
    center: Vec2,
    size: f32,
    offset: f32,
    color: Color,
    _is_edge: bool,
) {
    let line_width = 1.5;

    // 四个角的标记
    let corners = [
        (Vec2::new(-offset, offset), Vec2::new(-1.0, 0.0), Vec2::new(0.0, 1.0)),
        (Vec2::new(offset, offset), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)),
        (Vec2::new(-offset, -offset), Vec2::new(-1.0, 0.0), Vec2::new(0.0, -1.0)),
        (Vec2::new(offset, -offset), Vec2::new(1.0, 0.0), Vec2::new(0.0, -1.0)),
    ];

    for (corner_offset, h_dir, v_dir) in corners {
        let corner = center + corner_offset;
        // 水平线
        spawn_line(
            commands,
            corner,
            corner + h_dir * size,
            line_width,
            color,
        );
        // 垂直线
        spawn_line(
            commands,
            corner,
            corner + v_dir * size,
            line_width,
            color,
        );
    }
}

/// 绘制边缘位置标记（只有内侧两个角）
fn spawn_edge_position_marker(
    commands: &mut Commands,
    center: Vec2,
    size: f32,
    offset: f32,
    color: Color,
    is_left_edge: bool,
) {
    let line_width = 1.5;

    // 只绘制内侧的两个角
    let corners = if is_left_edge {
        // 左边缘：只绘制右侧的两个角
        vec![
            (Vec2::new(offset, offset), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0)),
            (Vec2::new(offset, -offset), Vec2::new(1.0, 0.0), Vec2::new(0.0, -1.0)),
        ]
    } else {
        // 右边缘：只绘制左侧的两个角
        vec![
            (Vec2::new(-offset, offset), Vec2::new(-1.0, 0.0), Vec2::new(0.0, 1.0)),
            (Vec2::new(-offset, -offset), Vec2::new(-1.0, 0.0), Vec2::new(0.0, -1.0)),
        ]
    };

    for (corner_offset, h_dir, v_dir) in corners {
        let corner = center + corner_offset;
        spawn_line(commands, corner, corner + h_dir * size, line_width, color);
        spawn_line(commands, corner, corner + v_dir * size, line_width, color);
    }
}

/// 绘制线条
fn spawn_line(commands: &mut Commands, start: Vec2, end: Vec2, width: f32, color: Color) {
    let diff = end - start;
    let length = diff.length();
    let angle = diff.y.atan2(diff.x);
    let center = (start + end) / 2.0;

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(length, width)),
            ..default()
        },
        Transform::from_xyz(center.x, center.y, 1.0)
            .with_rotation(Quat::from_rotation_z(angle)),
        BoardMarker,
    ));
}

/// 生成高亮
pub fn spawn_highlight(
    commands: &mut Commands,
    layout: &BoardLayout,
    pos: Position,
    color: Color,
    highlight_type: HighlightType,
) {
    let screen_pos = layout.board_to_screen(pos.x, pos.y);
    let size = match highlight_type {
        HighlightType::Selected => layout.piece_radius * 2.2,
        HighlightType::ValidMove => layout.piece_radius * 0.5,
        HighlightType::LastMove => layout.piece_radius * 2.0,
    };

    let z = match highlight_type {
        HighlightType::Selected => 5.0,
        HighlightType::ValidMove => 6.0,
        HighlightType::LastMove => 4.0,
    };

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        Transform::from_xyz(screen_pos.x, screen_pos.y, z),
        HighlightMarker,
        BoardMarker,
    ));
}

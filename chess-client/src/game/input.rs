//! 输入处理

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{ClientGame, GameEvent};
use crate::board::BoardLayout;

/// 处理鼠标输入
pub fn handle_mouse_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    layout: Res<BoardLayout>,
    game: Res<ClientGame>,
    mut events: EventWriter<GameEvent>,
) {
    // 只处理左键点击
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    // 不是玩家回合时不处理
    if !game.is_my_turn() {
        return;
    }

    // 获取鼠标位置
    let window = windows.single();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    // 转换为世界坐标
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) else {
        return;
    };

    // 转换为棋盘坐标
    let Some((x, y)) = layout.screen_to_board(world_position) else {
        return;
    };

    // 检查是否点击了合法落点
    let Some(clicked_pos) = protocol::Position::new(x, y) else {
        return;
    };

    if game.valid_moves.contains(&clicked_pos) {
        if let Some(from) = game.selected_piece {
            events.send(GameEvent::MovePiece {
                from_x: from.x,
                from_y: from.y,
                to_x: x,
                to_y: y,
            });
            return;
        }
    }

    // 检查是否点击了棋子
    if let Some(state) = &game.game_state {
        if let Some(piece) = state.board.get(clicked_pos) {
            if Some(piece.side) == game.player_side {
                events.send(GameEvent::SelectPiece { x, y });
                return;
            }
        }
    }

    // 点击空白处取消选择
    if game.selected_piece.is_some() {
        events.send(GameEvent::Deselect);
    }
}

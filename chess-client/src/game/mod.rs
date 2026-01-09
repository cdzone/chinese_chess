//! 游戏逻辑模块
//!
//! 管理游戏状态和交互

mod input;
mod state;

pub use input::*;
pub use state::*;

use bevy::prelude::*;

use crate::board::pieces::animate_pieces;
use crate::GameState;

/// 游戏插件
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClientGame::default())
            .add_event::<GameEvent>()
            .add_systems(
                Update,
                (
                    handle_mouse_input,
                    handle_game_events,
                    animate_pieces,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// 游戏事件
#[derive(Event, Clone, Debug)]
pub enum GameEvent {
    /// 选择棋子
    SelectPiece { x: u8, y: u8 },
    /// 移动棋子
    MovePiece { from_x: u8, from_y: u8, to_x: u8, to_y: u8 },
    /// 取消选择
    Deselect,
    /// 请求悔棋
    RequestUndo,
    /// 认输
    Resign,
    /// 暂停游戏
    PauseGame,
    /// 继续游戏
    ResumeGame,
}

/// 处理游戏事件
fn handle_game_events(
    mut events: EventReader<GameEvent>,
    mut game: ResMut<ClientGame>,
    mut network_events: EventWriter<crate::network::NetworkEvent>,
) {
    for event in events.read() {
        match event {
            GameEvent::SelectPiece { x, y } => {
                game.select_piece(*x, *y);
            }
            GameEvent::MovePiece { from_x, from_y, to_x, to_y } => {
                // 发送网络消息
                if let (Some(from), Some(to)) = (
                    protocol::Position::new(*from_x, *from_y),
                    protocol::Position::new(*to_x, *to_y),
                ) {
                    network_events.send(crate::network::NetworkEvent::SendMove { from, to });
                }
                // 清除选择
                game.clear_selection();
            }
            GameEvent::Deselect => {
                game.clear_selection();
            }
            GameEvent::RequestUndo => {
                network_events.send(crate::network::NetworkEvent::SendUndo);
            }
            GameEvent::Resign => {
                network_events.send(crate::network::NetworkEvent::SendResign);
            }
            GameEvent::PauseGame => {
                network_events.send(crate::network::NetworkEvent::SendPause);
            }
            GameEvent::ResumeGame => {
                network_events.send(crate::network::NetworkEvent::SendResume);
            }
        }
    }
}

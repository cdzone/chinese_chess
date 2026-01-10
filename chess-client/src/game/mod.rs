//! 游戏逻辑模块
//!
//! 管理游戏状态和交互

mod ai;
mod input;
mod state;

pub use ai::*;
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
            .add_message::<GameEvent>()
            .add_plugins(LocalAiPlugin)
            .add_systems(
                Update,
                (
                    update_local_timer,
                    handle_mouse_input,
                    handle_game_events,
                    animate_pieces,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// 游戏事件
#[derive(Message, Clone, Debug)]
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
    mut events: MessageReader<GameEvent>,
    mut game: ResMut<ClientGame>,
    mut network_events: MessageWriter<crate::network::NetworkEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        match event {
            GameEvent::SelectPiece { x, y } => {
                game.select_piece(*x, *y);
            }
            GameEvent::MovePiece { from_x, from_y, to_x, to_y } => {
                if let (Some(from), Some(to)) = (
                    protocol::Position::new(*from_x, *from_y),
                    protocol::Position::new(*to_x, *to_y),
                ) {
                    if game.is_local() {
                        // 本地模式：直接执行走棋
                        execute_local_move(&mut game, from, to);
                    } else {
                        // 在线模式：发送网络消息
                        network_events.write(crate::network::NetworkEvent::SendMove { from, to });
                    }
                }
                // 清除选择
                game.clear_selection();
            }
            GameEvent::Deselect => {
                game.clear_selection();
            }
            GameEvent::RequestUndo => {
                if game.is_local() {
                    // 本地模式：直接悔棋（撤销 2 步：玩家 + AI）
                    if game.local_undo() {
                        tracing::info!("本地悔棋成功");
                    } else {
                        tracing::warn!("无法悔棋：没有可撤销的走法");
                    }
                } else {
                    network_events.write(crate::network::NetworkEvent::SendUndo);
                }
            }
            GameEvent::Resign => {
                if game.is_local() {
                    // 本地模式：直接认输
                    let result = protocol::GameResult::BlackWin(protocol::WinReason::Resign);
                    game.set_result(result);
                    game_state.set(GameState::GameOver);
                    tracing::info!("玩家认输");
                } else {
                    network_events.write(crate::network::NetworkEvent::SendResign);
                }
            }
            GameEvent::PauseGame => {
                if game.is_local() {
                    game.is_paused = true;
                    tracing::info!("游戏已暂停");
                } else {
                    network_events.write(crate::network::NetworkEvent::SendPause);
                }
            }
            GameEvent::ResumeGame => {
                if game.is_local() {
                    game.is_paused = false;
                    tracing::info!("游戏已继续");
                } else {
                    network_events.write(crate::network::NetworkEvent::SendResume);
                }
            }
        }
    }
}

/// 执行本地走棋
fn execute_local_move(game: &mut ResMut<ClientGame>, from: protocol::Position, to: protocol::Position) {
    let state = match &game.game_state {
        Some(s) => s,
        None => return,
    };

    let mv = protocol::Move::new(from, to);

    // 验证走法合法性
    let legal_moves = protocol::MoveGenerator::generate_legal(state);
    if !legal_moves.iter().any(|m| m.from == mv.from && m.to == mv.to) {
        tracing::warn!("非法走法: {:?}", mv);
        return;
    }

    // 执行走法
    let mut new_state = state.clone();
    new_state.board.move_piece(mv.from, mv.to);
    new_state.current_turn = new_state.current_turn.opponent();

    // 生成中文纵线表示法
    let notation = protocol::Notation::to_chinese(&state.board, &mv)
        .unwrap_or_else(|| format!("{:?}->{:?}", mv.from, mv.to));

    // 更新游戏状态
    game.update_state(new_state, mv.from, mv.to, notation);
}

/// P1: 本地计时器系统（每帧更新）
fn update_local_timer(
    mut game: ResMut<ClientGame>,
    time: Res<Time>,
    mut game_state: ResMut<NextState<GameState>>,
    settings: Res<crate::settings::GameSettings>,
) {
    // 只在本地模式、非暂停、游戏进行中时更新
    if !game.is_local() || game.is_paused || game.game_result.is_some() {
        return;
    }

    // 如果时间限制是无限制，不更新计时器
    if settings.time_limit == crate::settings::TimeLimit::Unlimited {
        return;
    }

    let Some(state) = &game.game_state else { return };

    let delta_ms = time.delta().as_millis() as u64;

    match state.current_turn {
        protocol::Side::Red => {
            game.red_time_ms = game.red_time_ms.saturating_sub(delta_ms);
            if game.red_time_ms == 0 {
                // 红方超时，黑方获胜
                let result = protocol::GameResult::BlackWin(protocol::WinReason::Timeout);
                game.set_result(result);
                game_state.set(GameState::GameOver);
                tracing::info!("红方超时，黑方获胜");
            }
        }
        protocol::Side::Black => {
            game.black_time_ms = game.black_time_ms.saturating_sub(delta_ms);
            if game.black_time_ms == 0 {
                // 黑方超时，红方获胜
                let result = protocol::GameResult::RedWin(protocol::WinReason::Timeout);
                game.set_result(result);
                game_state.set(GameState::GameOver);
                tracing::info!("黑方超时，红方获胜");
            }
        }
    }
}

//! 服务器主逻辑

use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use chess_ai::AiEngine;
use protocol::{
    ClientMessage, ErrorCode, GameResult, Move, Notation, PlayerId,
    Position, RoomId, RoomInfo, RoomState, RoomType, ServerMessage, Side, WinReason,
};

use crate::player::{PlayerManager, PlayerStatus};
use crate::room::RoomManager;
use crate::storage::StorageManager;

/// 断线超时时间（秒）
const DISCONNECT_TIMEOUT_SECS: u64 = 60;

/// 服务器状态
pub struct ServerState {
    pub players: PlayerManager,
    pub rooms: RoomManager,
    pub storage: StorageManager,
    /// 玩家 ID -> 消息发送通道
    pub connections: HashMap<PlayerId, mpsc::Sender<ServerMessage>>,
    /// 断线玩家的超时时间
    pub disconnect_timeouts: HashMap<PlayerId, Instant>,
}

impl ServerState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            players: PlayerManager::new(),
            rooms: RoomManager::new(),
            storage: StorageManager::new()?,
            connections: HashMap::new(),
            disconnect_timeouts: HashMap::new(),
        })
    }

    /// 发送消息给玩家
    pub async fn send_to_player(&self, player_id: PlayerId, msg: ServerMessage) {
        if let Some(tx) = self.connections.get(&player_id) {
            let _ = tx.send(msg).await;
        }
    }

    /// 广播消息给房间内所有玩家
    pub async fn broadcast_to_room(&self, room_id: RoomId, msg: ServerMessage) {
        if let Some(room) = self.rooms.get(room_id) {
            if let Some(red_id) = room.red_player {
                self.send_to_player(red_id, msg.clone()).await;
            }
            if let Some(black_id) = room.black_player {
                self.send_to_player(black_id, msg).await;
            }
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new().expect("无法初始化服务器状态")
    }
}

/// 待发送的消息
struct PendingMessages {
    messages: Vec<(PlayerId, ServerMessage)>,
    broadcasts: Vec<(RoomId, ServerMessage)>,
}

impl PendingMessages {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            broadcasts: Vec::new(),
        }
    }

    fn send(&mut self, player_id: PlayerId, msg: ServerMessage) {
        self.messages.push((player_id, msg));
    }

    fn broadcast(&mut self, room_id: RoomId, msg: ServerMessage) {
        self.broadcasts.push((room_id, msg));
    }

    async fn flush(self, state: &ServerState) {
        for (player_id, msg) in self.messages {
            state.send_to_player(player_id, msg).await;
        }
        for (room_id, msg) in self.broadcasts {
            state.broadcast_to_room(room_id, msg).await;
        }
    }
}

/// 消息处理器
pub struct MessageHandler;

impl MessageHandler {
    /// 处理客户端消息
    pub async fn handle(
        state: &mut ServerState,
        player_id: PlayerId,
        msg: ClientMessage,
    ) -> Option<ServerMessage> {
        let mut pending = PendingMessages::new();
        
        let result = match msg {
            ClientMessage::Login { nickname } => {
                Self::handle_login(state, nickname)
            }
            ClientMessage::Reconnect { player_id: pid, room_id } => {
                Self::handle_reconnect(state, &mut pending, pid, room_id)
            }
            ClientMessage::CreateRoom { room_type, preferred_side } => {
                Self::handle_create_room(state, player_id, room_type, preferred_side)
            }
            ClientMessage::JoinRoom { room_id } => {
                Self::handle_join_room(state, &mut pending, player_id, room_id)
            }
            ClientMessage::LeaveRoom => {
                Self::handle_leave_room(state, &mut pending, player_id)
            }
            ClientMessage::ListRooms => {
                Self::handle_list_rooms(state)
            }
            ClientMessage::MakeMove { from, to } => {
                Self::handle_make_move(state, &mut pending, player_id, from, to)
            }
            ClientMessage::RequestUndo => {
                Self::handle_request_undo(state, &mut pending, player_id)
            }
            ClientMessage::RespondUndo { accept } => {
                Self::handle_respond_undo(state, &mut pending, player_id, accept)
            }
            ClientMessage::Resign => {
                Self::handle_resign(state, &mut pending, player_id)
            }
            ClientMessage::PauseGame => {
                Self::handle_pause(state, player_id)
            }
            ClientMessage::ResumeGame => {
                Self::handle_resume(state, player_id)
            }
            ClientMessage::SaveGame => {
                Self::handle_save_game(state, player_id)
            }
            ClientMessage::LoadGame { game_id } => {
                Self::handle_load_game(state, &mut pending, player_id, game_id)
            }
            ClientMessage::Ping => Some(ServerMessage::Pong),
        };

        // 发送待发送的消息
        pending.flush(state).await;
        
        result
    }

    /// 处理登录
    fn handle_login(state: &mut ServerState, nickname: String) -> Option<ServerMessage> {
        match state.players.login(nickname) {
            Ok(player_id) => Some(ServerMessage::LoginSuccess { player_id }),
            Err(msg) => Some(ServerMessage::Error {
                code: if msg.contains("占用") {
                    ErrorCode::NicknameOccupied
                } else {
                    ErrorCode::InvalidNickname
                },
                message: msg.to_string(),
            }),
        }
    }

    /// 处理重连
    fn handle_reconnect(
        state: &mut ServerState,
        pending: &mut PendingMessages,
        player_id: PlayerId,
        room_id: RoomId,
    ) -> Option<ServerMessage> {
        // 检查玩家是否存在
        if !state.players.exists(player_id) {
            return Some(ServerMessage::Error {
                code: ErrorCode::PlayerNotFound,
                message: "玩家不存在".to_string(),
            });
        }

        // 检查房间是否存在
        let room = state.rooms.get(room_id)?;
        if !room.has_player(player_id) {
            return Some(ServerMessage::Error {
                code: ErrorCode::NotInRoom,
                message: "不在该房间中".to_string(),
            });
        }

        // 获取房间信息
        let your_side = room.get_player_side(player_id)?;
        let game_state = room.game_state.clone()?;
        let (red_time_ms, black_time_ms) = if let Some(timer) = &room.timer {
            (timer.red_time_ms(), timer.black_time_ms())
        } else {
            (0, 0)
        };
        let opponent_id = room.get_opponent_id(player_id);

        // 恢复玩家状态
        state.players.reconnect(player_id);
        state.disconnect_timeouts.remove(&player_id);

        // 如果是当前走棋方重连，重置计时器开始时间
        let room = state.rooms.get_mut(room_id)?;
        if let Some(timer) = &mut room.timer {
            if timer.current_turn() == your_side && !timer.is_paused() {
                // 重连时重置当前回合开始时间，避免断线期间时间被计入
                timer.reset_turn_start();
            }
        }

        // 通知对手
        if let Some(opponent_id) = opponent_id {
            pending.send(opponent_id, ServerMessage::OpponentReconnected);
        }

        Some(ServerMessage::ReconnectSuccess {
            room_id,
            game_state,
            your_side,
            red_time_ms,
            black_time_ms,
        })
    }

    /// 处理创建房间
    fn handle_create_room(
        state: &mut ServerState,
        player_id: PlayerId,
        room_type: RoomType,
        preferred_side: Option<Side>,
    ) -> Option<ServerMessage> {
        // 检查玩家是否已在房间中
        if let Some(player) = state.players.get(player_id) {
            if matches!(player.status, PlayerStatus::InRoom(_)) {
                return Some(ServerMessage::Error {
                    code: ErrorCode::AlreadyInRoom,
                    message: "已在房间中".to_string(),
                });
            }
        }

        // 创建房间
        let room_id = state.rooms.create(room_type);
        let room = state.rooms.get_mut(room_id)?;

        // 玩家加入房间
        let your_side = room.add_player(player_id, preferred_side)?;
        state.players.set_status(player_id, PlayerStatus::InRoom(room_id));

        // 如果是 PvE，AI 自动加入并开始游戏
        if let RoomType::PvE(difficulty) = room_type {
            // AI 作为对方（玩家默认红方）
            // 使用专用的 AI_PLAYER_ID 避免与真实玩家 ID 冲突
            if your_side == Side::Red {
                room.black_player = Some(protocol::AI_PLAYER_ID);
            } else {
                room.red_player = Some(protocol::AI_PLAYER_ID);
            }
            room.start_game();

            // 返回游戏开始消息
            let game_state = room.game_state.clone()?;
            let red_player = if your_side == Side::Red {
                state.players.get_nickname(player_id).unwrap_or("玩家").to_string()
            } else {
                format!("AI ({:?})", difficulty)
            };
            let black_player = if your_side == Side::Black {
                state.players.get_nickname(player_id).unwrap_or("玩家").to_string()
            } else {
                format!("AI ({:?})", difficulty)
            };

            return Some(ServerMessage::GameStarted {
                initial_state: game_state,
                your_side,
                red_player,
                black_player,
            });
        }

        Some(ServerMessage::RoomCreated { room_id, your_side })
    }

    /// 处理加入房间
    fn handle_join_room(
        state: &mut ServerState,
        pending: &mut PendingMessages,
        player_id: PlayerId,
        room_id: RoomId,
    ) -> Option<ServerMessage> {
        // 检查玩家是否已在房间中
        if let Some(player) = state.players.get(player_id) {
            if matches!(player.status, PlayerStatus::InRoom(_)) {
                return Some(ServerMessage::Error {
                    code: ErrorCode::AlreadyInRoom,
                    message: "已在房间中".to_string(),
                });
            }
        }

        // 检查房间是否存在
        let room = match state.rooms.get(room_id) {
            Some(r) => r,
            None => {
                return Some(ServerMessage::Error {
                    code: ErrorCode::RoomNotFound,
                    message: "房间不存在".to_string(),
                });
            }
        };

        // 检查房间状态
        if room.state != RoomState::Waiting {
            return Some(ServerMessage::Error {
                code: ErrorCode::RoomClosed,
                message: "房间不可加入".to_string(),
            });
        }

        // 检查房间是否已满
        if room.is_full() {
            return Some(ServerMessage::Error {
                code: ErrorCode::RoomFull,
                message: "房间已满".to_string(),
            });
        }

        // 获取房主 ID
        let opponent_id = room.red_player.or(room.black_player);

        // 获取加入者昵称
        let joiner_nickname = state.players.get_nickname(player_id).unwrap_or("玩家").to_string();

        // 加入房间
        let room = state.rooms.get_mut(room_id)?;
        let side = room.add_player(player_id, None)?;
        state.players.set_status(player_id, PlayerStatus::InRoom(room_id));

        // 通知房主有人加入
        if let Some(opponent_id) = opponent_id {
            pending.send(opponent_id, ServerMessage::OpponentJoined { nickname: joiner_nickname });
        }

        // 如果房间满了，开始游戏
        if room.is_full() {
            room.start_game();

            let game_state = room.game_state.clone()?;
            let red_id = room.red_player?;
            let black_id = room.black_player?;
            let red_player = state.players.get_nickname(red_id).unwrap_or("玩家").to_string();
            let black_player = state.players.get_nickname(black_id).unwrap_or("玩家").to_string();

            // 通知双方游戏开始
            pending.send(
                red_id,
                ServerMessage::GameStarted {
                    initial_state: game_state.clone(),
                    your_side: Side::Red,
                    red_player: red_player.clone(),
                    black_player: black_player.clone(),
                },
            );

            pending.send(
                black_id,
                ServerMessage::GameStarted {
                    initial_state: game_state,
                    your_side: Side::Black,
                    red_player,
                    black_player,
                },
            );
        }

        Some(ServerMessage::RoomJoined { room_id, side })
    }

    /// 处理离开房间
    fn handle_leave_room(
        state: &mut ServerState,
        pending: &mut PendingMessages,
        player_id: PlayerId,
    ) -> Option<ServerMessage> {
        // 查找玩家所在房间
        let room_id = state.rooms.find_player_room(player_id)?;
        let room = state.rooms.get(room_id)?;

        let side = room.get_player_side(player_id)?;
        let opponent_id = room.get_opponent_id(player_id);
        let is_playing = room.state == RoomState::Playing;

        // 如果游戏进行中，离开者判负
        if is_playing {
            let result = match side {
                Side::Red => GameResult::BlackWin(WinReason::Disconnect),
                Side::Black => GameResult::RedWin(WinReason::Disconnect),
            };

            // 通知对手胜利
            if let Some(opponent_id) = opponent_id {
                pending.send(opponent_id, ServerMessage::GameOver { result: result.clone() });
            }

            // 更新房间状态
            let room = state.rooms.get_mut(room_id)?;
            room.finish(result);
        }

        // 移除玩家
        let room = state.rooms.get_mut(room_id)?;
        room.remove_player(player_id);
        state.players.set_status(player_id, PlayerStatus::Online);

        // 如果房间空了，销毁房间
        if room.red_player.is_none() && room.black_player.is_none() {
            state.rooms.remove(room_id);
        }

        None
    }

    /// 处理房间列表
    fn handle_list_rooms(state: &ServerState) -> Option<ServerMessage> {
        let rooms: Vec<RoomInfo> = state.rooms.list_joinable()
            .iter()
            .map(|r| {
                let red_name = r.red_player.and_then(|id| state.players.get_nickname(id).map(|s| s.to_string()));
                let black_name = r.black_player.and_then(|id| state.players.get_nickname(id).map(|s| s.to_string()));
                r.info(red_name, black_name)
            })
            .collect();

        Some(ServerMessage::RoomList { rooms })
    }

    /// 处理走棋
    fn handle_make_move(
        state: &mut ServerState,
        pending: &mut PendingMessages,
        player_id: PlayerId,
        from: Position,
        to: Position,
    ) -> Option<ServerMessage> {
        // 查找玩家所在房间
        let room_id = state.rooms.find_player_room(player_id)?;
        let room = state.rooms.get(room_id)?;

        // 检查游戏状态
        if room.state != RoomState::Playing {
            return Some(ServerMessage::Error {
                code: ErrorCode::GameNotStarted,
                message: "游戏未开始".to_string(),
            });
        }

        // 检查是否轮到该玩家
        let player_side = room.get_player_side(player_id)?;
        let game_state = room.game_state.as_ref()?;
        if game_state.current_turn != player_side {
            return Some(ServerMessage::Error {
                code: ErrorCode::NotYourTurn,
                message: "不是你的回合".to_string(),
            });
        }

        // 获取 PvE 难度（如果是 PvE 模式）
        let pve_difficulty = match room.room_type {
            RoomType::PvE(difficulty) => Some(difficulty),
            _ => None,
        };

        // 执行走棋
        let room = state.rooms.get_mut(room_id)?;
        let mv = Move::new(from, to);
        if let Err(msg) = room.make_move(mv) {
            return Some(ServerMessage::Error {
                code: ErrorCode::InvalidMove,
                message: msg.to_string(),
            });
        }

        // 生成中文记谱
        let new_state = room.game_state.clone()?;
        let last_move = room.move_history.last()?.clone();
        let notation = Notation::to_chinese(&new_state.board, &last_move).unwrap_or_default();

        // 获取时间信息
        let (red_time_ms, black_time_ms) = if let Some(timer) = &room.timer {
            (timer.red_time_ms(), timer.black_time_ms())
        } else {
            (0, 0)
        };

        // 检查游戏是否结束
        let game_over = room.check_game_over();

        // 广播走棋消息
        pending.broadcast(room_id, ServerMessage::MoveMade {
            from,
            to,
            new_state: new_state.clone(),
            notation,
        });

        // 发送时间更新
        pending.broadcast(room_id, ServerMessage::TimeUpdate {
            red_time_ms,
            black_time_ms,
        });

        // 处理游戏结束
        if let Some(result) = game_over {
            let room = state.rooms.get_mut(room_id)?;
            room.finish(result.clone());
            pending.broadcast(room_id, ServerMessage::GameOver { result });
            return None;
        }

        // PvE 模式：AI 自动走棋（在独立线程中执行，不阻塞其他异步任务）
        if let Some(difficulty) = pve_difficulty {
            // 获取当前状态版本和游戏状态
            let room = state.rooms.get(room_id)?;
            let version_before = room.version;
            let game_state_for_ai = room.game_state.clone()?;
            
            // 在阻塞线程池中运行 AI 计算
            // 使用 block_in_place 允许 tokio 在等待期间处理其他任务
            let ai_result = tokio::task::block_in_place(|| {
                let mut engine = AiEngine::from_difficulty(difficulty);
                engine.search(&game_state_for_ai)
            });
            
            // 处理 AI 结果
            match ai_result {
                Some(ai_move) => {
                    Self::apply_ai_move(state, pending, room_id, ai_move, version_before);
                }
                None => {
                    // AI 无法走棋，判定 AI 负
                    tracing::warn!("AI 无法找到合法走法，判定 AI 负");
                    Self::ai_loses(state, pending, room_id);
                }
            }
        }

        None
    }

    /// 应用 AI 走法（带版本检查）
    fn apply_ai_move(
        state: &mut ServerState,
        pending: &mut PendingMessages,
        room_id: RoomId,
        ai_move: Move,
        version_before: u64,
    ) {
        let room = match state.rooms.get(room_id) {
            Some(r) => r,
            None => return,
        };

        // 检查版本号，确保状态未改变
        if room.version != version_before {
            tracing::warn!("AI 计算期间游戏状态已改变，丢弃 AI 走法");
            return;
        }

        // 检查游戏状态
        if room.state != RoomState::Playing {
            return;
        }

        // 执行 AI 走棋
        let room = match state.rooms.get_mut(room_id) {
            Some(r) => r,
            None => return,
        };

        if room.make_move(ai_move).is_err() {
            tracing::error!("AI 走棋失败: {:?}，判定 AI 负", ai_move);
            // AI 走棋失败，判定 AI 负
            let result = GameResult::RedWin(WinReason::Resign);
            room.finish(result.clone());
            pending.broadcast(room_id, ServerMessage::GameOver { result });
            return;
        }

        // 重置计时器起点（AI 思考时间不应计入玩家时间）
        if let Some(timer) = &mut room.timer {
            timer.reset_turn_start();
        }

        // 生成中文记谱
        let new_state = match room.game_state.clone() {
            Some(s) => s,
            None => return,
        };
        let notation = Notation::to_chinese(&new_state.board, &ai_move).unwrap_or_default();

        // 获取时间信息
        let (red_time_ms, black_time_ms) = if let Some(timer) = &room.timer {
            (timer.red_time_ms(), timer.black_time_ms())
        } else {
            (0, 0)
        };

        // 检查游戏是否结束
        let game_over = room.check_game_over();

        // 广播 AI 走棋消息
        pending.broadcast(room_id, ServerMessage::MoveMade {
            from: ai_move.from,
            to: ai_move.to,
            new_state,
            notation,
        });

        // 发送时间更新
        pending.broadcast(room_id, ServerMessage::TimeUpdate {
            red_time_ms,
            black_time_ms,
        });

        // 处理游戏结束
        if let Some(result) = game_over {
            let room = match state.rooms.get_mut(room_id) {
                Some(r) => r,
                None => return,
            };
            room.finish(result.clone());
            pending.broadcast(room_id, ServerMessage::GameOver { result });
        }
    }

    /// AI 失败，判定 AI 负（玩家胜）
    fn ai_loses(
        state: &mut ServerState,
        pending: &mut PendingMessages,
        room_id: RoomId,
    ) {
        let room = match state.rooms.get_mut(room_id) {
            Some(r) => r,
            None => return,
        };

        if room.state != RoomState::Playing {
            return;
        }

        // 玩家是红方，AI 是黑方，所以红方胜
        let result = GameResult::RedWin(WinReason::Resign);
        room.finish(result.clone());
        pending.broadcast(room_id, ServerMessage::GameOver { result });
    }

    /// 处理悔棋请求
    fn handle_request_undo(
        state: &mut ServerState,
        pending: &mut PendingMessages,
        player_id: PlayerId,
    ) -> Option<ServerMessage> {
        let room_id = state.rooms.find_player_room(player_id)?;
        let room = state.rooms.get(room_id)?;

        if room.state != RoomState::Playing {
            return Some(ServerMessage::Error {
                code: ErrorCode::GameNotStarted,
                message: "游戏未开始".to_string(),
            });
        }

        if room.move_history.is_empty() {
            return Some(ServerMessage::Error {
                code: ErrorCode::UndoNotAllowed,
                message: "没有可悔的棋".to_string(),
            });
        }

        let player_side = room.get_player_side(player_id)?;
        let is_pve = matches!(room.room_type, RoomType::PvE(_));
        let opponent_id = room.get_opponent_id(player_id);

        // PvE 模式直接悔棋
        if is_pve {
            let room = state.rooms.get_mut(room_id)?;
            
            // PvE 模式需要撤回两步：玩家一步 + AI一步
            // 确保悔棋后轮到玩家
            let mut undo_count = 0;
            let target_count = if room.move_history.len() >= 2 { 2 } else { 1 };
            
            for _ in 0..target_count {
                if room.undo_move().is_ok() {
                    undo_count += 1;
                } else {
                    break;
                }
            }
            
            if undo_count > 0 {
                let new_state = room.game_state.clone()?;
                return Some(ServerMessage::UndoApproved { new_state });
            } else {
                return Some(ServerMessage::Error {
                    code: ErrorCode::UndoNotAllowed,
                    message: "悔棋失败".to_string(),
                });
            }
        }

        // PvP 模式需要对方同意
        let room = state.rooms.get_mut(room_id)?;
        room.undo_requested_by = Some(player_side);
        
        // 通知对手
        if let Some(opponent_id) = opponent_id {
            pending.send(opponent_id, ServerMessage::UndoRequested { by: player_side });
        }

        None
    }

    /// 处理悔棋响应
    fn handle_respond_undo(
        state: &mut ServerState,
        pending: &mut PendingMessages,
        player_id: PlayerId,
        accept: bool,
    ) -> Option<ServerMessage> {
        let room_id = state.rooms.find_player_room(player_id)?;
        let room = state.rooms.get(room_id)?;

        // 检查是否有悔棋请求
        let requester_side = room.undo_requested_by?;
        let requester_id = room.get_player_id(requester_side)?;

        if accept {
            let room = state.rooms.get_mut(room_id)?;
            if room.undo_move().is_ok() {
                let new_state = room.game_state.clone()?;
                pending.broadcast(room_id, ServerMessage::UndoApproved { new_state });
            }
        } else {
            // 通知请求方被拒绝
            pending.send(requester_id, ServerMessage::UndoRejected);
        }

        let room = state.rooms.get_mut(room_id)?;
        room.undo_requested_by = None;
        None
    }

    /// 处理认输
    fn handle_resign(
        state: &mut ServerState,
        pending: &mut PendingMessages,
        player_id: PlayerId,
    ) -> Option<ServerMessage> {
        let room_id = state.rooms.find_player_room(player_id)?;
        let room = state.rooms.get(room_id)?;

        if room.state != RoomState::Playing {
            return Some(ServerMessage::Error {
                code: ErrorCode::GameNotStarted,
                message: "游戏未开始".to_string(),
            });
        }

        let player_side = room.get_player_side(player_id)?;
        let result = match player_side {
            Side::Red => GameResult::BlackWin(WinReason::Resign),
            Side::Black => GameResult::RedWin(WinReason::Resign),
        };

        let room = state.rooms.get_mut(room_id)?;
        room.finish(result.clone());
        pending.broadcast(room_id, ServerMessage::GameOver { result });

        None
    }

    /// 处理暂停
    fn handle_pause(state: &mut ServerState, player_id: PlayerId) -> Option<ServerMessage> {
        let room_id = state.rooms.find_player_room(player_id)?;
        let room = state.rooms.get_mut(room_id)?;

        if room.pause() {
            Some(ServerMessage::GamePaused)
        } else {
            Some(ServerMessage::Error {
                code: ErrorCode::GameNotStarted,
                message: "无法暂停".to_string(),
            })
        }
    }

    /// 处理继续
    fn handle_resume(state: &mut ServerState, player_id: PlayerId) -> Option<ServerMessage> {
        let room_id = state.rooms.find_player_room(player_id)?;
        let room = state.rooms.get_mut(room_id)?;

        if room.resume() {
            Some(ServerMessage::GameResumed)
        } else {
            Some(ServerMessage::Error {
                code: ErrorCode::GameNotStarted,
                message: "无法继续".to_string(),
            })
        }
    }

    /// 处理保存棋局
    fn handle_save_game(state: &mut ServerState, player_id: PlayerId) -> Option<ServerMessage> {
        let room_id = state.rooms.find_player_room(player_id)?;
        let room = state.rooms.get(room_id)?;

        // 检查游戏状态
        if room.state != RoomState::Playing && room.state != RoomState::Paused {
            return Some(ServerMessage::Error {
                code: ErrorCode::GameNotStarted,
                message: "只能保存进行中的棋局".to_string(),
            });
        }

        // 获取玩家名称
        let red_name = if let Some(red_id) = room.red_player {
            if red_id == protocol::AI_PLAYER_ID {
                "AI".to_string()
            } else {
                state.players.get_nickname(red_id).unwrap_or("未知").to_string()
            }
        } else {
            "空".to_string()
        };

        let black_name = if let Some(black_id) = room.black_player {
            if black_id == protocol::AI_PLAYER_ID {
                "AI".to_string()
            } else {
                state.players.get_nickname(black_id).unwrap_or("未知").to_string()
            }
        } else {
            "空".to_string()
        };

        // 生成棋谱记录
        if let Some(mut record) = room.generate_game_record(&red_name, &black_name) {
            let (red_time, black_time) = room.get_time_state();
            
            // 保存 AI 难度
            if let RoomType::PvE(difficulty) = room.room_type {
                let difficulty_str = match difficulty {
                    protocol::Difficulty::Easy => "Easy",
                    protocol::Difficulty::Medium => "Medium",
                    protocol::Difficulty::Hard => "Hard",
                };
                record.set_ai_difficulty(difficulty_str);
            }
            
            // 保存到文件
            match state.storage.save_game(
                &red_name,
                &black_name,
                &mut record,
                room.game_state.as_ref()?,
                red_time,
                black_time,
            ) {
                Ok(game_id) => Some(ServerMessage::GameSaved { game_id }),
                Err(e) => Some(ServerMessage::Error {
                    code: ErrorCode::InternalError,
                    message: format!("保存失败: {}", e),
                }),
            }
        } else {
            Some(ServerMessage::Error {
                code: ErrorCode::GameNotStarted,
                message: "无法生成棋谱".to_string(),
            })
        }
    }

    /// 处理加载棋局
    fn handle_load_game(
        state: &mut ServerState,
        _pending: &mut PendingMessages,
        player_id: PlayerId,
        game_id: String,
    ) -> Option<ServerMessage> {
        // 检查玩家是否在房间中
        if let Some(_room_id) = state.rooms.find_player_room(player_id) {
            return Some(ServerMessage::Error {
                code: ErrorCode::AlreadyInRoom,
                message: "请先离开当前房间".to_string(),
            });
        }

        // 加载棋谱
        let record = match state.storage.load_game(&game_id) {
            Ok(record) => record,
            Err(e) => {
                return Some(ServerMessage::Error {
                    code: ErrorCode::RoomNotFound,
                    message: format!("加载失败: {}", e),
                });
            }
        };

        // 检查是否是保存的进行中棋局
        let save_info = match record.save_info {
            Some(info) => info,
            None => {
                return Some(ServerMessage::Error {
                    code: ErrorCode::GameAlreadyOver,
                    message: "只能加载进行中的棋局".to_string(),
                });
            }
        };

        // 创建新房间用于加载的棋局
        let room_type = if record.metadata.black_player == "AI" || record.metadata.red_player == "AI" {
            // 从保存的难度恢复，如果没有则默认中等
            let difficulty = record.metadata.ai_difficulty.as_ref()
                .and_then(|d| match d.as_str() {
                    "Easy" | "简单" => Some(protocol::Difficulty::Easy),
                    "Medium" | "中等" => Some(protocol::Difficulty::Medium),
                    "Hard" | "困难" => Some(protocol::Difficulty::Hard),
                    _ => None,
                })
                .unwrap_or(protocol::Difficulty::Medium);
            RoomType::PvE(difficulty)
        } else {
            RoomType::PvP
        };

        let room_id = state.rooms.create(room_type);
        let room = state.rooms.get_mut(room_id)?;

        // 设置玩家
        let your_side = if record.metadata.red_player != "AI" {
            room.red_player = Some(player_id);
            if record.metadata.black_player == "AI" {
                room.black_player = Some(protocol::AI_PLAYER_ID);
            }
            Side::Red
        } else {
            room.black_player = Some(player_id);
            room.red_player = Some(protocol::AI_PLAYER_ID);
            Side::Black
        };

        // 恢复游戏状态
        // TODO: 这里需要从 FEN 和走法历史重建棋盘状态
        // 目前简化处理，从初始状态开始重放走法
        room.start_game();

        // 重放走法
        for move_record in &record.moves {
            if let (Some(from_pos), Some(to_pos)) = (move_record.from_position(), move_record.to_position()) {
                let mv = Move::new(from_pos, to_pos);
                if room.make_move(mv).is_err() {
                    // 如果重放失败，返回错误
                    state.rooms.remove(room_id);
                    return Some(ServerMessage::Error {
                        code: ErrorCode::InternalError,
                        message: "棋谱数据损坏".to_string(),
                    });
                }
            }
        }
        
        // 设置时间（在重放走法后设置，并重置 turn_start）
        if let Some(timer) = &mut room.timer {
            timer.set_times(save_info.red_time_remaining_ms, save_info.black_time_remaining_ms);
            timer.reset_turn_start();
        }

        // 加入房间
        state.players.set_status(player_id, PlayerStatus::InRoom(room_id));

        Some(ServerMessage::GameLoaded {
            room_id,
            game_state: room.game_state.clone()?,
            your_side,
        })
    }

    /// 处理玩家断线
    pub async fn handle_disconnect(state: &mut ServerState, player_id: PlayerId) {
        let mut pending = PendingMessages::new();

        // 标记玩家断线
        if let Some(room_id) = state.players.disconnect(player_id) {
            // 设置断线超时
            state.disconnect_timeouts.insert(
                player_id,
                Instant::now() + Duration::from_secs(DISCONNECT_TIMEOUT_SECS),
            );

            // 获取房间信息
            if let Some(room) = state.rooms.get(room_id) {
                let opponent_id = room.get_opponent_id(player_id);
                let is_pve = matches!(room.room_type, RoomType::PvE(_));

                // 通知对手
                if let Some(opponent_id) = opponent_id {
                    pending.send(
                        opponent_id,
                        ServerMessage::OpponentDisconnected {
                            timeout_secs: DISCONNECT_TIMEOUT_SECS as u32,
                        },
                    );
                }

                // PvE 模式自动暂停
                if is_pve {
                    if let Some(room) = state.rooms.get_mut(room_id) {
                        room.pause();
                    }
                }
            }
        }

        // 移除连接
        state.connections.remove(&player_id);

        // 发送待发送的消息
        pending.flush(state).await;
    }

    /// 检查断线超时
    pub async fn check_disconnect_timeouts(state: &mut ServerState) {
        let now = Instant::now();
        let mut timed_out = Vec::new();

        for (&player_id, &timeout) in &state.disconnect_timeouts {
            if now >= timeout {
                timed_out.push(player_id);
            }
        }

        let mut pending = PendingMessages::new();

        for player_id in timed_out {
            state.disconnect_timeouts.remove(&player_id);
            
            // 查找玩家房间并判负
            if let Some(room_id) = state.rooms.find_player_room(player_id) {
                if let Some(room) = state.rooms.get(room_id) {
                    if room.state == RoomState::Playing {
                        let player_side = room.get_player_side(player_id);
                        let opponent_id = room.get_opponent_id(player_id);

                        if let Some(side) = player_side {
                            let result = match side {
                                Side::Red => GameResult::BlackWin(WinReason::Disconnect),
                                Side::Black => GameResult::RedWin(WinReason::Disconnect),
                            };

                            // 更新房间状态
                            if let Some(room) = state.rooms.get_mut(room_id) {
                                room.finish(result.clone());
                            }

                            // 通知对手胜利
                            if let Some(opponent_id) = opponent_id {
                                pending.send(opponent_id, ServerMessage::GameOver { result });
                            }
                        }
                    }
                }
            }

            // 移除玩家
            state.players.remove(player_id);
        }

        pending.flush(state).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_login() {
        let mut state = ServerState::new().unwrap();
        
        let result = MessageHandler::handle_login(&mut state, "玩家1".to_string());
        assert!(matches!(result, Some(ServerMessage::LoginSuccess { .. })));
    }

    #[tokio::test]
    async fn test_create_room() {
        let mut state = ServerState::new().unwrap();
        
        // 先登录
        let login_result = MessageHandler::handle_login(&mut state, "玩家1".to_string());
        let player_id = match login_result {
            Some(ServerMessage::LoginSuccess { player_id }) => player_id,
            _ => panic!("Login failed"),
        };

        // 创建房间
        let result = MessageHandler::handle_create_room(
            &mut state,
            player_id,
            RoomType::PvP,
            None,
        );

        assert!(matches!(result, Some(ServerMessage::RoomCreated { .. })));
    }
}

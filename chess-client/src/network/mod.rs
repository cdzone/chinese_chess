//! 网络通信模块
//!
//! 使用全局静态 tokio Runtime 处理异步网络操作

mod connection;

pub use connection::*;

use bevy::prelude::*;
use protocol::{Position, ClientMessage, ServerMessage};
use std::sync::Arc;
use std::time::Instant;

use crate::GameState;

/// 网络插件
pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NetworkState::default())
            .insert_resource(NetworkConnectionHandle::default())
            .add_event::<NetworkEvent>()
            .add_event::<ServerMessageEvent>()
            .add_systems(
                Update,
                (
                    handle_network_events,
                    handle_server_messages,
                    poll_network,
                    check_quick_match_timeout,
                ),
            );
    }
}

/// 网络连接句柄（Bevy 资源）
#[derive(Resource, Default, Clone)]
pub struct NetworkConnectionHandle {
    /// 共享的网络连接
    pub connection: Arc<NetworkConnection>,
}

/// 待处理操作（登录成功后执行）
#[derive(Clone, Debug, Default)]
pub enum PendingAction {
    #[default]
    None,
    /// 创建房间
    CreateRoom { room_type: protocol::RoomType, preferred_side: Option<protocol::Side> },
    /// 获取房间列表
    ListRooms,
    /// 快速匹配（加入第一个可用房间或创建新房间）
    QuickMatch,
}

/// 网络状态
#[derive(Resource, Default)]
pub struct NetworkState {
    /// 连接状态
    pub status: ConnectionStatus,
    /// 服务器地址
    pub server_addr: String,
    /// 玩家 ID
    pub player_id: Option<protocol::PlayerId>,
    /// 房间 ID
    pub room_id: Option<protocol::RoomId>,
    /// 玩家昵称
    pub nickname: String,
    /// 登录成功后待执行的操作
    pub pending_action: PendingAction,
    /// 当前房间类型（用于再来一局）
    pub current_room_type: Option<protocol::RoomType>,
    /// 房间列表（从服务器获取）
    pub room_list: Vec<protocol::RoomInfo>,
    /// 是否正在快速匹配
    pub is_quick_matching: bool,
    /// 快速匹配开始时间（用于超时检测）
    pub quick_match_start: Option<Instant>,
}

/// 快速匹配超时时间（秒）
const QUICK_MATCH_TIMEOUT_SECS: u64 = 10;

/// 连接状态
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// 网络事件（客户端发起）
#[derive(Event, Clone, Debug)]
pub enum NetworkEvent {
    /// 连接服务器
    Connect { addr: String, nickname: String },
    /// 断开连接
    Disconnect,
    /// 创建房间
    CreateRoom { room_type: protocol::RoomType, preferred_side: Option<protocol::Side> },
    /// 加入房间
    JoinRoom { room_id: protocol::RoomId },
    /// 离开房间
    LeaveRoom,
    /// 获取房间列表
    ListRooms,
    /// 发送走棋
    SendMove { from: Position, to: Position },
    /// 发送悔棋请求
    SendUndo,
    /// 响应悔棋请求
    RespondUndo { accept: bool },
    /// 发送认输
    SendResign,
    /// 发送暂停
    SendPause,
    /// 发送继续
    SendResume,
    /// 保存游戏
    SaveGame,
    /// 加载游戏
    LoadGame { game_id: String },
}

/// 服务器消息事件（收到服务器消息后触发）
#[derive(Event, Clone, Debug)]
pub struct ServerMessageEvent(pub ServerMessage);

/// 处理网络事件
fn handle_network_events(
    mut events: EventReader<NetworkEvent>,
    mut network: ResMut<NetworkState>,
    conn_handle: Res<NetworkConnectionHandle>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        match event {
            NetworkEvent::Connect { addr, nickname } => {
                network.server_addr = addr.clone();
                network.nickname = nickname.clone();
                network.status = ConnectionStatus::Connecting;
                game_state.set(GameState::Connecting);
                
                tracing::info!("Connecting to {} as {}", addr, nickname);
                
                // 使用全局 Runtime 连接
                conn_handle.connection.connect(addr.clone(), nickname.clone());
            }
            NetworkEvent::Disconnect => {
                network.status = ConnectionStatus::Disconnected;
                network.player_id = None;
                network.room_id = None;
                conn_handle.connection.disconnect();
                game_state.set(GameState::Menu);
            }
            NetworkEvent::CreateRoom { room_type, preferred_side } => {
                // 保存房间类型
                network.current_room_type = Some(room_type.clone());
                
                let msg = ClientMessage::CreateRoom {
                    room_type: room_type.clone(),
                    preferred_side: *preferred_side,
                };
                conn_handle.connection.queue_send(msg);
                tracing::info!("Creating room: {:?}", room_type);
            }
            NetworkEvent::JoinRoom { room_id } => {
                let msg = ClientMessage::JoinRoom { room_id: *room_id };
                conn_handle.connection.queue_send(msg);
                tracing::info!("Joining room: {:?}", room_id);
            }
            NetworkEvent::LeaveRoom => {
                let msg = ClientMessage::LeaveRoom;
                conn_handle.connection.queue_send(msg);
                network.room_id = None;
            }
            NetworkEvent::ListRooms => {
                let msg = ClientMessage::ListRooms;
                conn_handle.connection.queue_send(msg);
            }
            NetworkEvent::SendMove { from, to } => {
                let msg = ClientMessage::MakeMove { from: *from, to: *to };
                conn_handle.connection.queue_send(msg);
                tracing::info!("Move: {:?} -> {:?}", from, to);
            }
            NetworkEvent::SendUndo => {
                let msg = ClientMessage::RequestUndo;
                conn_handle.connection.queue_send(msg);
            }
            NetworkEvent::RespondUndo { accept } => {
                let msg = ClientMessage::RespondUndo { accept: *accept };
                conn_handle.connection.queue_send(msg);
            }
            NetworkEvent::SendResign => {
                let msg = ClientMessage::Resign;
                conn_handle.connection.queue_send(msg);
            }
            NetworkEvent::SendPause => {
                let msg = ClientMessage::PauseGame;
                conn_handle.connection.queue_send(msg);
            }
            NetworkEvent::SendResume => {
                let msg = ClientMessage::ResumeGame;
                conn_handle.connection.queue_send(msg);
            }
            NetworkEvent::SaveGame => {
                let msg = ClientMessage::SaveGame;
                conn_handle.connection.queue_send(msg);
            }
            NetworkEvent::LoadGame { game_id } => {
                let msg = ClientMessage::LoadGame { game_id: game_id.clone() };
                conn_handle.connection.queue_send(msg);
            }
        }
    }
}

/// 处理服务器消息
fn handle_server_messages(
    mut events: EventReader<ServerMessageEvent>,
    mut game: ResMut<crate::game::ClientGame>,
    mut network: ResMut<NetworkState>,
    mut game_state: ResMut<NextState<GameState>>,
    conn_handle: Res<NetworkConnectionHandle>,
) {
    for ServerMessageEvent(msg) in events.read() {
        match msg {
            ServerMessage::LoginSuccess { player_id } => {
                network.player_id = Some(*player_id);
                network.status = ConnectionStatus::Connected;
                tracing::info!("Login success: {:?}", player_id);
                
                // 执行待处理操作
                match std::mem::take(&mut network.pending_action) {
                    PendingAction::None => {
                        game_state.set(GameState::Lobby);
                    }
                    PendingAction::CreateRoom { room_type, preferred_side } => {
                        // 保存房间类型
                        network.current_room_type = Some(room_type.clone());
                        
                        let msg = ClientMessage::CreateRoom {
                            room_type,
                            preferred_side,
                        };
                        conn_handle.connection.queue_send(msg);
                        tracing::info!("Creating room after login");
                    }
                    PendingAction::ListRooms => {
                        let msg = ClientMessage::ListRooms;
                        conn_handle.connection.queue_send(msg);
                        game_state.set(GameState::Lobby);
                    }
                    PendingAction::QuickMatch => {
                        // 快速匹配：先获取房间列表
                        network.is_quick_matching = true;
                        network.quick_match_start = Some(Instant::now());
                        let msg = ClientMessage::ListRooms;
                        conn_handle.connection.queue_send(msg);
                        tracing::info!("Quick match: fetching room list");
                    }
                }
            }
            ServerMessage::RoomCreated { room_id, .. } => {
                network.room_id = Some(*room_id);
                tracing::info!("Room created: {:?}", room_id);
            }
            ServerMessage::RoomJoined { room_id, side } => {
                network.room_id = Some(*room_id);
                game.player_side = Some(*side);
                tracing::info!("Joined room {:?} as {:?}", room_id, side);
            }
            ServerMessage::GameStarted { initial_state, your_side, .. } => {
                // 将 RoomType 转换为 GameMode
                let room_type = network.current_room_type.clone().unwrap_or(protocol::RoomType::PvP);
                let room_id = network.room_id.unwrap_or(0);
                let game_mode = match room_type {
                    protocol::RoomType::PvE(difficulty) => {
                        crate::game::GameMode::OnlinePvE { room_id, difficulty }
                    }
                    protocol::RoomType::PvP => {
                        crate::game::GameMode::OnlinePvP { room_id }
                    }
                };
                game.start_game(initial_state.clone(), *your_side, game_mode);
                game_state.set(GameState::Playing);
                tracing::info!("Game started!");
            }
            ServerMessage::MoveMade { from, to, new_state, notation } => {
                game.update_state(new_state.clone(), *from, *to, notation.clone());
            }
            ServerMessage::TimeUpdate { red_time_ms, black_time_ms } => {
                game.update_time(*red_time_ms, *black_time_ms);
            }
            ServerMessage::UndoApproved { new_state } => {
                // PvE 模式悔棋 2 步（玩家+AI），PvP 模式悔棋 1 步
                let steps = if game.is_pve() { 2 } else { 1 };
                game.undo(new_state.clone(), steps);
            }
            ServerMessage::GameOver { result } => {
                game.set_result(result.clone());
                game_state.set(GameState::GameOver);
                tracing::info!("Game over: {:?}", result);
            }
            ServerMessage::GamePaused => {
                game.is_paused = true;
            }
            ServerMessage::GameResumed => {
                game.is_paused = false;
            }
            ServerMessage::RoomList { rooms } => {
                tracing::info!("Received room list: {} rooms", rooms.len());
                network.room_list = rooms.clone();
                
                // 如果正在快速匹配，自动加入或创建房间
                if network.is_quick_matching {
                    network.is_quick_matching = false;
                    network.quick_match_start = None;  // 清除超时计时器
                    
                    // 查找等待中的 PvP 房间
                    let waiting_room = rooms.iter().find(|r| {
                        matches!(r.room_type, protocol::RoomType::PvP) && 
                        r.state == protocol::RoomState::Waiting
                    });
                    
                    if let Some(room) = waiting_room {
                        // 加入已有房间
                        let msg = ClientMessage::JoinRoom { room_id: room.id };
                        conn_handle.connection.queue_send(msg);
                        tracing::info!("Quick match: joining room {:?}", room.id);
                    } else {
                        // 没有可用房间，创建新房间
                        network.current_room_type = Some(protocol::RoomType::PvP);
                        let msg = ClientMessage::CreateRoom {
                            room_type: protocol::RoomType::PvP,
                            preferred_side: None,
                        };
                        conn_handle.connection.queue_send(msg);
                        tracing::info!("Quick match: creating new room");
                    }
                }
            }
            ServerMessage::Error { code, message } => {
                tracing::error!("Server error {:?}: {}", code, message);
            }
            _ => {
                tracing::debug!("Unhandled server message: {:?}", msg);
            }
        }
    }
}

/// 轮询网络消息
fn poll_network(
    conn_handle: Res<NetworkConnectionHandle>,
    mut server_events: EventWriter<ServerMessageEvent>,
    mut network: ResMut<NetworkState>,
) {
    // 从接收队列获取消息
    let messages = conn_handle.connection.drain_received();
    
    for msg in messages {
        // 检查是否是登录成功消息，更新连接状态
        if matches!(msg, ServerMessage::LoginSuccess { .. }) {
            network.status = ConnectionStatus::Connected;
        }
        
        server_events.send(ServerMessageEvent(msg));
    }
}

/// 检查快速匹配超时
fn check_quick_match_timeout(
    mut network: ResMut<NetworkState>,
    mut game_state: ResMut<NextState<GameState>>,
    conn_handle: Res<NetworkConnectionHandle>,
) {
    if !network.is_quick_matching {
        return;
    }
    
    if let Some(start_time) = network.quick_match_start {
        if start_time.elapsed().as_secs() > QUICK_MATCH_TIMEOUT_SECS {
            tracing::warn!("Quick match timeout after {} seconds, returning to menu", QUICK_MATCH_TIMEOUT_SECS);
            
            // 清除快速匹配状态
            network.is_quick_matching = false;
            network.quick_match_start = None;
            
            // 断开连接并返回主菜单
            conn_handle.connection.disconnect();
            game_state.set(GameState::Menu);
        }
    }
}

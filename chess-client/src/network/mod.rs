//! 网络通信模块
//!
//! 处理与服务器的连接和消息收发

mod connection;

pub use connection::*;

use bevy::prelude::*;
use protocol::{Position, ClientMessage, ServerMessage};
use std::sync::Arc;

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
}

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
                
                // 异步连接服务器
                let connection = conn_handle.connection.clone();
                let addr_clone = addr.clone();
                let nickname_clone = nickname.clone();
                
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        if connection.connect(&addr_clone).await.is_ok() {
                            // 连接成功后发送登录消息
                            connection.queue_send(ClientMessage::Login { nickname: nickname_clone });
                            // 启动轮询任务
                            loop {
                                if let Err(e) = connection.poll().await {
                                    tracing::error!("Network poll error: {}", e);
                                    break;
                                }
                                tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                            }
                        } else {
                            tracing::error!("Failed to connect to {}", addr_clone);
                        }
                    });
                });
            }
            NetworkEvent::Disconnect => {
                network.status = ConnectionStatus::Disconnected;
                network.player_id = None;
                network.room_id = None;
                
                let connection = conn_handle.connection.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let _ = connection.disconnect().await;
                    });
                });
                
                game_state.set(GameState::Menu);
            }
            NetworkEvent::CreateRoom { room_type, preferred_side } => {
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
                // 从网络状态获取房间类型，默认 PvP
                let room_type = protocol::RoomType::PvP;
                game.start_game(initial_state.clone(), *your_side, room_type);
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
                // 默认悔棋 1 步
                game.undo(new_state.clone(), 1);
            }
            ServerMessage::GameOver { result } => {
                game_state.set(GameState::GameOver);
                tracing::info!("Game over: {:?}", result);
            }
            ServerMessage::GamePaused => {
                game.is_paused = true;
            }
            ServerMessage::GameResumed => {
                game.is_paused = false;
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

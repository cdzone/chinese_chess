//! 中国象棋服务端入口

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};

use chess_server::{MessageHandler, ServerState};
use protocol::{
    ClientMessage, FrameReader, FrameWriter, PlayerId, ProtocolError, ServerMessage,
};

/// 默认服务端口
const DEFAULT_PORT: u16 = 9527;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let addr: SocketAddr = format!("0.0.0.0:{}", DEFAULT_PORT).parse()?;
    let listener = TcpListener::bind(&addr).await?;
    
    info!("中国象棋服务器启动，监听 {}", addr);

    let state = Arc::new(RwLock::new(ServerState::new()?));

    // 启动断线超时检查任务
    let state_clone = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            let mut state = state_clone.write().await;
            MessageHandler::check_disconnect_timeouts(&mut state).await;
        }
    });

    loop {
        let (socket, addr) = listener.accept().await?;
        info!("新连接: {}", addr);

        let state = state.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, state).await {
                error!("连接处理错误: {}", e);
            }
        });
    }
}

async fn handle_connection(
    socket: tokio::net::TcpStream,
    state: Arc<RwLock<ServerState>>,
) -> anyhow::Result<()> {
    let (read_half, write_half) = socket.into_split();
    let mut reader = FrameReader::new(read_half);
    let mut writer = FrameWriter::new(write_half);

    // 创建消息通道
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(32);

    // 等待登录消息
    let player_id: PlayerId;
    loop {
        match reader.read_frame::<ClientMessage>().await {
            Ok(msg) => {
                match msg {
                    ClientMessage::Login { nickname } => {
                        let mut state = state.write().await;
                        match state.players.login(nickname) {
                            Ok(id) => {
                                player_id = id;
                                state.connections.insert(id, tx.clone());
                                
                                // 发送登录成功
                                let response = ServerMessage::LoginSuccess { player_id: id };
                                writer.write_frame(&response).await?;
                                break;
                            }
                            Err(msg) => {
                                let response = ServerMessage::Error {
                                    code: protocol::ErrorCode::InvalidNickname,
                                    message: msg.to_string(),
                                };
                                writer.write_frame(&response).await?;
                            }
                        }
                    }
                    ClientMessage::Reconnect { player_id: pid, room_id } => {
                        let mut state = state.write().await;
                        if let Some(response) = MessageHandler::handle(
                            &mut state,
                            pid,
                            ClientMessage::Reconnect { player_id: pid, room_id },
                        ).await {
                            if matches!(response, ServerMessage::ReconnectSuccess { .. }) {
                                player_id = pid;
                                state.connections.insert(pid, tx.clone());
                                writer.write_frame(&response).await?;
                                break;
                            } else {
                                writer.write_frame(&response).await?;
                            }
                        }
                    }
                    _ => {
                        let response = ServerMessage::Error {
                            code: protocol::ErrorCode::InvalidNickname,
                            message: "请先登录".to_string(),
                        };
                        writer.write_frame(&response).await?;
                    }
                }
            }
            Err(ProtocolError::ConnectionClosed) => {
                info!("客户端断开连接（登录前）");
                return Ok(());
            }
            Err(e) => {
                error!("读取帧错误: {}", e);
                return Err(e.into());
            }
        }
    }

    info!("玩家 {} 登录成功", player_id);

    // 启动发送任务
    let mut write_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if writer.write_frame(&msg).await.is_err() {
                break;
            }
        }
    });

    // 主循环：读取客户端消息
    loop {
        tokio::select! {
            result = reader.read_frame::<ClientMessage>() => {
                match result {
                    Ok(msg) => {
                        let mut state = state.write().await;
                        
                        if let Some(response) = MessageHandler::handle(&mut state, player_id, msg).await {
                            let _ = tx.send(response).await;
                        }
                    }
                    Err(ProtocolError::ConnectionClosed) => {
                        info!("玩家 {} 断开连接", player_id);
                        break;
                    }
                    Err(e) => {
                        error!("读取帧错误: {}", e);
                        break;
                    }
                }
            }
            _ = &mut write_task => {
                warn!("发送任务结束");
                break;
            }
        }
    }

    // 处理断线
    {
        let mut state = state.write().await;
        MessageHandler::handle_disconnect(&mut state, player_id).await;
    }

    Ok(())
}

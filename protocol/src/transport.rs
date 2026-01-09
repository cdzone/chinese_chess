//! 传输层抽象
//!
//! 提供 Connector/Connection/Listener traits 使上层协议与具体传输实现解耦，
//! 便于从 TCP 切换到 QUIC 等其他传输协议。

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::error::{ProtocolError, Result};
use crate::{CONNECT_TIMEOUT, MAX_FRAME_SIZE, PROTOCOL_VERSION};

/// 传输协议类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportType {
    Tcp,
    Quic,
}

/// 网络配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub transport: TransportType,
    pub host: String,
    pub port: u16,
    /// QUIC 专用配置
    pub quic_cert_path: Option<String>,
    pub quic_key_path: Option<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            transport: TransportType::Tcp,
            host: "127.0.0.1".to_string(),
            port: 9527,
            quic_cert_path: None,
            quic_key_path: None,
        }
    }
}

/// 连接抽象 trait（核心抽象，用于业务层）
#[async_trait]
pub trait Connection: Send + Sync {
    /// 发送消息
    async fn send<M: Serialize + Send + Sync>(&mut self, msg: &M) -> Result<()>;

    /// 接收消息
    async fn recv<M: DeserializeOwned>(&mut self) -> Result<M>;

    /// 关闭连接
    async fn close(&mut self) -> Result<()>;

    /// 获取远端地址
    fn peer_addr(&self) -> Option<String>;
}

/// 连接器 trait（客户端使用）
#[async_trait]
pub trait Connector: Send + Sync {
    type Conn: Connection;

    /// 建立连接
    async fn connect(&self, addr: &str) -> Result<Self::Conn>;
}

/// 监听器 trait（服务端使用）
#[async_trait]
pub trait Listener: Send + Sync + Sized {
    type Conn: Connection;

    /// 绑定地址
    async fn bind(addr: &str) -> Result<Self>;

    /// 接受连接
    async fn accept(&mut self) -> Result<Self::Conn>;

    /// 获取本地地址
    fn local_addr(&self) -> Option<String>;
}

// ============================================================================
// TCP 实现
// ============================================================================

/// TCP 连接器
pub struct TcpConnector;

#[async_trait]
impl Connector for TcpConnector {
    type Conn = TcpConnection;

    async fn connect(&self, addr: &str) -> Result<Self::Conn> {
        let stream = timeout(CONNECT_TIMEOUT, TcpStream::connect(addr))
            .await
            .map_err(|_| ProtocolError::ConnectionTimeout)?
            .map_err(ProtocolError::Io)?;

        stream.set_nodelay(true)?;

        let peer_addr = stream.peer_addr().ok().map(|a| a.to_string());
        let (read_half, write_half) = stream.into_split();

        Ok(TcpConnection {
            reader: FrameReader::new(read_half),
            writer: FrameWriter::new(write_half),
            peer_addr,
        })
    }
}

/// TCP 连接
pub struct TcpConnection {
    reader: FrameReader<OwnedReadHalf>,
    writer: FrameWriter<OwnedWriteHalf>,
    peer_addr: Option<String>,
}

impl TcpConnection {
    /// 从 TcpStream 创建（服务端使用）
    pub fn from_stream(stream: TcpStream) -> Result<Self> {
        stream.set_nodelay(true)?;
        let peer_addr = stream.peer_addr().ok().map(|a| a.to_string());
        let (read_half, write_half) = stream.into_split();

        Ok(Self {
            reader: FrameReader::new(read_half),
            writer: FrameWriter::new(write_half),
            peer_addr,
        })
    }

    /// 分离读写端
    pub fn split(self) -> (FrameReader<OwnedReadHalf>, FrameWriter<OwnedWriteHalf>) {
        (self.reader, self.writer)
    }
}

#[async_trait]
impl Connection for TcpConnection {
    async fn send<M: Serialize + Send + Sync>(&mut self, msg: &M) -> Result<()> {
        self.writer.write_frame(msg).await
    }

    async fn recv<M: DeserializeOwned>(&mut self) -> Result<M> {
        self.reader.read_frame().await
    }

    async fn close(&mut self) -> Result<()> {
        // TCP 连接会在 drop 时自动关闭
        Ok(())
    }

    fn peer_addr(&self) -> Option<String> {
        self.peer_addr.clone()
    }
}

/// TCP 监听器
pub struct TcpListener {
    listener: tokio::net::TcpListener,
}

#[async_trait]
impl Listener for TcpListener {
    type Conn = TcpConnection;

    async fn bind(addr: &str) -> Result<Self> {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(ProtocolError::Io)?;
        Ok(Self { listener })
    }

    async fn accept(&mut self) -> Result<Self::Conn> {
        let (stream, _addr) = self.listener.accept().await.map_err(ProtocolError::Io)?;
        TcpConnection::from_stream(stream)
    }

    fn local_addr(&self) -> Option<String> {
        self.listener.local_addr().ok().map(|a| a.to_string())
    }
}

// ============================================================================
// 帧编解码
// ============================================================================

/// 帧头大小: 1 字节版本 + 4 字节长度
const HEADER_SIZE: usize = 5;

/// 帧读取器
pub struct FrameReader<R> {
    reader: R,
    buffer: Vec<u8>,
}

impl<R: AsyncRead + Unpin + Send> FrameReader<R> {
    /// 创建新的帧读取器
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: Vec::with_capacity(MAX_FRAME_SIZE),
        }
    }

    /// 读取并解码一帧消息
    pub async fn read_frame<M: DeserializeOwned>(&mut self) -> Result<M> {
        // 读取帧头
        let mut header = [0u8; HEADER_SIZE];
        self.reader.read_exact(&mut header).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                ProtocolError::ConnectionClosed
            } else {
                ProtocolError::Io(e)
            }
        })?;

        // 解析版本号
        let version = header[0];
        if version != PROTOCOL_VERSION {
            return Err(ProtocolError::VersionMismatch {
                expected: PROTOCOL_VERSION,
                actual: version,
            });
        }

        // 解析长度（大端序）
        let length = u32::from_be_bytes([header[1], header[2], header[3], header[4]]) as usize;

        // 检查帧大小
        if length > MAX_FRAME_SIZE {
            return Err(ProtocolError::FrameTooLarge {
                size: length,
                max: MAX_FRAME_SIZE,
            });
        }

        // 读取消息体
        if self.buffer.len() < length {
            self.buffer.resize(length, 0);
        }
        self.reader
            .read_exact(&mut self.buffer[..length])
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    ProtocolError::ConnectionClosed
                } else {
                    ProtocolError::Io(e)
                }
            })?;

        // 反序列化
        let msg = bincode::deserialize(&self.buffer[..length])?;
        Ok(msg)
    }

    /// 接收消息（read_frame 的别名）
    pub async fn recv<M: DeserializeOwned>(&mut self) -> Result<M> {
        self.read_frame().await
    }
}

/// 帧写入器
pub struct FrameWriter<W> {
    writer: W,
}

impl<W: AsyncWrite + Unpin + Send> FrameWriter<W> {
    /// 创建新的帧写入器
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// 编码并写入一帧消息
    pub async fn write_frame<M: Serialize>(&mut self, msg: &M) -> Result<()> {
        // 序列化消息
        let payload = bincode::serialize(msg)?;

        // 检查大小
        if payload.len() > MAX_FRAME_SIZE {
            return Err(ProtocolError::FrameTooLarge {
                size: payload.len(),
                max: MAX_FRAME_SIZE,
            });
        }

        // 构造帧头
        let length = payload.len() as u32;
        let mut header = [0u8; HEADER_SIZE];
        header[0] = PROTOCOL_VERSION;
        header[1..5].copy_from_slice(&length.to_be_bytes());

        // 写入帧头和消息体
        self.writer.write_all(&header).await?;
        self.writer.write_all(&payload).await?;
        self.writer.flush().await?;

        Ok(())
    }

    /// 发送消息（write_frame 的别名）
    pub async fn send<M: Serialize>(&mut self, msg: &M) -> Result<()> {
        self.write_frame(msg).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{ClientMessage, ServerMessage};

    #[tokio::test]
    async fn test_tcp_connection() {
        // 启动监听
        let mut listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // 客户端连接
        let client_handle = tokio::spawn(async move {
            let connector = TcpConnector;
            let mut conn = connector.connect(&addr).await.unwrap();

            // 发送消息
            conn.send(&ClientMessage::Login {
                nickname: "test".to_string(),
            })
            .await
            .unwrap();

            // 接收响应
            let msg: ServerMessage = conn.recv().await.unwrap();
            match msg {
                ServerMessage::LoginSuccess { player_id } => assert_eq!(player_id, 1),
                _ => panic!("Unexpected message"),
            }
        });

        // 服务端接受连接
        let mut conn = listener.accept().await.unwrap();

        // 接收消息
        let msg: ClientMessage = conn.recv().await.unwrap();
        match msg {
            ClientMessage::Login { nickname } => assert_eq!(nickname, "test"),
            _ => panic!("Unexpected message"),
        }

        // 发送响应
        conn.send(&ServerMessage::LoginSuccess { player_id: 1 })
            .await
            .unwrap();

        client_handle.await.unwrap();
    }
}

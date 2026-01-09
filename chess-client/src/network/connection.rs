//! 网络连接管理
//!
//! 使用 protocol 库的传输层抽象

use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;
use protocol::{ClientMessage, ServerMessage, TcpConnection, TcpConnector, Connector, Connection};

/// 网络连接包装器
/// 
/// 用于在 Bevy 的同步环境中管理异步网络连接
pub struct NetworkConnection {
    /// 内部连接（异步）
    inner: Arc<Mutex<Option<TcpConnection>>>,
    /// 发送队列（使用标准库 Mutex 以支持同步访问）
    send_queue: Arc<StdMutex<Vec<ClientMessage>>>,
    /// 接收队列（使用标准库 Mutex 以支持同步访问）
    recv_queue: Arc<StdMutex<Vec<ServerMessage>>>,
}

impl NetworkConnection {
    /// 创建新的网络连接管理器
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
            send_queue: Arc::new(StdMutex::new(Vec::new())),
            recv_queue: Arc::new(StdMutex::new(Vec::new())),
        }
    }

    /// 连接到服务器（在异步上下文中调用）
    pub async fn connect(&self, addr: &str) -> anyhow::Result<()> {
        let connector = TcpConnector;
        let conn = connector.connect(addr).await?;
        
        let mut inner = self.inner.lock().await;
        *inner = Some(conn);
        
        tracing::info!("Connected to server: {}", addr);
        Ok(())
    }

    /// 断开连接
    pub async fn disconnect(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        if let Some(mut conn) = inner.take() {
            conn.close().await?;
        }
        Ok(())
    }

    /// 发送消息（加入发送队列，同步调用）
    pub fn queue_send(&self, msg: ClientMessage) {
        if let Ok(mut queue) = self.send_queue.lock() {
            queue.push(msg);
        }
    }

    /// 获取接收到的消息（同步版本）
    pub fn drain_received(&self) -> Vec<ServerMessage> {
        if let Ok(mut queue) = self.recv_queue.lock() {
            std::mem::take(&mut *queue)
        } else {
            Vec::new()
        }
    }

    /// 处理网络 I/O（在异步任务中定期调用）
    pub async fn poll(&self) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().await;
        let Some(conn) = inner.as_mut() else {
            return Ok(());
        };

        // 发送队列中的消息
        {
            let messages: Vec<ClientMessage> = {
                if let Ok(mut queue) = self.send_queue.lock() {
                    std::mem::take(&mut *queue)
                } else {
                    Vec::new()
                }
            };
            
            for msg in messages {
                if let Err(e) = conn.send(&msg).await {
                    tracing::error!("Failed to send message: {}", e);
                }
            }
        }

        // 接收消息（非阻塞尝试）
        match tokio::time::timeout(
            std::time::Duration::from_millis(1),
            conn.recv::<ServerMessage>(),
        ).await {
            Ok(Ok(msg)) => {
                if let Ok(mut queue) = self.recv_queue.lock() {
                    queue.push(msg);
                }
            }
            Ok(Err(e)) => {
                // 接收错误，可能是连接断开
                tracing::warn!("Receive error: {}", e);
            }
            Err(_) => {
                // 超时，没有消息（正常情况）
            }
        }

        Ok(())
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        let inner = self.inner.lock().await;
        inner.is_some()
    }
}

impl Default for NetworkConnection {
    fn default() -> Self {
        Self::new()
    }
}

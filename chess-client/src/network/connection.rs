//! 网络连接管理
//!
//! 使用 protocol 库的传输层抽象，通过独立的读写任务避免帧边界问题

use std::sync::{Arc, Mutex as StdMutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use protocol::{ClientMessage, ServerMessage, TcpConnector, Connector};

/// 网络连接包装器
/// 
/// 用于在 Bevy 的同步环境中管理异步网络连接
pub struct NetworkConnection {
    /// 发送通道
    send_tx: Arc<StdMutex<Option<mpsc::UnboundedSender<ClientMessage>>>>,
    /// 接收队列（使用标准库 Mutex 以支持同步访问）
    recv_queue: Arc<StdMutex<Vec<ServerMessage>>>,
    /// 是否正在运行
    running: Arc<AtomicBool>,
}

impl NetworkConnection {
    /// 创建新的网络连接管理器
    pub fn new() -> Self {
        Self {
            send_tx: Arc::new(StdMutex::new(None)),
            recv_queue: Arc::new(StdMutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 连接到服务器并启动读写任务
    pub async fn connect(&self, addr: &str) -> anyhow::Result<()> {
        let connector = TcpConnector;
        let conn = connector.connect(addr).await?;
        
        tracing::info!("Connected to server: {}", addr);
        
        // 分离读写端
        let (reader, writer) = conn.split();
        
        // 创建发送通道
        let (send_tx, send_rx) = mpsc::unbounded_channel::<ClientMessage>();
        
        // 保存发送端
        if let Ok(mut tx) = self.send_tx.lock() {
            *tx = Some(send_tx);
        }
        
        self.running.store(true, Ordering::SeqCst);
        
        // 启动写任务
        let running_write = self.running.clone();
        tokio::spawn(async move {
            write_task(writer, send_rx, running_write).await;
        });
        
        // 启动读任务
        let recv_queue = self.recv_queue.clone();
        let running_read = self.running.clone();
        tokio::spawn(async move {
            read_task(reader, recv_queue, running_read).await;
        });
        
        Ok(())
    }

    /// 断开连接
    pub async fn disconnect(&self) -> anyhow::Result<()> {
        self.running.store(false, Ordering::SeqCst);
        if let Ok(mut tx) = self.send_tx.lock() {
            *tx = None; // 关闭发送通道
        }
        Ok(())
    }

    /// 发送消息（加入发送队列，同步调用）
    pub fn queue_send(&self, msg: ClientMessage) {
        if let Ok(tx) = self.send_tx.lock() {
            if let Some(sender) = tx.as_ref() {
                let _ = sender.send(msg);
            }
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

    /// 空的 poll 函数（保持 API 兼容，实际工作由独立任务完成）
    pub async fn poll(&self) -> anyhow::Result<()> {
        // 读写任务已经在独立的 tokio 任务中运行
        // 这里只需要检查是否还在运行
        if !self.running.load(Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Connection closed"));
        }
        Ok(())
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 检查是否已连接
    pub async fn is_connected(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for NetworkConnection {
    fn default() -> Self {
        Self::new()
    }
}

/// 写任务：从通道接收消息并发送到服务器
async fn write_task(
    mut writer: protocol::FrameWriter<tokio::net::tcp::OwnedWriteHalf>,
    mut rx: mpsc::UnboundedReceiver<ClientMessage>,
    running: Arc<AtomicBool>,
) {
    while running.load(Ordering::SeqCst) {
        match rx.recv().await {
            Some(msg) => {
                if let Err(e) = writer.send(&msg).await {
                    tracing::error!("Failed to send message: {}", e);
                    running.store(false, Ordering::SeqCst);
                    break;
                }
            }
            None => {
                // 通道关闭
                break;
            }
        }
    }
    tracing::info!("Write task ended");
}

/// 读任务：从服务器接收消息并放入队列
async fn read_task(
    mut reader: protocol::FrameReader<tokio::net::tcp::OwnedReadHalf>,
    recv_queue: Arc<StdMutex<Vec<ServerMessage>>>,
    running: Arc<AtomicBool>,
) {
    while running.load(Ordering::SeqCst) {
        match reader.recv::<ServerMessage>().await {
            Ok(msg) => {
                if let Ok(mut queue) = recv_queue.lock() {
                    queue.push(msg);
                }
            }
            Err(e) => {
                tracing::warn!("Receive error: {}", e);
                running.store(false, Ordering::SeqCst);
                break;
            }
        }
    }
    tracing::info!("Read task ended");
}

//! 网络连接管理
//!
//! 使用全局共享的 tokio Runtime，避免频繁创建销毁

use std::sync::{Arc, Mutex as StdMutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio::runtime::Runtime;
use protocol::{ClientMessage, ServerMessage, TcpConnector, Connector};

/// 全局 Tokio Runtime 包装器
pub struct TokioRuntime {
    runtime: Runtime,
}

impl TokioRuntime {
    /// 创建新的 Runtime
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create tokio runtime");
        Self { runtime }
    }

    /// 获取 runtime handle
    pub fn handle(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
    }
}

impl Default for TokioRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// 网络连接包装器
/// 
/// 使用共享的 Runtime 运行异步任务
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

    /// 使用指定的 runtime handle 连接到服务器
    pub fn connect_with_handle(
        &self,
        addr: String,
        nickname: String,
        handle: tokio::runtime::Handle,
    ) {
        let send_tx = self.send_tx.clone();
        let recv_queue = self.recv_queue.clone();
        let running = self.running.clone();

        // 在共享的 runtime 上 spawn 连接任务
        handle.spawn(async move {
            if let Err(e) = connect_task(addr, nickname, send_tx, recv_queue, running).await {
                tracing::error!("Connection task error: {}", e);
            }
        });
    }

    /// 断开连接
    pub fn disconnect(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Ok(mut tx) = self.send_tx.lock() {
            *tx = None; // 关闭发送通道
        }
    }

    /// 发送消息（加入发送队列，同步调用）
    pub fn queue_send(&self, msg: ClientMessage) {
        if let Ok(tx) = self.send_tx.lock() {
            if let Some(sender) = tx.as_ref() {
                if let Err(e) = sender.send(msg) {
                    tracing::error!("Failed to queue message: {}", e);
                }
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

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for NetworkConnection {
    fn default() -> Self {
        Self::new()
    }
}

/// 连接任务：建立连接并启动读写循环
async fn connect_task(
    addr: String,
    nickname: String,
    send_tx: Arc<StdMutex<Option<mpsc::UnboundedSender<ClientMessage>>>>,
    recv_queue: Arc<StdMutex<Vec<ServerMessage>>>,
    running: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let connector = TcpConnector;
    let conn = connector.connect(&addr).await?;
    
    tracing::info!("Connected to server: {}", addr);
    
    // 分离读写端
    let (reader, writer) = conn.split();
    
    // 创建发送通道
    let (tx, rx) = mpsc::unbounded_channel::<ClientMessage>();
    
    // 保存发送端
    if let Ok(mut sender) = send_tx.lock() {
        *sender = Some(tx.clone());
    }
    
    running.store(true, Ordering::SeqCst);
    
    // 发送登录消息
    tx.send(ClientMessage::Login { nickname })?;
    
    // 启动读写任务
    let running_write = running.clone();
    let running_read = running.clone();
    
    let write_handle = tokio::spawn(write_task(writer, rx, running_write));
    let read_handle = tokio::spawn(read_task(reader, recv_queue, running_read));
    
    // 等待任一任务结束
    tokio::select! {
        result = write_handle => {
            if let Err(e) = result {
                tracing::error!("Write task panicked: {}", e);
            }
        }
        result = read_handle => {
            if let Err(e) = result {
                tracing::error!("Read task panicked: {}", e);
            }
        }
    }
    
    running.store(false, Ordering::SeqCst);
    tracing::info!("Connection closed");
    
    Ok(())
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
                tracing::debug!("Sending: {:?}", msg);
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
                tracing::debug!("Received: {:?}", msg);
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

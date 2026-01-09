//! 网络连接管理
//!
//! 使用全局静态 tokio Runtime，正确管理任务生命周期

use std::sync::{Arc, Mutex as StdMutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use protocol::{ClientMessage, ServerMessage, TcpConnector, Connector};

/// 连接超时时间
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

// 全局静态 Tokio Runtime
// 使用 lazy_static 确保 Runtime 在整个程序生命周期内存活
lazy_static::lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
    };
}

/// 获取全局 Runtime 的 Handle
pub fn runtime_handle() -> tokio::runtime::Handle {
    RUNTIME.handle().clone()
}

/// 网络连接包装器
/// 
/// 正确管理任务生命周期，支持取消和资源清理
pub struct NetworkConnection {
    /// 发送通道
    send_tx: Arc<StdMutex<Option<mpsc::UnboundedSender<ClientMessage>>>>,
    /// 接收队列（使用标准库 Mutex 以支持同步访问）
    recv_queue: Arc<StdMutex<Vec<ServerMessage>>>,
    /// 是否正在运行
    running: Arc<AtomicBool>,
    /// 主连接任务句柄（用于取消）
    task_handle: Arc<StdMutex<Option<JoinHandle<()>>>>,
}

impl NetworkConnection {
    /// 创建新的网络连接管理器
    pub fn new() -> Self {
        Self {
            send_tx: Arc::new(StdMutex::new(None)),
            recv_queue: Arc::new(StdMutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
            task_handle: Arc::new(StdMutex::new(None)),
        }
    }

    /// 连接到服务器
    /// 
    /// 如果已有连接，会先取消旧连接
    pub fn connect(&self, addr: String, nickname: String) {
        // 先取消旧任务
        self.abort_task();
        
        // 清理旧状态
        if let Ok(mut tx) = self.send_tx.lock() {
            *tx = None;
        }
        if let Ok(mut queue) = self.recv_queue.lock() {
            queue.clear();
        }
        
        let send_tx = self.send_tx.clone();
        let recv_queue = self.recv_queue.clone();
        let running = self.running.clone();

        // 在全局 Runtime 上 spawn 连接任务
        let handle = RUNTIME.spawn(async move {
            if let Err(e) = connect_task(addr, nickname, send_tx, recv_queue, running).await {
                tracing::error!("Connection task error: {}", e);
            }
        });
        
        // 保存任务句柄
        if let Ok(mut task) = self.task_handle.lock() {
            *task = Some(handle);
        }
    }

    /// 断开连接并清理资源
    pub fn disconnect(&self) {
        // 设置停止标志
        self.running.store(false, Ordering::SeqCst);
        
        // 关闭发送通道（会导致写任务退出）
        if let Ok(mut tx) = self.send_tx.lock() {
            *tx = None;
        }
        
        // 取消任务
        self.abort_task();
        
        // 清理接收队列
        if let Ok(mut queue) = self.recv_queue.lock() {
            queue.clear();
        }
    }

    /// 取消当前任务
    fn abort_task(&self) {
        if let Ok(mut handle) = self.task_handle.lock() {
            if let Some(task) = handle.take() {
                task.abort();
                tracing::debug!("Aborted previous connection task");
            }
        }
    }

    /// 发送消息（加入发送队列，同步调用）
    pub fn queue_send(&self, msg: ClientMessage) {
        if let Ok(tx) = self.send_tx.lock() {
            if let Some(sender) = tx.as_ref() {
                if let Err(e) = sender.send(msg) {
                    tracing::error!("Failed to queue message: {}", e);
                }
            } else {
                tracing::warn!("Cannot send message: not connected");
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
    
    /// 检查是否已连接
    pub fn is_connected(&self) -> bool {
        if let Ok(tx) = self.send_tx.lock() {
            tx.is_some() && self.running.load(Ordering::SeqCst)
        } else {
            false
        }
    }
}

impl Default for NetworkConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for NetworkConnection {
    fn drop(&mut self) {
        // 确保资源被清理
        self.disconnect();
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
    // 带超时的连接
    let connector = TcpConnector;
    let conn = match tokio::time::timeout(CONNECT_TIMEOUT, connector.connect(&addr)).await {
        Ok(Ok(conn)) => conn,
        Ok(Err(e)) => {
            tracing::error!("Failed to connect to {}: {}", addr, e);
            return Err(e.into());
        }
        Err(_) => {
            tracing::error!("Connection to {} timed out", addr);
            return Err(anyhow::anyhow!("Connection timeout"));
        }
    };
    
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
                if !e.is_cancelled() {
                    tracing::error!("Write task panicked: {}", e);
                }
            }
        }
        result = read_handle => {
            if let Err(e) = result {
                if !e.is_cancelled() {
                    tracing::error!("Read task panicked: {}", e);
                }
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
                tracing::trace!("Sending: {:?}", msg);
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
    tracing::debug!("Write task ended");
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
                tracing::trace!("Received: {:?}", msg);
                if let Ok(mut queue) = recv_queue.lock() {
                    queue.push(msg);
                }
            }
            Err(e) => {
                if running.load(Ordering::SeqCst) {
                    tracing::warn!("Receive error: {}", e);
                }
                running.store(false, Ordering::SeqCst);
                break;
            }
        }
    }
    tracing::debug!("Read task ended");
}

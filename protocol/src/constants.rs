//! 协议常量定义

use std::time::Duration;

/// 协议版本号
pub const PROTOCOL_VERSION: u8 = 1;

/// 棋盘宽度（列数）
pub const BOARD_WIDTH: usize = 9;

/// 棋盘高度（行数）
pub const BOARD_HEIGHT: usize = 10;

/// 昵称最大长度
pub const MAX_NICKNAME_LEN: usize = 20;

/// 消息帧最大大小
pub const MAX_FRAME_SIZE: usize = 65536;

/// 服务端最大连接数
pub const MAX_CONNECTIONS: usize = 100;

/// 客户端心跳间隔（秒）
pub const HEARTBEAT_INTERVAL_SECS: u64 = 10;

/// 服务端心跳超时（秒）- 超过此时间无消息则断开
pub const HEARTBEAT_TIMEOUT_SECS: u64 = 30;

/// 连接超时（秒）
pub const CONNECT_TIMEOUT_SECS: u64 = 10;

/// 断线重连超时（秒）
pub const RECONNECT_TIMEOUT_SECS: u64 = 60;

/// 每方初始时间（毫秒）- 10分钟
pub const INITIAL_TIME_MS: u64 = 10 * 60 * 1000;

/// AI 玩家 ID（使用最大值避免与真实玩家 ID 冲突）
pub const AI_PLAYER_ID: u64 = u64::MAX;

/// 心跳间隔 Duration
pub const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(HEARTBEAT_INTERVAL_SECS);

/// 心跳超时 Duration
pub const HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(HEARTBEAT_TIMEOUT_SECS);

/// 连接超时 Duration
pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(CONNECT_TIMEOUT_SECS);

/// 断线重连超时 Duration
pub const RECONNECT_TIMEOUT: Duration = Duration::from_secs(RECONNECT_TIMEOUT_SECS);

//! 错误类型定义

use thiserror::Error;

/// 象棋规则错误
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ChessError {
    /// 无效的位置
    #[error("Invalid position: ({x}, {y})")]
    InvalidPosition { x: i8, y: i8 },

    /// 无效的走法
    #[error("Invalid move: from ({from_x}, {from_y}) to ({to_x}, {to_y})")]
    InvalidMove {
        from_x: u8,
        from_y: u8,
        to_x: u8,
        to_y: u8,
    },

    /// 没有棋子
    #[error("No piece at position ({x}, {y})")]
    NoPiece { x: u8, y: u8 },

    /// 不是你的回合
    #[error("Not your turn")]
    NotYourTurn,

    /// 走法会导致被将军
    #[error("Move would leave king in check")]
    KingInCheck,

    /// 无效的 FEN 字符串
    #[error("Invalid FEN string: {reason}")]
    InvalidFen { reason: String },

    /// 游戏已结束
    #[error("Game is already over")]
    GameOver,
}

/// 协议错误类型
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// IO 错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 序列化错误（bincode）
    #[error("Bincode serialization error: {0}")]
    Bincode(#[from] bincode::Error),

    /// JSON 序列化错误
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// 协议版本不匹配
    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u8, actual: u8 },

    /// 帧大小超限
    #[error("Frame too large: {size} bytes (max: {max})")]
    FrameTooLarge { size: usize, max: usize },

    /// 连接超时
    #[error("Connection timeout")]
    ConnectionTimeout,

    /// 连接已关闭
    #[error("Connection closed")]
    ConnectionClosed,

    /// 昵称为空
    #[error("Nickname is empty")]
    NicknameEmpty,

    /// 昵称过长
    #[error("Nickname too long: {len} chars (max: {max})")]
    NicknameTooLong { len: usize, max: usize },

    /// 昵称已被占用
    #[error("Nickname is already occupied")]
    NicknameOccupied,

    /// 象棋规则错误
    #[error("Chess error: {0}")]
    Chess(#[from] ChessError),
}

/// 协议操作结果类型
pub type Result<T> = std::result::Result<T, ProtocolError>;

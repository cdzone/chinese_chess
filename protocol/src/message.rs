//! 消息类型定义

use serde::{Deserialize, Serialize};

use crate::board::BoardState;
use crate::piece::{Position, Side};

/// 玩家 ID
pub type PlayerId = u64;

/// 房间 ID
pub type RoomId = u64;

/// 房间类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomType {
    /// 玩家对战
    PvP,
    /// 人机对战
    PvE(Difficulty),
}

/// AI 难度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    /// 简单：depth=3, 30%概率选次优解
    Easy,
    /// 中等：depth=4
    Medium,
    /// 困难：depth=6+
    Hard,
}

/// 游戏结果
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameResult {
    /// 红方胜
    RedWin(WinReason),
    /// 黑方胜
    BlackWin(WinReason),
    /// 和棋
    Draw(DrawReason),
}

/// 胜利原因
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WinReason {
    /// 将死
    Checkmate,
    /// 对方认输
    Resign,
    /// 对方超时
    Timeout,
    /// 对方断线超时
    Disconnect,
}

/// 和棋原因
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawReason {
    /// 双方同意
    Agreement,
    /// 无子可动（困毙）
    Stalemate,
    /// 长将/长捉
    Repetition,
    /// 60回合无吃子
    FiftyMoves,
}

/// 房间信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub id: RoomId,
    pub room_type: RoomType,
    pub red_player: Option<String>,
    pub black_player: Option<String>,
    pub state: RoomState,
}

/// 房间状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomState {
    /// 等待玩家加入
    Waiting,
    /// 游戏进行中
    Playing,
    /// 游戏暂停
    Paused,
    /// 游戏结束
    Finished,
}

/// 客户端发送给服务端的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    // === 身份认证 ===
    /// 登录
    Login { nickname: String },
    /// 重连
    Reconnect { player_id: PlayerId, room_id: RoomId },

    // === 房间操作 ===
    /// 创建房间
    CreateRoom {
        room_type: RoomType,
        preferred_side: Option<Side>,
    },
    /// 加入房间
    JoinRoom { room_id: RoomId },
    /// 离开房间
    LeaveRoom,
    /// 获取房间列表
    ListRooms,

    // === 游戏操作 ===
    /// 走棋
    MakeMove { from: Position, to: Position },
    /// 请求悔棋
    RequestUndo,
    /// 响应悔棋请求
    RespondUndo { accept: bool },
    /// 认输
    Resign,

    // === 人机专用 ===
    /// 暂停游戏
    PauseGame,
    /// 继续游戏
    ResumeGame,

    // === 棋局管理 ===
    /// 保存棋局
    SaveGame,
    /// 加载棋局
    LoadGame { game_id: String },

    // === 心跳 ===
    /// 心跳请求
    Ping,
}

/// 服务端发送给客户端的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    // === 身份认证 ===
    /// 登录成功
    LoginSuccess { player_id: PlayerId },
    /// 重连成功
    ReconnectSuccess {
        room_id: RoomId,
        game_state: BoardState,
        your_side: Side,
        red_time_ms: u64,
        black_time_ms: u64,
    },

    // === 房间事件 ===
    /// 房间创建成功
    RoomCreated { room_id: RoomId, your_side: Side },
    /// 加入房间成功
    RoomJoined { room_id: RoomId, side: Side },
    /// 房间列表
    RoomList { rooms: Vec<RoomInfo> },
    /// 对手加入
    OpponentJoined { nickname: String },

    // === 游戏事件 ===
    /// 游戏开始
    GameStarted {
        initial_state: BoardState,
        your_side: Side,
        red_player: String,
        black_player: String,
    },
    /// 走棋完成
    MoveMade {
        from: Position,
        to: Position,
        new_state: BoardState,
        notation: String,
    },
    /// 收到悔棋请求
    UndoRequested { by: Side },
    /// 悔棋被接受
    UndoApproved { new_state: BoardState },
    /// 悔棋被拒绝
    UndoRejected,
    /// 游戏结束
    GameOver { result: GameResult },
    /// 游戏暂停
    GamePaused,
    /// 游戏继续
    GameResumed,

    // === 时间 ===
    /// 时间更新
    TimeUpdate { red_time_ms: u64, black_time_ms: u64 },

    // === 断线重连 ===
    /// 对手断线
    OpponentDisconnected { timeout_secs: u32 },
    /// 对手重连
    OpponentReconnected,

    // === 棋局管理 ===
    /// 棋局保存成功
    GameSaved { game_id: String },
    /// 棋局加载成功
    GameLoaded { state: BoardState },

    // === 心跳 ===
    /// 心跳响应
    Pong,

    // === 错误 ===
    /// 错误消息
    Error { code: ErrorCode, message: String },
}

/// 错误码定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum ErrorCode {
    // === 房间相关 (1xx) ===
    /// 房间不存在
    RoomNotFound = 100,
    /// 房间已满
    RoomFull = 101,
    /// 房间已关闭
    RoomClosed = 102,
    /// 不在房间中
    NotInRoom = 103,
    /// 已在房间中
    AlreadyInRoom = 104,

    // === 游戏相关 (2xx) ===
    /// 不是你的回合
    NotYourTurn = 200,
    /// 无效走法
    InvalidMove = 201,
    /// 游戏未开始
    GameNotStarted = 202,
    /// 游戏已结束
    GameAlreadyOver = 203,
    /// 不允许悔棋
    UndoNotAllowed = 204,

    // === 玩家相关 (3xx) ===
    /// 无效昵称
    InvalidNickname = 300,
    /// 玩家不存在
    PlayerNotFound = 301,
    /// 昵称已被占用
    NicknameOccupied = 302,

    // === 系统相关 (5xx) ===
    /// 内部错误
    InternalError = 500,
    /// 超时
    Timeout = 501,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialize() {
        let msg = ClientMessage::Login {
            nickname: "player1".to_string(),
        };
        let bytes = bincode::serialize(&msg).unwrap();
        let decoded: ClientMessage = bincode::deserialize(&bytes).unwrap();
        
        match decoded {
            ClientMessage::Login { nickname } => assert_eq!(nickname, "player1"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_server_message_serialize() {
        let msg = ServerMessage::LoginSuccess { player_id: 12345 };
        let bytes = bincode::serialize(&msg).unwrap();
        let decoded: ServerMessage = bincode::deserialize(&bytes).unwrap();
        
        match decoded {
            ServerMessage::LoginSuccess { player_id } => assert_eq!(player_id, 12345),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_room_type_serialize() {
        let room_type = RoomType::PvE(Difficulty::Medium);
        let bytes = bincode::serialize(&room_type).unwrap();
        let decoded: RoomType = bincode::deserialize(&bytes).unwrap();
        assert_eq!(decoded, room_type);
    }
}

//! 中国象棋服务端
//!
//! 包含:
//! - 房间系统
//! - 对局控制
//! - 玩家管理
//! - AI 集成
//! - 棋局存储

pub mod game;
pub mod player;
pub mod room;
pub mod server;
pub mod storage;

pub use game::GameTimer;
pub use player::{Player, PlayerManager, PlayerStatus};
pub use room::{Room, RoomManager};
pub use server::{MessageHandler, ServerState};
pub use storage::{StorageManager, SavedGameInfo};

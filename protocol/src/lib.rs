//! 中国象棋共享协议库
//!
//! 包含:
//! - 棋子、棋盘、位置等核心数据结构
//! - 走法生成和规则验证
//! - 消息类型定义 (ClientMessage, ServerMessage)
//! - 传输层抽象 (Connector, Connection, Listener traits)
//! - 帧编解码 (Codec)
//! - 棋谱格式 (JSON, FEN)

mod board;
mod constants;
mod error;
mod fen;
mod message;
mod moves;
mod notation;
mod piece;
mod record;
mod transport;

pub use board::{Board, BoardState};
pub use constants::*;
pub use error::{ChessError, ProtocolError, Result};
pub use fen::{Fen, INITIAL_FEN};
pub use message::{
    ClientMessage, ServerMessage, ErrorCode, RoomInfo, RoomType, RoomState,
    GameResult, WinReason, DrawReason, Difficulty, PlayerId, RoomId,
};
pub use moves::{Move, MoveGenerator};
pub use notation::Notation;
pub use piece::{Piece, PieceType, Side, Position};
pub use record::{GameRecord, MoveRecord, GameMetadata, SaveInfo};
pub use transport::{
    Connection, Connector, Listener, 
    TcpConnection, TcpConnector, TcpListener,
    TransportType, NetworkConfig,
    FrameReader, FrameWriter,
};

//! 棋谱记录格式
//!
//! 支持 JSON 格式的棋谱存储，便于 LLM 分析

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::message::GameResult;
use crate::piece::Position;

/// 棋谱版本
pub const RECORD_VERSION: &str = "1.0";

/// 游戏元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    /// 红方玩家名
    pub red_player: String,
    /// 黑方玩家名
    pub black_player: String,
    /// 游戏日期
    pub date: String,
    /// 游戏结果
    pub result: Option<GameResult>,
    /// 时间控制（如 "10+0" 表示每方10分钟，无加秒）
    pub time_control: Option<String>,
    /// AI 难度（PvE 模式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_difficulty: Option<String>,
}

/// 走法记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveRecord {
    /// 起始位置 [x, y]
    pub from: [u8; 2],
    /// 目标位置 [x, y]
    pub to: [u8; 2],
    /// 中文纵线表示法
    pub notation: String,
    /// 走棋时的 Unix 时间戳（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    /// 走棋后剩余时间（毫秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_left_ms: Option<u64>,
}

impl MoveRecord {
    /// 创建新的走法记录
    pub fn new(from: Position, to: Position, notation: String) -> Self {
        Self {
            from: [from.x, from.y],
            to: [to.x, to.y],
            notation,
            timestamp: None,
            time_left_ms: None,
        }
    }

    /// 带时间戳创建
    pub fn with_timestamp(from: Position, to: Position, notation: String, timestamp: u64) -> Self {
        Self {
            from: [from.x, from.y],
            to: [to.x, to.y],
            notation,
            timestamp: Some(timestamp),
            time_left_ms: None,
        }
    }

    /// 获取起始位置
    pub fn from_position(&self) -> Option<Position> {
        Position::new(self.from[0], self.from[1])
    }

    /// 获取目标位置
    pub fn to_position(&self) -> Option<Position> {
        Position::new(self.to[0], self.to[1])
    }
}

/// 保存信息（用于中途保存的棋局）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveInfo {
    /// 保存时间
    pub saved_at: DateTime<Utc>,
    /// 游戏状态
    pub game_state: String,
    /// 红方剩余时间（毫秒）
    pub red_time_remaining_ms: u64,
    /// 黑方剩余时间（毫秒）
    pub black_time_remaining_ms: u64,
}

/// 完整的棋谱记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    /// 版本号
    pub version: String,
    /// 元数据
    pub metadata: GameMetadata,
    /// 初始局面 FEN
    pub initial_fen: String,
    /// 走法列表
    pub moves: Vec<MoveRecord>,
    /// 保存信息（可选，用于中途保存）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_info: Option<SaveInfo>,
}

impl GameRecord {
    /// 创建新的棋谱记录
    pub fn new(red_player: String, black_player: String) -> Self {
        Self {
            version: RECORD_VERSION.to_string(),
            metadata: GameMetadata {
                red_player,
                black_player,
                date: Utc::now().format("%Y-%m-%d").to_string(),
                result: None,
                time_control: Some("10+0".to_string()),
                ai_difficulty: None,
            },
            initial_fen: crate::fen::INITIAL_FEN.to_string(),
            moves: Vec::new(),
            save_info: None,
        }
    }

    /// 设置 AI 难度
    pub fn set_ai_difficulty(&mut self, difficulty: &str) {
        self.metadata.ai_difficulty = Some(difficulty.to_string());
    }

    /// 从自定义 FEN 创建
    pub fn from_fen(red_player: String, black_player: String, fen: String) -> Self {
        let mut record = Self::new(red_player, black_player);
        record.initial_fen = fen;
        record
    }

    /// 添加走法
    pub fn add_move(&mut self, mv: MoveRecord) {
        self.moves.push(mv);
    }

    /// 设置游戏结果
    pub fn set_result(&mut self, result: GameResult) {
        self.metadata.result = Some(result);
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// 从 JSON 字符串解析
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// 生成 LLM 友好的文本格式
    pub fn to_llm_format(&self) -> String {
        let mut output = String::new();

        output.push_str("当前棋局状态：\n");
        output.push_str(&format!("红方: {}\n", self.metadata.red_player));
        output.push_str(&format!("黑方: {}\n", self.metadata.black_player));
        output.push_str(&format!("初始局面: {}\n", self.initial_fen));

        if !self.moves.is_empty() {
            output.push_str("\n历史走法：\n");
            for (i, mv) in self.moves.iter().enumerate() {
                let round = i / 2 + 1;
                if i % 2 == 0 {
                    output.push_str(&format!("{}. {}", round, mv.notation));
                } else {
                    output.push_str(&format!("  {}\n", mv.notation));
                }
            }
            if self.moves.len() % 2 == 1 {
                output.push('\n');
            }
        }

        if let Some(ref result) = self.metadata.result {
            output.push_str(&format!("\n结果: {:?}\n", result));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{GameResult, WinReason};

    #[test]
    fn test_game_record_json() {
        let mut record = GameRecord::new("玩家1".to_string(), "AI-中等".to_string());

        // 添加一些走法
        record.add_move(MoveRecord::new(
            Position::new_unchecked(7, 2),
            Position::new_unchecked(4, 2),
            "炮二平五".to_string(),
        ));
        record.add_move(MoveRecord::new(
            Position::new_unchecked(1, 9),
            Position::new_unchecked(2, 7),
            "馬8進7".to_string(),
        ));

        record.set_result(GameResult::RedWin(WinReason::Checkmate));

        // 转换为 JSON
        let json = record.to_json().unwrap();
        println!("{}", json);

        // 从 JSON 解析
        let parsed = GameRecord::from_json(&json).unwrap();
        assert_eq!(parsed.metadata.red_player, "玩家1");
        assert_eq!(parsed.moves.len(), 2);
    }

    #[test]
    fn test_llm_format() {
        let mut record = GameRecord::new("玩家1".to_string(), "AI-中等".to_string());

        record.add_move(MoveRecord::new(
            Position::new_unchecked(7, 2),
            Position::new_unchecked(4, 2),
            "炮二平五".to_string(),
        ));
        record.add_move(MoveRecord::new(
            Position::new_unchecked(1, 9),
            Position::new_unchecked(2, 7),
            "馬8進7".to_string(),
        ));

        let llm_format = record.to_llm_format();
        println!("{}", llm_format);

        assert!(llm_format.contains("炮二平五"));
        assert!(llm_format.contains("馬8進7"));
    }

    #[test]
    fn test_move_record() {
        let mv = MoveRecord::with_timestamp(
            Position::new_unchecked(7, 2),
            Position::new_unchecked(4, 2),
            "炮二平五".to_string(),
            1234567890,
        );

        assert_eq!(mv.from_position(), Some(Position::new_unchecked(7, 2)));
        assert_eq!(mv.to_position(), Some(Position::new_unchecked(4, 2)));
        assert_eq!(mv.timestamp, Some(1234567890));
    }
}

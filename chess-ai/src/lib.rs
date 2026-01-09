//! 中国象棋 AI 引擎
//!
//! 包含:
//! - 棋局评估函数
//! - Minimax + Alpha-Beta 搜索
//! - 迭代加深
//! - 置换表

mod evaluate;
mod search;

pub use evaluate::Evaluator;
pub use search::{AiEngine, AiConfig, Difficulty};

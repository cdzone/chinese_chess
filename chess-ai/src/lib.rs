//! 中国象棋 AI 引擎
//!
//! 包含:
//! - 棋局评估函数
//! - Minimax + Alpha-Beta 搜索
//! - 迭代加深
//! - Zobrist 哈希
//! - 置换表
//! - LLM 集成（可选，需要 `llm` feature）

mod evaluate;
mod search;
mod transposition;
mod zobrist;
pub mod llm;

pub use evaluate::Evaluator;
pub use search::{AiEngine, AiConfig, Difficulty};
pub use transposition::{TranspositionTable, TTEntry, EntryType, TTStats};
pub use zobrist::ZobristTable;
pub use llm::{LlmEngine, OllamaConfig, AiBackend, PromptTemplate};

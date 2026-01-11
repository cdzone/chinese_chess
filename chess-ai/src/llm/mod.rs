//! LLM 集成模块
//!
//! 支持通过本地 Ollama 运行 LLM 作为象棋 AI 后端。

mod prompt;
mod client;
mod parser;
mod engine;

pub use prompt::PromptTemplate;
pub use client::{OllamaClient, OllamaConfig};
pub use parser::{MoveParser, LlmMove};
pub use engine::{LlmEngine, AiBackend};

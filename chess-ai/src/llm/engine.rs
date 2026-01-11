//! LLM AI 引擎
//!
//! 使用 LLM 作为象棋 AI 后端，支持：
//! - 走法生成
//! - 对局总结

use anyhow::{Result, bail};
use protocol::{BoardState, Move};

#[cfg(feature = "llm")]
use tracing::{info, warn, debug};

use super::{OllamaClient, OllamaConfig};
#[cfg(feature = "llm")]
use super::{PromptTemplate, MoveParser};

/// LLM AI 引擎
pub struct LlmEngine {
    client: OllamaClient,
    /// 最大重试次数
    max_retries: u32,
    /// 走法历史（用于生成提示）
    move_history: Vec<Move>,
}

impl LlmEngine {
    /// 创建新的 LLM 引擎
    pub fn new(config: OllamaConfig) -> Result<Self> {
        let client = OllamaClient::new(config)?;
        Ok(Self {
            client,
            max_retries: 3,
            move_history: Vec::new(),
        })
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Result<Self> {
        Self::new(OllamaConfig::default())
    }

    /// 设置最大重试次数
    pub fn set_max_retries(&mut self, retries: u32) {
        self.max_retries = retries;
    }

    /// 添加走法到历史
    pub fn add_move(&mut self, mv: Move) {
        self.move_history.push(mv);
    }

    /// 清空走法历史
    pub fn clear_history(&mut self) {
        self.move_history.clear();
    }

    /// 获取客户端配置
    pub fn config(&self) -> &OllamaConfig {
        self.client.config()
    }

    /// 设置模型
    pub fn set_model(&mut self, model: String) {
        self.client.set_model(model);
    }

    /// 检查服务是否可用
    #[cfg(feature = "llm")]
    pub async fn is_available(&self) -> bool {
        self.client.health_check().await.unwrap_or(false)
    }

    #[cfg(not(feature = "llm"))]
    pub async fn is_available(&self) -> bool {
        false
    }

    /// 生成走法（异步）
    #[cfg(feature = "llm")]
    pub async fn generate_move(&self, state: &BoardState) -> Result<Move> {
        let system = PromptTemplate::system_prompt();
        let prompt = PromptTemplate::move_request_prompt(state, &self.move_history);

        debug!("LLM prompt length: {} chars", prompt.len());

        for attempt in 1..=self.max_retries {
            info!("LLM move generation attempt {}/{}", attempt, self.max_retries);

            match self.client.generate(&prompt, Some(system)).await {
                Ok(response) => {
                    debug!("LLM raw response: {}", response);
                    
                    match MoveParser::parse_with_fix(&response, state) {
                        Ok(mv) => {
                            info!("LLM generated valid move: ({},{}) -> ({},{})",
                                mv.from.x, mv.from.y, mv.to.x, mv.to.y);
                            return Ok(mv);
                        }
                        Err(e) => {
                            warn!("Failed to parse LLM response (attempt {}): {}", attempt, e);
                            // 打印响应前 500 字符帮助调试
                            let preview: String = response.chars().take(500).collect();
                            warn!("Response preview: {}", preview);
                        }
                    }
                }
                Err(e) => {
                    warn!("LLM request failed (attempt {}): {}", attempt, e);
                }
            }
        }

        bail!("LLM failed to generate valid move after {} attempts", self.max_retries)
    }

    #[cfg(not(feature = "llm"))]
    pub async fn generate_move(&self, _state: &BoardState) -> Result<Move> {
        bail!("LLM feature not enabled. Compile with --features llm")
    }

    /// 生成对局总结（异步）
    #[cfg(feature = "llm")]
    pub async fn generate_summary(&self, state: &BoardState, result: &str) -> Result<String> {
        let prompt = PromptTemplate::game_summary_prompt(state, &self.move_history, result);

        info!("Generating game summary with LLM");
        
        let response = self.client.generate(&prompt, None).await?;
        
        Ok(response)
    }

    #[cfg(not(feature = "llm"))]
    pub async fn generate_summary(&self, _state: &BoardState, _result: &str) -> Result<String> {
        bail!("LLM feature not enabled. Compile with --features llm")
    }
}

/// AI 后端类型
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AiBackend {
    /// 传统搜索算法（Alpha-Beta）
    Traditional,
    /// LLM（需要 Ollama）
    Llm,
    /// 混合模式：LLM 失败时回退到传统算法
    Hybrid,
}

impl Default for AiBackend {
    fn default() -> Self {
        Self::Traditional
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let config = OllamaConfig::default();
        let engine = LlmEngine::new(config);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_move_history() {
        let mut engine = LlmEngine::with_defaults().unwrap();
        
        let mv = Move::new(
            protocol::Position::new_unchecked(1, 2),
            protocol::Position::new_unchecked(4, 2),
        );
        
        engine.add_move(mv);
        assert_eq!(engine.move_history.len(), 1);
        
        engine.clear_history();
        assert_eq!(engine.move_history.len(), 0);
    }

    #[test]
    fn test_ai_backend_default() {
        let backend = AiBackend::default();
        assert_eq!(backend, AiBackend::Traditional);
    }
}

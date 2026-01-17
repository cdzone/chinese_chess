//! LLM AI 引擎
//!
//! 使用 LLM 作为象棋 AI 后端，支持：
//! - 走法生成
//! - 对局总结
//! - 对局复盘分析

use anyhow::{Result, bail};
use protocol::{Board, BoardState, Move};

#[cfg(feature = "llm")]
use tracing::{info, warn, debug};

use super::{OllamaClient, OllamaConfig};
use super::analysis::GameAnalysis;
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

    /// 检查服务是否可用，返回具体错误信息
    #[cfg(feature = "llm")]
    pub async fn check_available(&self) -> Result<()> {
        self.client.health_check().await
    }

    /// 检查服务是否可用（简化版，仅返回 bool）
    #[cfg(feature = "llm")]
    pub async fn is_available(&self) -> bool {
        self.client.health_check().await.is_ok()
    }

    #[cfg(not(feature = "llm"))]
    pub async fn check_available(&self) -> Result<()> {
        Err(anyhow::anyhow!("LLM feature not enabled"))
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

    /// 生成对局复盘分析（异步）
    /// 
    /// 返回结构化的对局分析报告，包括：
    /// - 开局评价
    /// - 关键时刻（精彩/失误/转折点）
    /// - 残局评价
    /// - 改进建议
    /// - 整体评分
    #[cfg(feature = "llm")]
    pub async fn analyze_game(
        &self,
        state: &BoardState,
        initial_board: &Board,
        result: &str,
        red_player: &str,
        black_player: &str,
    ) -> Result<GameAnalysis> {
        let system = PromptTemplate::analysis_system_prompt();
        let prompt = PromptTemplate::game_analysis_prompt(
            state,
            initial_board,
            &self.move_history,
            result,
            red_player,
            black_player,
        );

        info!("Generating game analysis with LLM, {} moves", self.move_history.len());
        debug!("Analysis prompt length: {} chars", prompt.len());

        for attempt in 1..=self.max_retries {
            info!("Game analysis attempt {}/{}", attempt, self.max_retries);

            match self.client.generate(&prompt, Some(system)).await {
                Ok(response) => {
                    debug!("LLM analysis response length: {} chars", response.len());

                    match MoveParser::parse_analysis(&response) {
                        Ok(analysis) => {
                            info!("Successfully parsed game analysis");
                            return Ok(analysis);
                        }
                        Err(e) => {
                            warn!("Failed to parse analysis response (attempt {}): {}", attempt, e);
                            let preview: String = response.chars().take(500).collect();
                            warn!("Response preview: {}", preview);
                        }
                    }
                }
                Err(e) => {
                    warn!("LLM analysis request failed (attempt {}): {}", attempt, e);
                }
            }
        }

        // 所有重试失败后，返回带有错误信息的默认分析
        warn!("All analysis attempts failed, returning default analysis");
        Ok(GameAnalysis::default())
    }

    #[cfg(not(feature = "llm"))]
    pub async fn analyze_game(
        &self,
        _state: &BoardState,
        _initial_board: &Board,
        _result: &str,
        _red_player: &str,
        _black_player: &str,
    ) -> Result<GameAnalysis> {
        bail!("LLM feature not enabled. Compile with --features llm")
    }
}

/// AI 后端类型
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum AiBackend {
    /// 传统搜索算法（Alpha-Beta）
    #[default]
    Traditional,
    /// LLM（需要 Ollama）
    Llm,
    /// 混合模式：LLM 失败时回退到传统算法
    Hybrid,
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

//! LLM 走法解析器
//!
//! 解析 LLM 返回的 JSON 格式走法，并验证其合法性。

use anyhow::{Result, Context, bail};
use protocol::{BoardState, Move, Position, MoveGenerator};
use serde::Deserialize;
use tracing::{debug, warn};

/// LLM 返回的走法格式
#[derive(Debug, Deserialize)]
pub struct LlmMove {
    pub from: [u8; 2],
    pub to: [u8; 2],
    #[serde(default)]
    pub reason: Option<String>,
}

/// LLM 走法解析器
pub struct MoveParser;

impl MoveParser {
    /// 从 LLM 响应中解析走法
    pub fn parse_response(response: &str) -> Result<LlmMove> {
        // 尝试直接解析
        if let Ok(mv) = serde_json::from_str::<LlmMove>(response) {
            return Ok(mv);
        }

        // 尝试提取 JSON 部分
        let json_str = Self::extract_json(response)
            .context("Failed to extract JSON from response")?;
        
        serde_json::from_str(&json_str)
            .context("Failed to parse extracted JSON")
    }

    /// 从文本中提取 JSON
    fn extract_json(text: &str) -> Result<String> {
        // 查找 { 和 } 的位置
        let start = text.find('{')
            .context("No JSON object found in response")?;
        
        let mut depth = 0;
        let mut end = start;
        
        for (i, ch) in text[start..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        
        if depth != 0 {
            bail!("Unbalanced braces in JSON");
        }
        
        Ok(text[start..end].to_string())
    }

    /// 将 LLM 走法转换为游戏走法
    pub fn to_move(llm_move: &LlmMove) -> Result<Move> {
        let from = Position::new(llm_move.from[0], llm_move.from[1])
            .context("Invalid 'from' position")?;
        let to = Position::new(llm_move.to[0], llm_move.to[1])
            .context("Invalid 'to' position")?;
        
        Ok(Move::new(from, to))
    }

    /// 解析并验证走法
    pub fn parse_and_validate(response: &str, state: &BoardState) -> Result<Move> {
        let llm_move = Self::parse_response(response)?;
        debug!("Parsed LLM move: {:?}", llm_move);
        
        let mv = Self::to_move(&llm_move)?;
        
        // 验证走法合法性
        Self::validate_move(&mv, state)?;
        
        if let Some(reason) = &llm_move.reason {
            debug!("LLM reasoning: {}", reason);
        }
        
        Ok(mv)
    }

    /// 验证走法是否合法
    pub fn validate_move(mv: &Move, state: &BoardState) -> Result<()> {
        // 检查起点是否有己方棋子
        let piece = state.board.get(mv.from)
            .context("No piece at 'from' position")?;
        
        if piece.side != state.current_turn {
            bail!("Cannot move opponent's piece");
        }

        // 检查是否在合法走法列表中
        let legal_moves = MoveGenerator::generate_legal(state);
        
        if !legal_moves.iter().any(|m| m.from == mv.from && m.to == mv.to) {
            warn!("LLM suggested illegal move: {:?}", mv);
            bail!("Move is not legal: ({},{}) -> ({},{})", 
                mv.from.x, mv.from.y, mv.to.x, mv.to.y);
        }

        Ok(())
    }

    /// 尝试修复常见的 LLM 错误
    pub fn try_fix_response(response: &str) -> String {
        let mut fixed = response.to_string();
        
        // 修复常见问题
        // 1. 移除 deepseek-r1 等模型的 <think>...</think> 标签
        if let Some(think_end) = fixed.find("</think>") {
            fixed = fixed[think_end + 8..].to_string();
        }
        
        // 2. 移除 markdown 代码块标记
        fixed = fixed.replace("```json", "").replace("```", "");
        
        // 3. 修复单引号
        fixed = fixed.replace('\'', "\"");
        
        // 4. 移除注释
        let lines: Vec<&str> = fixed.lines()
            .filter(|line| !line.trim().starts_with("//"))
            .collect();
        fixed = lines.join("\n");
        
        fixed.trim().to_string()
    }

    /// 解析带修复的响应
    pub fn parse_with_fix(response: &str, state: &BoardState) -> Result<Move> {
        // 先尝试修复（处理 <think> 标签等）
        let fixed = Self::try_fix_response(response);
        
        // 尝试解析修复后的内容
        if let Ok(mv) = Self::parse_and_validate(&fixed, state) {
            return Ok(mv);
        }

        // 如果修复后仍失败，尝试原始内容
        Self::parse_and_validate(response, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_json() {
        let response = r#"{"from": [1, 2], "to": [4, 2], "reason": "炮二平五，中炮开局"}"#;
        let result = MoveParser::parse_response(response);
        assert!(result.is_ok());
        
        let llm_move = result.unwrap();
        assert_eq!(llm_move.from, [1, 2]);
        assert_eq!(llm_move.to, [4, 2]);
        assert_eq!(llm_move.reason, Some("炮二平五，中炮开局".to_string()));
    }

    #[test]
    fn test_parse_json_with_text() {
        let response = r#"
        好的，我分析了当前局势。
        {"from": [7, 0], "to": [7, 2]}
        这是最佳走法。
        "#;
        
        let result = MoveParser::parse_response(response);
        assert!(result.is_ok());
        
        let llm_move = result.unwrap();
        assert_eq!(llm_move.from, [7, 0]);
        assert_eq!(llm_move.to, [7, 2]);
    }

    #[test]
    fn test_parse_invalid_json() {
        let response = "这不是有效的 JSON";
        let result = MoveParser::parse_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_move() {
        let llm_move = LlmMove {
            from: [1, 2],
            to: [4, 2],
            reason: None,
        };
        
        let result = MoveParser::to_move(&llm_move);
        assert!(result.is_ok());
        
        let mv = result.unwrap();
        assert_eq!(mv.from.x, 1);
        assert_eq!(mv.from.y, 2);
        assert_eq!(mv.to.x, 4);
        assert_eq!(mv.to.y, 2);
    }

    #[test]
    fn test_to_move_invalid_position() {
        let llm_move = LlmMove {
            from: [10, 2],  // x > 8，无效
            to: [4, 2],
            reason: None,
        };
        
        let result = MoveParser::to_move(&llm_move);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_move() {
        let state = BoardState::initial();
        
        // 合法走法：炮二平五
        let valid_move = Move::new(
            Position::new_unchecked(1, 2),
            Position::new_unchecked(4, 2),
        );
        assert!(MoveParser::validate_move(&valid_move, &state).is_ok());
        
        // 非法走法：炮斜走（炮不能斜走）
        let invalid_move = Move::new(
            Position::new_unchecked(1, 2),
            Position::new_unchecked(2, 3),
        );
        assert!(MoveParser::validate_move(&invalid_move, &state).is_err());
    }

    #[test]
    fn test_try_fix_response() {
        let response = "```json\n{'from': [1, 2], 'to': [4, 2]}\n```";
        let fixed = MoveParser::try_fix_response(response);
        
        assert!(!fixed.contains("```"));
        assert!(!fixed.contains('\''));
    }

    #[test]
    fn test_extract_json_nested() {
        let text = r#"Some text {"outer": {"inner": 1}} more text"#;
        let result = MoveParser::extract_json(text);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"outer": {"inner": 1}}"#);
    }
}

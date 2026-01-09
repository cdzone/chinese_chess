//! 搜索引擎
//!
//! 实现 Minimax + Alpha-Beta 剪枝 + 迭代加深

use std::time::{Duration, Instant};

use protocol::{BoardState, Move, MoveGenerator, Side};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::evaluate::Evaluator;

// 重导出 Difficulty 以便外部使用
pub use protocol::Difficulty;

/// AI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub difficulty: Difficulty,
    pub max_depth: u8,
    pub time_limit_ms: u64,
    pub use_opening_book: bool,
}

impl AiConfig {
    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        match difficulty {
            Difficulty::Easy => Self {
                difficulty,
                max_depth: 3,
                time_limit_ms: 1000,
                use_opening_book: false,
            },
            Difficulty::Medium => Self {
                difficulty,
                max_depth: 4,
                time_limit_ms: 3000,
                use_opening_book: false,
            },
            Difficulty::Hard => Self {
                difficulty,
                max_depth: 6,
                time_limit_ms: 5000,
                use_opening_book: true,
            },
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self::from_difficulty(Difficulty::Medium)
    }
}

/// AI 引擎
pub struct AiEngine {
    config: AiConfig,
    nodes_searched: u64,
}

impl AiEngine {
    /// 创建新的 AI 引擎
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
            nodes_searched: 0,
        }
    }

    /// 从难度创建
    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        Self::new(AiConfig::from_difficulty(difficulty))
    }

    /// 搜索最佳走法
    pub fn search(&mut self, state: &BoardState) -> Option<Move> {
        self.nodes_searched = 0;
        let deadline = Instant::now() + Duration::from_millis(self.config.time_limit_ms);

        // 生成所有合法走法
        let moves = MoveGenerator::generate_legal(state);
        if moves.is_empty() {
            return None;
        }

        // 如果只有一个走法，直接返回
        if moves.len() == 1 {
            return Some(moves[0]);
        }

        // 迭代加深搜索
        let mut best_move = moves[0];
        let mut best_score = i32::MIN;

        for depth in 1..=self.config.max_depth {
            if Instant::now() >= deadline {
                break;
            }

            let mut current_best_move = moves[0];
            let mut current_best_score = i32::MIN;

            for mv in &moves {
                if Instant::now() >= deadline {
                    break;
                }

                // 模拟走法
                let mut new_state = state.clone();
                new_state.board.move_piece(mv.from, mv.to);
                new_state.switch_turn();

                // Alpha-Beta 搜索
                // 注意：depth 表示剩余搜索层数，当前层已经在这里展开，所以传 depth-1
                // 但外层 depth 从 1 开始，所以实际搜索深度是正确的
                let score = -self.alpha_beta(
                    &new_state,
                    depth,  // 修复：传入 depth 而非 depth-1，因为当前层已在此展开
                    i32::MIN + 1,
                    i32::MAX - 1,
                    &deadline,
                );

                if score > current_best_score {
                    current_best_score = score;
                    current_best_move = *mv;
                }
            }

            // 更新最佳走法
            if current_best_score > best_score || depth == 1 {
                best_score = current_best_score;
                best_move = current_best_move;
            }
        }

        // Easy 难度：30% 概率选择次优解
        if self.config.difficulty == Difficulty::Easy {
            if rand::random::<f32>() < 0.3 {
                // 随机选择一个走法
                let mut rng = rand::thread_rng();
                if let Some(random_move) = moves.choose(&mut rng) {
                    return Some(*random_move);
                }
            }
        }

        Some(best_move)
    }

    /// Alpha-Beta 搜索
    fn alpha_beta(
        &mut self,
        state: &BoardState,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        deadline: &Instant,
    ) -> i32 {
        self.nodes_searched += 1;

        // 检查时间：超时时返回当前静态评估值，而非 0
        if Instant::now() >= *deadline {
            return self.evaluate(state);
        }

        // 到达深度限制，返回评估值
        if depth == 0 {
            return self.quiescence(state, alpha, beta, 4);
        }

        // 生成所有合法走法
        let moves = MoveGenerator::generate_legal(state);

        // 无子可动
        if moves.is_empty() {
            if MoveGenerator::is_in_check(&state.board, state.current_turn) {
                // 被将死
                return -10000 + (self.config.max_depth - depth) as i32;
            } else {
                // 困毙（和棋）
                return 0;
            }
        }

        for mv in moves {
            // 模拟走法
            let mut new_state = state.clone();
            new_state.board.move_piece(mv.from, mv.to);
            new_state.switch_turn();

            let score = -self.alpha_beta(&new_state, depth - 1, -beta, -alpha, deadline);

            if score >= beta {
                return beta; // Beta 剪枝
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    /// 静态搜索（只搜索吃子走法）
    fn quiescence(&mut self, state: &BoardState, mut alpha: i32, beta: i32, depth: u8) -> i32 {
        self.nodes_searched += 1;

        // 静态评估
        let stand_pat = self.evaluate(state);

        if depth == 0 {
            return stand_pat;
        }

        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // 只搜索吃子走法
        let moves = MoveGenerator::generate_legal(state);
        let captures: Vec<_> = moves.into_iter().filter(|m| m.captured.is_some()).collect();

        for mv in captures {
            let mut new_state = state.clone();
            new_state.board.move_piece(mv.from, mv.to);
            new_state.switch_turn();

            let score = -self.quiescence(&new_state, -beta, -alpha, depth - 1);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
            }
        }

        alpha
    }

    /// 评估当前局面
    fn evaluate(&self, state: &BoardState) -> i32 {
        let score = Evaluator::evaluate(&state.board);
        // 根据当前走子方调整符号
        match state.current_turn {
            Side::Red => score,
            Side::Black => -score,
        }
    }

    /// 获取搜索的节点数
    pub fn nodes_searched(&self) -> u64 {
        self.nodes_searched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::Fen;

    #[test]
    fn test_search_initial_position() {
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Easy);

        let mv = engine.search(&state);
        assert!(mv.is_some());
        println!("Best move: {:?}", mv);
        println!("Nodes searched: {}", engine.nodes_searched());
    }

    #[test]
    fn test_search_checkmate() {
        // 一个简单的将死局面
        let fen = "3k5/9/9/9/9/9/9/9/3r5/3K5 r 0 1";
        let state = Fen::parse(fen).unwrap();

        // 红方应该能找到逃跑的走法
        let mut engine = AiEngine::from_difficulty(Difficulty::Medium);
        let mv = engine.search(&state);

        // 可能没有合法走法（被将死）
        if mv.is_some() {
            println!("Found escape: {:?}", mv);
        }
    }

    #[test]
    fn test_difficulty_config() {
        let easy = AiConfig::from_difficulty(Difficulty::Easy);
        assert_eq!(easy.max_depth, 3);
        assert_eq!(easy.time_limit_ms, 1000);

        let medium = AiConfig::from_difficulty(Difficulty::Medium);
        assert_eq!(medium.max_depth, 4);

        let hard = AiConfig::from_difficulty(Difficulty::Hard);
        assert_eq!(hard.max_depth, 6);
        assert!(hard.use_opening_book);
    }
}

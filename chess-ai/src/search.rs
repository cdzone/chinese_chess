//! 搜索引擎
//!
//! 实现 Minimax + Alpha-Beta 剪枝 + 迭代加深 + 置换表

use std::time::{Duration, Instant};

use protocol::{BoardState, Move, MoveGenerator, Side};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::evaluate::Evaluator;
use crate::transposition::{TranspositionTable, EntryType};
use crate::zobrist::ZobristTable;

// 重导出 Difficulty 以便外部使用
pub use protocol::Difficulty;

/// AI 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub difficulty: Difficulty,
    pub max_depth: u8,
    pub time_limit_ms: u64,
    pub use_opening_book: bool,
    /// 置换表大小（MB）
    pub tt_size_mb: usize,
}

impl AiConfig {
    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        match difficulty {
            Difficulty::Easy => Self {
                difficulty,
                max_depth: 3,
                time_limit_ms: 1000,
                use_opening_book: false,
                tt_size_mb: 16,
            },
            Difficulty::Medium => Self {
                difficulty,
                max_depth: 4,
                time_limit_ms: 3000,
                use_opening_book: false,
                tt_size_mb: 32,
            },
            Difficulty::Hard => Self {
                difficulty,
                max_depth: 6,
                time_limit_ms: 5000,
                use_opening_book: true,
                tt_size_mb: 64,
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
    /// Zobrist 哈希表
    zobrist: ZobristTable,
    /// 置换表
    tt: TranspositionTable,
}

impl AiEngine {
    /// 创建新的 AI 引擎
    pub fn new(config: AiConfig) -> Self {
        let tt_size = config.tt_size_mb;
        Self {
            config,
            nodes_searched: 0,
            zobrist: ZobristTable::new(),
            tt: TranspositionTable::new(tt_size),
        }
    }

    /// 从难度创建
    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        Self::new(AiConfig::from_difficulty(difficulty))
    }

    /// 搜索最佳走法
    pub fn search(&mut self, state: &BoardState) -> Option<Move> {
        self.nodes_searched = 0;
        self.tt.new_search();
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

        // 计算当前局面哈希
        let hash = self.zobrist.hash(&state.board, state.current_turn);

        // 迭代加深搜索
        let mut best_move = moves[0];
        let mut best_score = i32::MIN;

        for depth in 1..=self.config.max_depth {
            if Instant::now() >= deadline {
                break;
            }

            let mut current_best_move = moves[0];
            let mut current_best_score = i32::MIN;

            // 尝试从置换表获取最佳走法，优先搜索
            let mut ordered_moves = moves.clone();
            if let Some(entry) = self.tt.probe(hash) {
                if let Some((fx, fy, tx, ty)) = entry.decode_move() {
                    // 将置换表中的最佳走法移到最前面
                    if let Some(pos) = ordered_moves.iter().position(|m| {
                        m.from.x == fx && m.from.y == fy && m.to.x == tx && m.to.y == ty
                    }) {
                        ordered_moves.swap(0, pos);
                    }
                }
            }

            for mv in &ordered_moves {
                if Instant::now() >= deadline {
                    break;
                }

                // 模拟走法
                let mut new_state = state.clone();
                let captured = new_state.board.get(mv.to);
                new_state.board.move_piece(mv.from, mv.to);
                new_state.switch_turn();

                // 增量更新哈希
                let new_hash = self.update_hash(hash, state, mv, captured.map(|p| p.piece_type));

                // Alpha-Beta 搜索
                let score = -self.alpha_beta(
                    &new_state,
                    new_hash,
                    depth,
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

            // 存储到置换表
            self.tt.store(
                hash,
                best_score,
                depth,
                EntryType::Exact,
                Some((best_move.from.x, best_move.from.y, best_move.to.x, best_move.to.y)),
            );
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

    /// 增量更新哈希值
    fn update_hash(
        &self,
        mut hash: u64,
        state: &BoardState,
        mv: &Move,
        captured: Option<protocol::PieceType>,
    ) -> u64 {
        // 移除原位置的棋子
        if let Some(piece) = state.board.get(mv.from) {
            hash ^= self.zobrist.piece_hash(piece.side, piece.piece_type, mv.from);
            // 添加到新位置
            hash ^= self.zobrist.piece_hash(piece.side, piece.piece_type, mv.to);
        }
        
        // 如果吃子，移除被吃的棋子
        if let Some(captured_type) = captured {
            let opponent = state.current_turn.opponent();
            hash ^= self.zobrist.piece_hash(opponent, captured_type, mv.to);
        }
        
        // 切换走子方
        hash ^= self.zobrist.side_hash();
        
        hash
    }

    /// Alpha-Beta 搜索
    fn alpha_beta(
        &mut self,
        state: &BoardState,
        hash: u64,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        deadline: &Instant,
    ) -> i32 {
        self.nodes_searched += 1;

        // 检查时间：超时时返回当前静态评估值
        if Instant::now() >= *deadline {
            return self.evaluate(state);
        }

        // 查询置换表
        if let Some(entry) = self.tt.probe(hash) {
            if entry.depth >= depth {
                match entry.entry_type {
                    EntryType::Exact => return entry.score as i32,
                    EntryType::LowerBound => {
                        if entry.score as i32 >= beta {
                            return entry.score as i32;
                        }
                    }
                    EntryType::UpperBound => {
                        if (entry.score as i32) <= alpha {
                            return entry.score as i32;
                        }
                    }
                }
            }
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

        let mut best_move: Option<Move> = None;
        let mut entry_type = EntryType::UpperBound;

        for mv in moves {
            // 模拟走法
            let mut new_state = state.clone();
            let captured = new_state.board.get(mv.to);
            new_state.board.move_piece(mv.from, mv.to);
            new_state.switch_turn();

            let new_hash = self.update_hash(hash, state, &mv, captured.map(|p| p.piece_type));
            let score = -self.alpha_beta(&new_state, new_hash, depth - 1, -beta, -alpha, deadline);

            if score >= beta {
                // Beta 剪枝
                self.tt.store(
                    hash,
                    score,
                    depth,
                    EntryType::LowerBound,
                    Some((mv.from.x, mv.from.y, mv.to.x, mv.to.y)),
                );
                return beta;
            }
            if score > alpha {
                alpha = score;
                best_move = Some(mv);
                entry_type = EntryType::Exact;
            }
        }

        // 存储到置换表
        self.tt.store(
            hash,
            alpha,
            depth,
            entry_type,
            best_move.map(|m| (m.from.x, m.from.y, m.to.x, m.to.y)),
        );

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

    /// 获取置换表统计
    pub fn tt_stats(&self) -> crate::transposition::TTStats {
        self.tt.stats()
    }

    /// 清空置换表
    pub fn clear_tt(&mut self) {
        self.tt.clear();
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
        assert_eq!(easy.tt_size_mb, 16);

        let medium = AiConfig::from_difficulty(Difficulty::Medium);
        assert_eq!(medium.max_depth, 4);
        assert_eq!(medium.tt_size_mb, 32);

        let hard = AiConfig::from_difficulty(Difficulty::Hard);
        assert_eq!(hard.max_depth, 6);
        assert!(hard.use_opening_book);
        assert_eq!(hard.tt_size_mb, 64);
    }

    #[test]
    fn test_tt_usage() {
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Medium);

        // 搜索后置换表应该有内容
        let _ = engine.search(&state);
        let stats = engine.tt_stats();
        
        assert!(stats.used > 0, "置换表应该有条目");
        println!("TT stats: {:?}", stats);
        println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
    }
}

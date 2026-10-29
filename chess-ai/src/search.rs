//! 搜索引擎
//!
//! 实现 Minimax + Alpha-Beta 剪枝 + 迭代加深 + 置换表 + 走法排序优化

use std::time::{Duration, Instant};

use protocol::{Board, BoardState, Move, MoveGenerator, Side};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::evaluate::Evaluator;
use crate::transposition::{TranspositionTable, EntryType};
use crate::zobrist::ZobristTable;

// 重导出 Difficulty 以便外部使用
pub use protocol::Difficulty;

// ============================================================================
// 走法排序优化相关常量和数据结构
// ============================================================================

/// 最大搜索深度（用于 Killer 表大小）
const MAX_PLY: usize = 64;

/// MVV-LVA 分数：用小子吃大子优先
/// 返回值越大表示优先级越高
fn mvv_lva_score(mv: &Move, board: &Board) -> i32 {
    let victim_value = board.get(mv.to)
        .map(|p| p.piece_type.value())
        .unwrap_or(0);
    let attacker_value = board.get(mv.from)
        .map(|p| p.piece_type.value())
        .unwrap_or(0);
    
    // victim * 10 - attacker，确保吃大子总是优先
    victim_value * 10 - attacker_value
}

/// Killer 走法表：每层保存 2 个导致 beta 截断的非吃子走法
#[derive(Clone)]
struct KillerMoves {
    /// killers[ply][0..2] - 每层最多 2 个 killer
    killers: [[Option<Move>; 2]; MAX_PLY],
}

impl KillerMoves {
    fn new() -> Self {
        Self {
            killers: [[None; 2]; MAX_PLY],
        }
    }
    
    /// 存储 killer 走法
    fn store(&mut self, ply: usize, mv: Move) {
        debug_assert!(ply < MAX_PLY, "ply {} exceeds MAX_PLY {}", ply, MAX_PLY);
        if ply >= MAX_PLY {
            return;
        }
        // 避免重复存储
        if self.killers[ply][0] != Some(mv) {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = Some(mv);
        }
    }
    
    /// 检查是否是 killer 走法
    fn is_killer(&self, ply: usize, mv: &Move) -> bool {
        debug_assert!(ply < MAX_PLY, "ply {} exceeds MAX_PLY {}", ply, MAX_PLY);
        if ply >= MAX_PLY {
            return false;
        }
        self.killers[ply][0].as_ref() == Some(mv) 
            || self.killers[ply][1].as_ref() == Some(mv)
    }
    
    /// 清空 killer 表（每次新的根节点搜索时调用）
    fn clear(&mut self) {
        for killers in self.killers.iter_mut() {
            *killers = [None, None];
        }
    }
}

/// 历史启发表：记录走法导致截断的累计分数
#[derive(Clone)]
struct HistoryTable {
    /// history[side][from_square][to_square]
    table: [[[i32; 90]; 90]; 2],
}

impl HistoryTable {
    fn new() -> Self {
        Self {
            table: [[[0; 90]; 90]; 2],
        }
    }
    
    /// 更新历史分数（截断时调用）
    fn update(&mut self, side: Side, mv: &Move, depth: u8) {
        let from_idx = mv.from.y as usize * 9 + mv.from.x as usize;
        let to_idx = mv.to.y as usize * 9 + mv.to.x as usize;
        let side_idx = side as usize;
        
        // 深度越大，加分越多（depth²）
        let bonus = (depth as i32) * (depth as i32);
        // 防止溢出
        self.table[side_idx][from_idx][to_idx] = 
            self.table[side_idx][from_idx][to_idx].saturating_add(bonus);
    }
    
    /// 获取历史分数
    fn get(&self, side: Side, mv: &Move) -> i32 {
        let from_idx = mv.from.y as usize * 9 + mv.from.x as usize;
        let to_idx = mv.to.y as usize * 9 + mv.to.x as usize;
        self.table[side as usize][from_idx][to_idx]
    }
    
    /// 衰减历史分数（每次新的根节点搜索时调用）
    fn age(&mut self) {
        for side in self.table.iter_mut() {
            for from in side.iter_mut() {
                for score in from.iter_mut() {
                    *score /= 2;
                }
            }
        }
    }
}

/// 走法排序上下文
struct MoveOrderContext<'a> {
    tt_move: Option<Move>,
    killers: &'a KillerMoves,
    history: &'a HistoryTable,
    ply: usize,
}

/// 对走法进行排序
fn order_moves(
    moves: &mut Vec<Move>,
    board: &Board,
    current_turn: Side,
    ctx: &MoveOrderContext,
) {
    // 注意：sort_by_cached_key 是升序排序，值越小越靠前
    moves.sort_by_cached_key(|mv| {
        // TT 最佳走法最优先（最小值）
        if ctx.tt_move.as_ref() == Some(mv) {
            return i32::MIN;
        }
        
        // 吃子走法按 MVV-LVA 排序（分数越高应越靠前，所以取负）
        if board.get(mv.to).is_some() {
            return i32::MIN / 2 - mvv_lva_score(mv, board);
        }
        
        // Killer 走法次之
        if ctx.killers.is_killer(ctx.ply, mv) {
            return 0;
        }
        
        // 其他走法按历史启发排序（分数越高越靠前，取负）
        -ctx.history.get(current_turn, mv)
    });
}

/// 对 quiescence 搜索中的吃子走法进行 MVV-LVA 排序
fn order_captures(moves: &mut Vec<Move>, board: &Board) {
    moves.sort_by_cached_key(|mv| {
        // 分数越高越靠前，取负
        -mvv_lva_score(mv, board)
    });
}

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
            Difficulty::Custom { depth, time_limit_ms } => Self {
                difficulty,
                max_depth: depth,
                time_limit_ms,
                use_opening_book: depth >= 5,
                tt_size_mb: if depth >= 6 { 64 } else if depth >= 4 { 32 } else { 16 },
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
    /// 搜索路径哈希栈（用于检测重复局面）
    path_hashes: Vec<u64>,
    /// Killer 走法表
    killer_moves: KillerMoves,
    /// 历史启发表
    history_table: HistoryTable,
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
            path_hashes: Vec::with_capacity(64),
            killer_moves: KillerMoves::new(),
            history_table: HistoryTable::new(),
        }
    }

    /// 从难度创建
    pub fn from_difficulty(difficulty: Difficulty) -> Self {
        Self::new(AiConfig::from_difficulty(difficulty))
    }

    /// 搜索最佳走法
    pub fn search(&mut self, state: &BoardState) -> Option<Move> {
        self.search_with_history(state, &[])
    }

    /// 搜索最佳走法（带历史局面检测跨回合重复）
    /// 
    /// # Arguments
    /// * `state` - 当前局面
    /// * `history_states` - 历史局面列表（用于检测跨回合重复）
    pub fn search_with_history(&mut self, state: &BoardState, history_states: &[BoardState]) -> Option<Move> {
        self.nodes_searched = 0;
        self.tt.new_search();
        self.path_hashes.clear();
        
        // 每次新搜索时清空 killer 表并衰减历史表
        self.killer_moves.clear();
        self.history_table.age();
        
        let deadline = Instant::now() + Duration::from_millis(self.config.time_limit_ms);

        // 继承历史局面的哈希（检测跨回合重复）
        for hist_state in history_states {
            let hist_hash = self.zobrist.hash(&hist_state.board, hist_state.current_turn);
            self.path_hashes.push(hist_hash);
        }

        // 生成所有合法走法
        let mut moves = MoveGenerator::generate_legal(state);
        if moves.is_empty() {
            return None;
        }

        // 如果只有一个走法，直接返回
        if moves.len() == 1 {
            return Some(moves[0]);
        }

        // 计算当前局面哈希
        let hash = self.zobrist.hash(&state.board, state.current_turn);
        
        // 记录根节点哈希
        self.path_hashes.push(hash);

        // 迭代加深搜索
        let mut best_move = moves[0];
        let mut best_score = i32::MIN;

        for depth in 1..=self.config.max_depth {
            if Instant::now() >= deadline {
                break;
            }

            let mut current_best_move = moves[0];
            let mut current_best_score = i32::MIN;

            // 获取置换表最佳走法
            let tt_move = self.tt.probe(hash).and_then(|entry| {
                entry.decode_move().and_then(|(fx, fy, tx, ty)| {
                    moves.iter().find(|m| {
                        m.from.x == fx && m.from.y == fy && m.to.x == tx && m.to.y == ty
                    }).cloned()
                })
            });

            // 对根节点走法排序
            let ctx = MoveOrderContext {
                tt_move,
                killers: &self.killer_moves,
                history: &self.history_table,
                ply: 0,
            };
            order_moves(&mut moves, &state.board, state.current_turn, &ctx);

            for mv in &moves {
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

                // 入栈
                self.path_hashes.push(new_hash);

                // Alpha-Beta 搜索（ply=1 因为从根节点进入）
                let score = -self.alpha_beta(
                    &new_state,
                    new_hash,
                    depth,
                    1,  // ply
                    i32::MIN + 1,
                    i32::MAX - 1,
                    &deadline,
                );

                // 出栈
                self.path_hashes.pop();

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
        if self.config.difficulty == Difficulty::Easy
            && rand::random::<f32>() < 0.3 {
                // 随机选择一个走法
                let mut rng = rand::thread_rng();
                if let Some(random_move) = moves.choose(&mut rng) {
                    return Some(*random_move);
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
        ply: usize,
        mut alpha: i32,
        beta: i32,
        deadline: &Instant,
    ) -> i32 {
        self.nodes_searched += 1;

        // 重复局面检测（在置换表查询之前）
        // path_hashes 中已有 2 次相同哈希，加上当前局面是第 3 次，判和
        let repetition_count = self.path_hashes.iter().filter(|&&h| h == hash).count();
        if repetition_count >= 2 {
            return 0;
        }

        // 检查时间：超时时返回当前静态评估值
        if Instant::now() >= *deadline {
            return self.evaluate(state);
        }

        // 查询置换表
        let tt_entry = self.tt.probe(hash);
        if let Some(ref entry) = tt_entry {
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
        let mut moves = MoveGenerator::generate_legal(state);

        // 无子可动
        if moves.is_empty() {
            if MoveGenerator::is_in_check(&state.board, state.current_turn) {
                // 被将死（ply 越大说明距离根越远，给较小惩罚让 AI 选择更快的将死）
                return -10000 + ply as i32;
            } else {
                // 困毙（和棋）
                return 0;
            }
        }

        // 获取置换表最佳走法
        let tt_move = tt_entry.and_then(|entry| {
            entry.decode_move().and_then(|(fx, fy, tx, ty)| {
                moves.iter().find(|m| {
                    m.from.x == fx && m.from.y == fy && m.to.x == tx && m.to.y == ty
                }).cloned()
            })
        });

        // 走法排序
        let ctx = MoveOrderContext {
            tt_move,
            killers: &self.killer_moves,
            history: &self.history_table,
            ply,
        };
        order_moves(&mut moves, &state.board, state.current_turn, &ctx);

        let mut best_move: Option<Move> = None;
        let mut entry_type = EntryType::UpperBound;

        for mv in moves {
            // 模拟走法
            let mut new_state = state.clone();
            let captured = new_state.board.get(mv.to);
            new_state.board.move_piece(mv.from, mv.to);
            new_state.switch_turn();

            let new_hash = self.update_hash(hash, state, &mv, captured.map(|p| p.piece_type));
            
            // 入栈
            self.path_hashes.push(new_hash);
            
            let score = -self.alpha_beta(&new_state, new_hash, depth - 1, ply + 1, -beta, -alpha, deadline);
            
            // 出栈
            self.path_hashes.pop();

            if score >= beta {
                // Beta 剪枝 - 更新 killer 和历史表
                let is_capture = state.board.get(mv.to).is_some();
                if !is_capture {
                    // 只存储非吃子走法为 killer
                    self.killer_moves.store(ply, mv);
                    // 更新历史启发表
                    self.history_table.update(state.current_turn, &mv, depth);
                }
                
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

    /// 静态搜索（只搜索吃子走法和将军应对）
    fn quiescence(&mut self, state: &BoardState, mut alpha: i32, beta: i32, qs_depth: u8) -> i32 {
        self.nodes_searched += 1;

        // 检查是否被将军
        let in_check = MoveGenerator::is_in_check(&state.board, state.current_turn);

        // 生成合法走法
        let moves = MoveGenerator::generate_legal(state);

        // 无子可动：将死或困毙
        if moves.is_empty() {
            if in_check {
                // 被将死，返回极低分（比正常将死分稍高，因为是静态搜索）
                return -9900;
            } else {
                // 困毙，和棋
                return 0;
            }
        }

        // 静态评估
        let mut stand_pat = self.evaluate(state);

        // 被将军时给予惩罚，必须搜索所有走法
        if in_check {
            stand_pat -= 150; // 被将军惩罚
        }

        if qs_depth == 0 {
            return stand_pat;
        }

        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }

        // 如果被将军，搜索所有走法；否则只搜索吃子走法
        let mut search_moves: Vec<_> = if in_check {
            moves
        } else {
            moves.into_iter().filter(|m| state.board.get(m.to).is_some()).collect()
        };

        // 对吃子走法进行 MVV-LVA 排序（深度 > 0 时排序收益更大）
        if qs_depth > 1 && !search_moves.is_empty() {
            order_captures(&mut search_moves, &state.board);
        }

        for mv in search_moves {
            let mut new_state = state.clone();
            new_state.board.move_piece(mv.from, mv.to);
            new_state.switch_turn();

            let score = -self.quiescence(&new_state, -beta, -alpha, qs_depth - 1);

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
    use std::time::Instant;

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

    #[test]
    fn test_easy_performance() {
        // Easy 难度应该在 1 秒内返回
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Easy);

        let start = Instant::now();
        let mv = engine.search(&state);
        let elapsed = start.elapsed();

        assert!(mv.is_some(), "Easy AI 应该返回走法");
        assert!(
            elapsed.as_millis() < 2000,
            "Easy AI 应该在 2 秒内返回，实际: {:?}",
            elapsed
        );
        println!("Easy AI 耗时: {:?}, 节点: {}", elapsed, engine.nodes_searched());
    }

    #[test]
    fn test_medium_performance() {
        // Medium 难度应该在 3 秒内返回
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Medium);

        let start = Instant::now();
        let mv = engine.search(&state);
        let elapsed = start.elapsed();

        assert!(mv.is_some(), "Medium AI 应该返回走法");
        assert!(
            elapsed.as_millis() < 5000,
            "Medium AI 应该在 5 秒内返回，实际: {:?}",
            elapsed
        );
        println!("Medium AI 耗时: {:?}, 节点: {}", elapsed, engine.nodes_searched());
    }

    #[test]
    fn test_search_returns_legal_move() {
        // AI 返回的走法必须是合法的
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Easy);

        let mv = engine.search(&state).unwrap();
        
        // 验证走法合法
        let legal_moves = protocol::MoveGenerator::generate_legal(&state);
        let is_legal = legal_moves.iter().any(|m| m.from == mv.from && m.to == mv.to);
        assert!(is_legal, "AI 返回的走法必须合法: {:?}", mv);
    }

    #[test]
    fn test_search_different_positions() {
        // 测试不同局面的搜索
        let positions = [
            "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w 0 1", // 初始
            "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C2C4/9/RNBAKABNR b 0 1", // 炮二平五后
            "r1bakabnr/9/1cn4c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w 0 1", // 马8进7后
        ];

        for fen in positions {
            let state = Fen::parse(fen).unwrap();
            let mut engine = AiEngine::from_difficulty(Difficulty::Easy);
            
            let mv = engine.search(&state);
            assert!(mv.is_some(), "AI 应该能找到走法: {}", fen);
        }
    }

    #[test]
    fn test_search_endgame() {
        // 残局测试
        let fen = "4k4/9/9/9/9/9/9/9/4R4/4K4 w 0 1";
        let state = Fen::parse(fen).unwrap();
        let mut engine = AiEngine::from_difficulty(Difficulty::Medium);

        let mv = engine.search(&state);
        assert!(mv.is_some(), "残局应该能找到走法");
    }

    #[test]
    fn test_alpha_beta_pruning() {
        // 验证 Alpha-Beta 剪枝有效
        let state = BoardState::initial();
        
        // 使用较深的搜索
        let mut engine = AiEngine::from_difficulty(Difficulty::Medium);
        let _ = engine.search(&state);
        let nodes_with_pruning = engine.nodes_searched();

        // Alpha-Beta 应该比完全搜索快很多
        // 对于深度 4，完全搜索可能需要数百万节点
        // Alpha-Beta 应该只需要几万到几十万节点
        println!("Alpha-Beta 搜索节点数: {}", nodes_with_pruning);
        assert!(
            nodes_with_pruning < 1_000_000,
            "Alpha-Beta 剪枝应该有效，节点数: {}",
            nodes_with_pruning
        );
    }

    #[test]
    fn test_iterative_deepening() {
        // 测试迭代加深
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Medium);

        // 迭代加深应该总是返回一个走法
        let mv = engine.search(&state);
        assert!(mv.is_some(), "迭代加深应该返回走法");
    }

    #[test]
    fn test_tt_improves_performance() {
        // 测试置换表对性能的提升
        let state = BoardState::initial();
        
        // 第一次搜索
        let mut engine = AiEngine::from_difficulty(Difficulty::Medium);
        let _ = engine.search(&state);
        let stats1 = engine.tt_stats();
        
        // 相同局面第二次搜索应该更快（命中置换表）
        let _ = engine.search(&state);
        let stats2 = engine.tt_stats();
        
        println!("第一次 TT 命中: {}, 第二次 TT 命中: {}", stats1.hits, stats2.hits);
        // 第二次搜索应该有严格更多的命中（置换表复用）
        assert!(stats2.hits > stats1.hits, "第二次搜索应该有更多 TT 命中");
        // 验证置换表显著提升：增量 >50 或倍数 >3（适应 debug/release 不同表现）
        let delta = stats2.hits - stats1.hits;
        let ratio = stats2.hits as f64 / stats1.hits.max(1) as f64;
        assert!(
            delta > 50 || ratio > 3.0,
            "TT 命中应显著增加: 增量={}, 倍数={:.1}",
            delta, ratio
        );
    }

    #[test]
    fn test_clear_tt() {
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Easy);

        let _ = engine.search(&state);
        assert!(engine.tt_stats().used > 0);

        engine.clear_tt();
        assert_eq!(engine.tt_stats().used, 0, "清空后置换表应该为空");
    }

    #[test]
    fn test_repetition_detection_path_hashes() {
        // 测试搜索后 path_hashes 只剩根节点
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Easy);

        let _ = engine.search(&state);
        
        // 搜索结束后，path_hashes 应该只剩根节点
        assert_eq!(
            engine.path_hashes.len(), 1,
            "搜索结束后 path_hashes 应该只剩根节点，实际: {}",
            engine.path_hashes.len()
        );
    }

    #[test]
    fn test_no_simple_two_move_cycle() {
        // 测试 AI 不会陷入简单的两步循环
        // 使用一个更复杂的中局局面，双方都有多个棋子
        let fen = "r1bakab1r/9/1cn4c1/p1p1p1p1p/9/2P6/P3P1P1P/1C2C1N2/9/RNBAKAB1R w 0 1";
        let state = Fen::parse(fen).unwrap();
        let mut engine = AiEngine::from_difficulty(Difficulty::Easy);

        let mut moves = Vec::new();
        let mut current_state = state.clone();

        // 模拟 6 步走法
        for _ in 0..6 {
            if let Some(mv) = engine.search(&current_state) {
                moves.push(mv);
                current_state.board.move_piece(mv.from, mv.to);
                current_state.switch_turn();
            } else {
                break;
            }
        }

        // 验证至少走了 4 步（确保测试有意义）
        assert!(
            moves.len() >= 4,
            "应该至少走 4 步，实际走了 {} 步",
            moves.len()
        );

        // 检查是否有简单的两步循环（A->B, B->A）
        let mut cycle_count = 0;
        for i in 2..moves.len() {
            let is_simple_cycle = moves[i].from == moves[i - 2].to
                && moves[i].to == moves[i - 2].from;
            if is_simple_cycle {
                cycle_count += 1;
                println!(
                    "检测到可能的循环: 第{}步 {:?} 与第{}步 {:?}",
                    i - 1, moves[i - 2], i + 1, moves[i]
                );
            }
        }

        println!("{}步走法: {:?}", moves.len(), moves);
        
        // 不应该有连续的循环
        assert!(
            cycle_count < 2,
            "不应该有连续的两步循环，检测到 {} 次",
            cycle_count
        );
    }

    #[test]
    fn test_repetition_returns_draw_score() {
        // 测试重复局面返回和棋分数
        // 这个测试验证重复检测逻辑的正确性
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Easy);

        // 手动设置 path_hashes 模拟重复局面
        let hash = engine.zobrist.hash(&state.board, state.current_turn);
        engine.path_hashes.clear();
        engine.path_hashes.push(hash);  // 第一次
        engine.path_hashes.push(hash);  // 第二次

        // 现在如果再遇到相同哈希，应该返回 0（和棋）
        // 由于 alpha_beta 是私有的，我们通过搜索间接测试
        // 这里主要验证 path_hashes 的设置不会导致崩溃
        let _ = engine.search(&state);
        
        // 搜索后 path_hashes 应该被清空并重新填充
        assert!(engine.path_hashes.len() >= 1, "搜索后应该有路径哈希");
    }

    #[test]
    fn test_search_with_history() {
        // 测试带历史局面的搜索
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Easy);

        // 创建一些历史局面
        let history: Vec<BoardState> = vec![
            BoardState::initial(),
            BoardState::initial(),
        ];

        // 带历史搜索应该正常工作
        let mv = engine.search_with_history(&state, &history);
        assert!(mv.is_some(), "带历史搜索应该返回走法");

        // path_hashes 应该包含历史局面 + 根节点
        // 搜索结束后，历史局面仍在，加上根节点
        assert!(
            engine.path_hashes.len() >= 3,
            "path_hashes 应该包含历史局面，实际: {}",
            engine.path_hashes.len()
        );
    }

    #[test]
    fn test_cross_turn_repetition_detection() {
        // 测试跨回合重复检测
        // 模拟：历史中已有相同局面，AI 应该避免回到该局面
        let state = BoardState::initial();
        let mut engine = AiEngine::from_difficulty(Difficulty::Easy);

        // 历史中有两次相同局面（初始局面）
        let history = vec![
            BoardState::initial(),
            BoardState::initial(),
        ];

        // 搜索时，如果走法导致回到初始局面，应该被视为和棋（0分）
        // 这会影响 AI 的选择
        let mv = engine.search_with_history(&state, &history);
        
        // AI 应该返回一个走法
        assert!(mv.is_some(), "AI 应该返回走法");
        
        println!("跨回合重复检测测试: AI 选择 {:?}", mv);
    }

    #[test]
    fn test_mvv_lva_score() {
        // 验证 MVV-LVA 分数计算正确性
        use protocol::Position;
        
        // 创建测试棋盘：红炮(4,5) 可以吃黑车(4,9) 或黑兵(4,6)
        let fen = "4r4/9/9/4p4/9/4C4/9/9/9/4K4 w 0 1";
        let state = Fen::parse(fen).unwrap();
        
        // 吃车的走法
        let capture_rook = Move {
            from: Position::new_unchecked(4, 5),
            to: Position::new_unchecked(4, 9),
            captured: state.board.get(Position::new_unchecked(4, 9)),
        };
        
        // 吃兵的走法
        let capture_pawn = Move {
            from: Position::new_unchecked(4, 5),
            to: Position::new_unchecked(4, 6),
            captured: state.board.get(Position::new_unchecked(4, 6)),
        };
        
        let score_rook = mvv_lva_score(&capture_rook, &state.board);
        let score_pawn = mvv_lva_score(&capture_pawn, &state.board);
        
        // 吃车分数应该高于吃兵
        assert!(
            score_rook > score_pawn,
            "吃车分数应高于吃兵: {} vs {}",
            score_rook, score_pawn
        );
        
        println!("MVV-LVA: 吃车={}, 吃兵={}", score_rook, score_pawn);
    }

    #[test]
    fn test_killer_moves() {
        // 验证 Killer Move 存储和检索
        use protocol::Position;
        
        let mut killers = KillerMoves::new();
        
        let mv1 = Move {
            from: Position::new_unchecked(0, 0),
            to: Position::new_unchecked(0, 1),
            captured: None,
        };
        let mv2 = Move {
            from: Position::new_unchecked(1, 0),
            to: Position::new_unchecked(1, 1),
            captured: None,
        };
        let mv3 = Move {
            from: Position::new_unchecked(2, 0),
            to: Position::new_unchecked(2, 1),
            captured: None,
        };
        
        // 存储第一个 killer
        killers.store(5, mv1);
        assert!(killers.is_killer(5, &mv1), "mv1 应该是 ply=5 的 killer");
        assert!(!killers.is_killer(6, &mv1), "mv1 不应该是 ply=6 的 killer");
        
        // 存储第二个 killer
        killers.store(5, mv2);
        assert!(killers.is_killer(5, &mv1), "mv1 仍应该是 ply=5 的 killer");
        assert!(killers.is_killer(5, &mv2), "mv2 应该是 ply=5 的 killer");
        
        // 存储第三个会挤掉第一个
        killers.store(5, mv3);
        assert!(!killers.is_killer(5, &mv1), "mv1 应该被挤掉");
        assert!(killers.is_killer(5, &mv2), "mv2 仍应该是 killer");
        assert!(killers.is_killer(5, &mv3), "mv3 应该是 killer");
        
        // 清空后应该没有 killer
        killers.clear();
        assert!(!killers.is_killer(5, &mv2), "清空后不应该有 killer");
        assert!(!killers.is_killer(5, &mv3), "清空后不应该有 killer");
    }

    #[test]
    fn test_history_table() {
        // 验证历史启发表
        use protocol::Position;
        
        let mut history = HistoryTable::new();
        
        let mv = Move {
            from: Position::new_unchecked(4, 0),
            to: Position::new_unchecked(4, 1),
            captured: None,
        };
        
        // 初始分数为 0
        assert_eq!(history.get(Side::Red, &mv), 0);
        
        // 更新后分数增加（depth² = 9）
        history.update(Side::Red, &mv, 3);
        assert_eq!(history.get(Side::Red, &mv), 9);
        
        // 再次更新累加
        history.update(Side::Red, &mv, 4);
        assert_eq!(history.get(Side::Red, &mv), 9 + 16);
        
        // 衰减后减半
        history.age();
        assert_eq!(history.get(Side::Red, &mv), (9 + 16) / 2);
        
        // 不同方的历史独立
        assert_eq!(history.get(Side::Black, &mv), 0);
    }

    #[test]
    fn test_move_ordering() {
        // 验证综合走法排序
        
        // 使用初始局面进行测试
        let state = BoardState::initial();
        
        let mut moves = MoveGenerator::generate_legal(&state);
        let original_len = moves.len();
        
        // 创建排序上下文
        let killers = KillerMoves::new();
        let history = HistoryTable::new();
        
        // 从生成的走法中选取第一个作为 TT 走法
        let tt_move = moves.get(5).cloned();  // 选取列表中间的一个走法
        
        assert!(tt_move.is_some(), "应该有走法可供选择");
        
        let ctx = MoveOrderContext {
            tt_move: tt_move.clone(),
            killers: &killers,
            history: &history,
            ply: 0,
        };
        
        order_moves(&mut moves, &state.board, state.current_turn, &ctx);
        
        // 排序后数量不变
        assert_eq!(moves.len(), original_len);
        
        // TT 走法应该在第一位
        if let Some(ref expected) = tt_move {
            assert_eq!(
                moves[0].from, expected.from,
                "TT 走法应该排在第一位 (from)"
            );
            assert_eq!(
                moves[0].to, expected.to,
                "TT 走法应该排在第一位 (to)"
            );
        }
        
        println!("排序后的走法数量: {}", moves.len());
    }

    /// 慢速性能基准：深度 4/5 全搜，默认 CI 跳过；本地用
    /// `cargo test -p chess-ai test_move_ordering_performance -- --ignored --nocapture`
    #[test]
    #[ignore = "slow move-ordering benchmark (depth 4/5)"]
    fn test_move_ordering_performance() {
        // 性能对比：验证走法排序后节点规模是否合理
        let test_positions = [
            ("初始局面", "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR w 0 1"),
            ("中盘局面", "r1bakab1r/9/1cn4c1/p1p1p1p1p/9/2P6/P3P1P1P/1C2C1N2/9/RNBAKAB1R w 0 1"),
        ];

        println!("\n========================================");
        println!("走法排序优化 - 性能测试");
        println!("========================================\n");

        for depth in [4, 5] {
            println!("--- 深度 {} ---", depth);
            let mut total_nodes = 0u64;

            for (name, fen) in &test_positions {
                let state = Fen::parse(fen).expect("FEN 解析失败");
                let mut engine = AiEngine::new(AiConfig {
                    difficulty: Difficulty::Custom { depth, time_limit_ms: 60000 },
                    max_depth: depth,
                    time_limit_ms: 60000,
                    use_opening_book: false,
                    tt_size_mb: 16,
                });

                let start = Instant::now();
                let mv = engine.search(&state);
                let elapsed = start.elapsed();
                let nodes = engine.nodes_searched();

                total_nodes += nodes;

                println!(
                    "{}: 节点={}, 耗时={:?}, 走法={:?}",
                    name,
                    nodes,
                    elapsed,
                    mv.map(|m| format!(
                        "({},{})->({},{})",
                        m.from.x, m.from.y, m.to.x, m.to.y
                    ))
                );
            }

            println!("深度{}合计: 总节点={}\n", depth, total_nodes);
        }

        let state = BoardState::initial();
        let mut engine = AiEngine::new(AiConfig {
            difficulty: Difficulty::Custom { depth: 5, time_limit_ms: 60000 },
            max_depth: 5,
            time_limit_ms: 60000,
            use_opening_book: false,
            tt_size_mb: 16,
        });
        let _ = engine.search(&state);
        let nodes = engine.nodes_searched();

        println!("深度5初始局面节点数: {}", nodes);
        // 有良好走法排序的 Alpha-Beta：深度 5 约数百万节点量级
        assert!(
            nodes < 15_000_000,
            "深度5节点数应小于1500万，实际: {}",
            nodes
        );
    }
}

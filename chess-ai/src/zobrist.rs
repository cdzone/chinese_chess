//! Zobrist 哈希
//!
//! 用于快速计算棋局的哈希值，支持增量更新

use protocol::{Board, PieceType, Position, Side};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Zobrist 哈希表
/// 
/// 使用随机数为每个位置的每种棋子生成唯一的哈希值
pub struct ZobristTable {
    /// 棋子哈希值 [side][piece_type][position]
    /// side: 0=Red, 1=Black
    /// piece_type: 0-6 对应 7 种棋子
    /// position: 0-89 对应 90 个位置
    pieces: [[[u64; 90]; 7]; 2],
    /// 当前走子方哈希值
    side_to_move: u64,
}

impl ZobristTable {
    /// 创建新的 Zobrist 表（使用固定种子保证确定性）
    pub fn new() -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(0xDEADBEEF_CAFE_1234);
        
        let mut pieces = [[[0u64; 90]; 7]; 2];
        for side in 0..2 {
            for piece in 0..7 {
                for pos in 0..90 {
                    pieces[side][piece][pos] = rng.gen();
                }
            }
        }
        
        Self {
            pieces,
            side_to_move: rng.gen(),
        }
    }
    
    /// 计算棋盘的完整哈希值
    pub fn hash(&self, board: &Board, current_turn: Side) -> u64 {
        let mut hash = 0u64;
        
        for (pos, piece) in board.all_pieces() {
            hash ^= self.piece_hash(piece.side, piece.piece_type, pos);
        }
        
        if current_turn == Side::Black {
            hash ^= self.side_to_move;
        }
        
        hash
    }
    
    /// 获取棋子的哈希值
    #[inline]
    pub fn piece_hash(&self, side: Side, piece_type: PieceType, pos: Position) -> u64 {
        let side_idx = match side {
            Side::Red => 0,
            Side::Black => 1,
        };
        let piece_idx = piece_type_to_index(piece_type);
        let pos_idx = pos.to_index();
        self.pieces[side_idx][piece_idx][pos_idx]
    }
    
    /// 获取走子方切换的哈希值
    #[inline]
    pub fn side_hash(&self) -> u64 {
        self.side_to_move
    }
}

impl Default for ZobristTable {
    fn default() -> Self {
        Self::new()
    }
}

/// 将棋子类型转换为索引
#[inline]
fn piece_type_to_index(piece_type: PieceType) -> usize {
    match piece_type {
        PieceType::King => 0,
        PieceType::Advisor => 1,
        PieceType::Bishop => 2,
        PieceType::Knight => 3,
        PieceType::Rook => 4,
        PieceType::Cannon => 5,
        PieceType::Pawn => 6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::BoardState;
    
    #[test]
    fn test_zobrist_deterministic() {
        let table1 = ZobristTable::new();
        let table2 = ZobristTable::new();
        
        let state = BoardState::initial();
        let hash1 = table1.hash(&state.board, state.current_turn);
        let hash2 = table2.hash(&state.board, state.current_turn);
        
        assert_eq!(hash1, hash2, "Zobrist 哈希应该是确定性的");
    }
    
    #[test]
    fn test_zobrist_different_positions() {
        let table = ZobristTable::new();
        
        let state1 = BoardState::initial();
        let hash1 = table.hash(&state1.board, state1.current_turn);
        
        // 走一步棋
        let mut state2 = state1.clone();
        let from = Position::new_unchecked(1, 2); // 红炮
        let to = Position::new_unchecked(1, 4);
        state2.board.move_piece(from, to);
        state2.switch_turn();
        let hash2 = table.hash(&state2.board, state2.current_turn);
        
        assert_ne!(hash1, hash2, "不同局面应该有不同的哈希值");
    }
    
    #[test]
    fn test_zobrist_side_matters() {
        let table = ZobristTable::new();
        let board = Board::initial();
        
        let hash_red = table.hash(&board, Side::Red);
        let hash_black = table.hash(&board, Side::Black);
        
        assert_ne!(hash_red, hash_black, "不同走子方应该有不同的哈希值");
    }
}

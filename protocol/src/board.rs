//! 棋盘状态

use serde::{Deserialize, Serialize};

use crate::piece::{Piece, PieceType, Position, Side};
use crate::constants::{BOARD_WIDTH, BOARD_HEIGHT};

/// 棋盘
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
    /// 9x10 棋盘，索引为 y * 9 + x，使用 Vec 以支持 serde
    squares: Vec<Option<Piece>>,
}

impl Board {
    /// 创建空棋盘
    pub fn empty() -> Self {
        Self {
            squares: vec![None; 90],
        }
    }

    /// 创建初始棋盘
    pub fn initial() -> Self {
        let mut board = Self::empty();
        
        // 红方（下方，y=0 开始）
        // 第一行：车马相仕帅仕相马车
        board.set(Position::new_unchecked(0, 0), Some(Piece::new(PieceType::Rook, Side::Red)));
        board.set(Position::new_unchecked(1, 0), Some(Piece::new(PieceType::Knight, Side::Red)));
        board.set(Position::new_unchecked(2, 0), Some(Piece::new(PieceType::Bishop, Side::Red)));
        board.set(Position::new_unchecked(3, 0), Some(Piece::new(PieceType::Advisor, Side::Red)));
        board.set(Position::new_unchecked(4, 0), Some(Piece::new(PieceType::King, Side::Red)));
        board.set(Position::new_unchecked(5, 0), Some(Piece::new(PieceType::Advisor, Side::Red)));
        board.set(Position::new_unchecked(6, 0), Some(Piece::new(PieceType::Bishop, Side::Red)));
        board.set(Position::new_unchecked(7, 0), Some(Piece::new(PieceType::Knight, Side::Red)));
        board.set(Position::new_unchecked(8, 0), Some(Piece::new(PieceType::Rook, Side::Red)));
        
        // 红方炮
        board.set(Position::new_unchecked(1, 2), Some(Piece::new(PieceType::Cannon, Side::Red)));
        board.set(Position::new_unchecked(7, 2), Some(Piece::new(PieceType::Cannon, Side::Red)));
        
        // 红方兵
        for x in (0..9).step_by(2) {
            board.set(Position::new_unchecked(x, 3), Some(Piece::new(PieceType::Pawn, Side::Red)));
        }
        
        // 黑方（上方，y=9 开始）
        // 第一行：车马象士将士象马车
        board.set(Position::new_unchecked(0, 9), Some(Piece::new(PieceType::Rook, Side::Black)));
        board.set(Position::new_unchecked(1, 9), Some(Piece::new(PieceType::Knight, Side::Black)));
        board.set(Position::new_unchecked(2, 9), Some(Piece::new(PieceType::Bishop, Side::Black)));
        board.set(Position::new_unchecked(3, 9), Some(Piece::new(PieceType::Advisor, Side::Black)));
        board.set(Position::new_unchecked(4, 9), Some(Piece::new(PieceType::King, Side::Black)));
        board.set(Position::new_unchecked(5, 9), Some(Piece::new(PieceType::Advisor, Side::Black)));
        board.set(Position::new_unchecked(6, 9), Some(Piece::new(PieceType::Bishop, Side::Black)));
        board.set(Position::new_unchecked(7, 9), Some(Piece::new(PieceType::Knight, Side::Black)));
        board.set(Position::new_unchecked(8, 9), Some(Piece::new(PieceType::Rook, Side::Black)));
        
        // 黑方炮
        board.set(Position::new_unchecked(1, 7), Some(Piece::new(PieceType::Cannon, Side::Black)));
        board.set(Position::new_unchecked(7, 7), Some(Piece::new(PieceType::Cannon, Side::Black)));
        
        // 黑方卒
        for x in (0..9).step_by(2) {
            board.set(Position::new_unchecked(x, 6), Some(Piece::new(PieceType::Pawn, Side::Black)));
        }
        
        board
    }

    /// 获取指定位置的棋子
    pub fn get(&self, pos: Position) -> Option<Piece> {
        if pos.is_valid() {
            self.squares[pos.to_index()]
        } else {
            None
        }
    }

    /// 设置指定位置的棋子
    pub fn set(&mut self, pos: Position, piece: Option<Piece>) {
        if pos.is_valid() {
            self.squares[pos.to_index()] = piece;
        }
    }

    /// 移动棋子（不检查规则）
    pub fn move_piece(&mut self, from: Position, to: Position) -> Option<Piece> {
        let piece = self.get(from);
        let captured = self.get(to);
        self.set(from, None);
        self.set(to, piece);
        captured
    }

    /// 查找指定阵营的将/帅位置
    pub fn find_king(&self, side: Side) -> Option<Position> {
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let pos = Position::new_unchecked(x as u8, y as u8);
                if let Some(piece) = self.get(pos) {
                    if piece.piece_type == PieceType::King && piece.side == side {
                        return Some(pos);
                    }
                }
            }
        }
        None
    }

    /// 获取指定阵营的所有棋子位置
    pub fn pieces(&self, side: Side) -> Vec<(Position, Piece)> {
        let mut result = Vec::new();
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let pos = Position::new_unchecked(x as u8, y as u8);
                if let Some(piece) = self.get(pos) {
                    if piece.side == side {
                        result.push((pos, piece));
                    }
                }
            }
        }
        result
    }

    /// 获取所有棋子
    pub fn all_pieces(&self) -> Vec<(Position, Piece)> {
        let mut result = Vec::new();
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let pos = Position::new_unchecked(x as u8, y as u8);
                if let Some(piece) = self.get(pos) {
                    result.push((pos, piece));
                }
            }
        }
        result
    }

    /// 检查两个将是否面对面（飞将）
    pub fn kings_facing(&self) -> bool {
        let red_king = self.find_king(Side::Red);
        let black_king = self.find_king(Side::Black);
        
        if let (Some(red_pos), Some(black_pos)) = (red_king, black_king) {
            // 必须在同一列
            if red_pos.x != black_pos.x {
                return false;
            }
            
            // 检查中间是否有棋子
            let (min_y, max_y) = if red_pos.y < black_pos.y {
                (red_pos.y, black_pos.y)
            } else {
                (black_pos.y, red_pos.y)
            };
            
            for y in (min_y + 1)..max_y {
                if self.get(Position::new_unchecked(red_pos.x, y)).is_some() {
                    return false;
                }
            }
            
            true
        } else {
            false
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::initial()
    }
}

/// 完整的棋盘状态（包含走子方、步数等）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardState {
    /// 棋盘
    pub board: Board,
    /// 当前走子方
    pub current_turn: Side,
    /// 无吃子步数（用于和棋判定）
    pub no_capture_count: u32,
    /// 完整回合数（一回合 = 红黑各走一步，黑方走完后 +1）
    pub round: u32,
    /// 位置历史（Zobrist hash，用于判断重复局面）
    /// TODO: 待实现 Zobrist hash 计算
    pub position_history: Vec<u64>,
}

impl BoardState {
    /// 创建初始状态
    pub fn initial() -> Self {
        Self {
            board: Board::initial(),
            current_turn: Side::Red,
            no_capture_count: 0,
            round: 1,
            position_history: Vec::new(),
        }
    }

    /// 从棋盘创建状态
    pub fn from_board(board: Board, current_turn: Side) -> Self {
        Self {
            board,
            current_turn,
            no_capture_count: 0,
            round: 1,
            position_history: Vec::new(),
        }
    }

    /// 切换走子方
    pub fn switch_turn(&mut self) {
        self.current_turn = self.current_turn.opponent();
        if self.current_turn == Side::Red {
            self.round += 1;
        }
    }
}

impl Default for BoardState {
    fn default() -> Self {
        Self::initial()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_board() {
        let board = Board::initial();
        
        // 检查红方帅
        let king = board.get(Position::new_unchecked(4, 0));
        assert_eq!(king, Some(Piece::new(PieceType::King, Side::Red)));
        
        // 检查黑方将
        let king = board.get(Position::new_unchecked(4, 9));
        assert_eq!(king, Some(Piece::new(PieceType::King, Side::Black)));
        
        // 检查红方炮
        let cannon = board.get(Position::new_unchecked(1, 2));
        assert_eq!(cannon, Some(Piece::new(PieceType::Cannon, Side::Red)));
        
        // 检查黑方卒
        let pawn = board.get(Position::new_unchecked(0, 6));
        assert_eq!(pawn, Some(Piece::new(PieceType::Pawn, Side::Black)));
    }

    #[test]
    fn test_move_piece() {
        let mut board = Board::initial();
        
        // 移动红方炮
        let from = Position::new_unchecked(1, 2);
        let to = Position::new_unchecked(1, 4);
        
        let captured = board.move_piece(from, to);
        assert!(captured.is_none());
        
        assert!(board.get(from).is_none());
        assert_eq!(
            board.get(to),
            Some(Piece::new(PieceType::Cannon, Side::Red))
        );
    }

    #[test]
    fn test_find_king() {
        let board = Board::initial();
        
        let red_king = board.find_king(Side::Red);
        assert_eq!(red_king, Some(Position::new_unchecked(4, 0)));
        
        let black_king = board.find_king(Side::Black);
        assert_eq!(black_king, Some(Position::new_unchecked(4, 9)));
    }

    #[test]
    fn test_kings_facing() {
        let mut board = Board::empty();
        
        // 放置两个将在同一列，中间没有棋子
        board.set(Position::new_unchecked(4, 0), Some(Piece::new(PieceType::King, Side::Red)));
        board.set(Position::new_unchecked(4, 9), Some(Piece::new(PieceType::King, Side::Black)));
        
        assert!(board.kings_facing());
        
        // 放一个棋子在中间
        board.set(Position::new_unchecked(4, 5), Some(Piece::new(PieceType::Pawn, Side::Red)));
        assert!(!board.kings_facing());
    }
}

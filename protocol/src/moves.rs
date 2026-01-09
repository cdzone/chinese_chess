//! 走法生成和验证

use serde::{Deserialize, Serialize};

use crate::board::{Board, BoardState};
use crate::piece::{Piece, PieceType, Position, Side};

/// 走法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Move {
    /// 起始位置
    pub from: Position,
    /// 目标位置
    pub to: Position,
    /// 被吃的棋子（如果有）
    pub captured: Option<Piece>,
}

impl Move {
    /// 创建新走法
    pub fn new(from: Position, to: Position) -> Self {
        Self {
            from,
            to,
            captured: None,
        }
    }

    /// 创建带吃子的走法
    pub fn with_capture(from: Position, to: Position, captured: Piece) -> Self {
        Self {
            from,
            to,
            captured: Some(captured),
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}

/// 走法生成器
pub struct MoveGenerator;

impl MoveGenerator {
    /// 生成指定阵营的所有伪合法走法（不考虑将军）
    pub fn generate_pseudo_legal(board: &Board, side: Side) -> Vec<Move> {
        let mut moves = Vec::with_capacity(64);

        for (pos, piece) in board.pieces(side) {
            Self::generate_piece_moves(board, pos, piece, &mut moves);
        }

        moves
    }

    /// 生成指定阵营的所有合法走法（过滤掉会导致被将军的走法）
    pub fn generate_legal(state: &BoardState) -> Vec<Move> {
        let pseudo_legal = Self::generate_pseudo_legal(&state.board, state.current_turn);
        
        pseudo_legal
            .into_iter()
            .filter(|mv| {
                // 模拟走法
                let mut test_board = state.board.clone();
                test_board.move_piece(mv.from, mv.to);
                
                // 检查是否被将军或飞将
                !Self::is_in_check(&test_board, state.current_turn)
                    && !test_board.kings_facing()
            })
            .collect()
    }

    /// 生成指定棋子的所有伪合法走法
    fn generate_piece_moves(board: &Board, pos: Position, piece: Piece, moves: &mut Vec<Move>) {
        match piece.piece_type {
            PieceType::King => Self::generate_king_moves(board, pos, piece.side, moves),
            PieceType::Advisor => Self::generate_advisor_moves(board, pos, piece.side, moves),
            PieceType::Bishop => Self::generate_bishop_moves(board, pos, piece.side, moves),
            PieceType::Knight => Self::generate_knight_moves(board, pos, piece.side, moves),
            PieceType::Rook => Self::generate_rook_moves(board, pos, piece.side, moves),
            PieceType::Cannon => Self::generate_cannon_moves(board, pos, piece.side, moves),
            PieceType::Pawn => Self::generate_pawn_moves(board, pos, piece.side, moves),
        }
    }

    /// 生成将/帅的走法
    fn generate_king_moves(board: &Board, pos: Position, side: Side, moves: &mut Vec<Move>) {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for (dx, dy) in directions {
            if let Some(to) = pos.offset(dx, dy) {
                // 必须在九宫格内
                if !to.is_in_palace(side) {
                    continue;
                }

                Self::try_add_move(board, pos, to, side, moves);
            }
        }
    }

    /// 生成士/仕的走法
    fn generate_advisor_moves(board: &Board, pos: Position, side: Side, moves: &mut Vec<Move>) {
        let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

        for (dx, dy) in directions {
            if let Some(to) = pos.offset(dx, dy) {
                // 必须在九宫格内
                if !to.is_in_palace(side) {
                    continue;
                }

                Self::try_add_move(board, pos, to, side, moves);
            }
        }
    }

    /// 生成象/相的走法
    fn generate_bishop_moves(board: &Board, pos: Position, side: Side, moves: &mut Vec<Move>) {
        let directions = [(2, 2), (2, -2), (-2, 2), (-2, -2)];
        let blocks = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

        for i in 0..4 {
            let (dx, dy) = directions[i];
            let (bx, by) = blocks[i];

            // 检查象眼是否被堵
            if let Some(block_pos) = pos.offset(bx, by) {
                if board.get(block_pos).is_some() {
                    continue;
                }
            } else {
                continue;
            }

            if let Some(to) = pos.offset(dx, dy) {
                // 不能过河
                let can_move = match side {
                    Side::Red => to.is_red_side(),
                    Side::Black => to.is_black_side(),
                };

                if can_move {
                    Self::try_add_move(board, pos, to, side, moves);
                }
            }
        }
    }

    /// 生成马/傌的走法
    fn generate_knight_moves(board: &Board, pos: Position, side: Side, moves: &mut Vec<Move>) {
        // 马的8个方向和对应的蹩马腿位置
        let knight_moves = [
            ((1, 2), (0, 1)),
            ((2, 1), (1, 0)),
            ((2, -1), (1, 0)),
            ((1, -2), (0, -1)),
            ((-1, -2), (0, -1)),
            ((-2, -1), (-1, 0)),
            ((-2, 1), (-1, 0)),
            ((-1, 2), (0, 1)),
        ];

        for ((dx, dy), (bx, by)) in knight_moves {
            // 检查马腿是否被堵
            if let Some(block_pos) = pos.offset(bx, by) {
                if board.get(block_pos).is_some() {
                    continue;
                }
            } else {
                continue;
            }

            if let Some(to) = pos.offset(dx, dy) {
                Self::try_add_move(board, pos, to, side, moves);
            }
        }
    }

    /// 生成车/俥的走法
    fn generate_rook_moves(board: &Board, pos: Position, side: Side, moves: &mut Vec<Move>) {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for (dx, dy) in directions {
            let mut current = pos;
            while let Some(to) = current.offset(dx, dy) {
                if let Some(target) = board.get(to) {
                    // 遇到棋子
                    if target.side != side {
                        // 可以吃
                        moves.push(Move::with_capture(pos, to, target));
                    }
                    break;
                } else {
                    // 空位，可以走
                    moves.push(Move::new(pos, to));
                }
                current = to;
            }
        }
    }

    /// 生成炮/砲的走法
    fn generate_cannon_moves(board: &Board, pos: Position, side: Side, moves: &mut Vec<Move>) {
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)];

        for (dx, dy) in directions {
            let mut current = pos;
            let mut jumped = false;

            while let Some(to) = current.offset(dx, dy) {
                if let Some(target) = board.get(to) {
                    if jumped {
                        // 已经跳过一个棋子，可以吃
                        if target.side != side {
                            moves.push(Move::with_capture(pos, to, target));
                        }
                        break;
                    } else {
                        // 第一个棋子，作为炮架
                        jumped = true;
                    }
                } else if !jumped {
                    // 还没跳过棋子，可以走到空位
                    moves.push(Move::new(pos, to));
                }
                current = to;
            }
        }
    }

    /// 生成兵/卒的走法
    fn generate_pawn_moves(board: &Board, pos: Position, side: Side, moves: &mut Vec<Move>) {
        let forward = match side {
            Side::Red => 1i8,
            Side::Black => -1i8,
        };

        // 前进
        if let Some(to) = pos.offset(0, forward) {
            Self::try_add_move(board, pos, to, side, moves);
        }

        // 过河后可以左右移动
        let crossed_river = match side {
            Side::Red => pos.y >= 5,
            Side::Black => pos.y <= 4,
        };

        if crossed_river {
            for dx in [-1i8, 1i8] {
                if let Some(to) = pos.offset(dx, 0) {
                    Self::try_add_move(board, pos, to, side, moves);
                }
            }
        }
    }

    /// 尝试添加走法（检查目标位置是否可以移动）
    fn try_add_move(board: &Board, from: Position, to: Position, side: Side, moves: &mut Vec<Move>) {
        if let Some(target) = board.get(to) {
            // 目标位置有棋子
            if target.side != side {
                // 可以吃
                moves.push(Move::with_capture(from, to, target));
            }
        } else {
            // 空位
            moves.push(Move::new(from, to));
        }
    }

    /// 检查指定阵营是否被将军
    pub fn is_in_check(board: &Board, side: Side) -> bool {
        let king_pos = match board.find_king(side) {
            Some(pos) => pos,
            None => return false, // 没有将，视为不被将军
        };

        // 检查对方所有棋子是否能攻击到将
        let opponent = side.opponent();
        for (pos, piece) in board.pieces(opponent) {
            if Self::can_attack(board, pos, piece, king_pos) {
                return true;
            }
        }

        false
    }

    /// 检查棋子是否能攻击到目标位置
    fn can_attack(board: &Board, from: Position, piece: Piece, target: Position) -> bool {
        match piece.piece_type {
            PieceType::King => {
                // 将不能攻击（飞将另外处理）
                false
            }
            PieceType::Advisor => {
                let dx = (target.x as i8 - from.x as i8).abs();
                let dy = (target.y as i8 - from.y as i8).abs();
                dx == 1 && dy == 1 && target.is_in_palace(piece.side)
            }
            PieceType::Bishop => {
                let dx = target.x as i8 - from.x as i8;
                let dy = target.y as i8 - from.y as i8;
                if dx.abs() != 2 || dy.abs() != 2 {
                    return false;
                }
                // 检查象眼
                let block_pos = Position::new_unchecked(
                    (from.x as i8 + dx / 2) as u8,
                    (from.y as i8 + dy / 2) as u8,
                );
                board.get(block_pos).is_none()
            }
            PieceType::Knight => {
                let dx = target.x as i8 - from.x as i8;
                let dy = target.y as i8 - from.y as i8;
                let is_knight_move = (dx.abs() == 1 && dy.abs() == 2)
                    || (dx.abs() == 2 && dy.abs() == 1);
                if !is_knight_move {
                    return false;
                }
                // 检查马腿
                let (bx, by) = if dx.abs() == 2 {
                    (dx.signum(), 0)
                } else {
                    (0, dy.signum())
                };
                let block_pos = Position::new_unchecked(
                    (from.x as i8 + bx) as u8,
                    (from.y as i8 + by) as u8,
                );
                board.get(block_pos).is_none()
            }
            PieceType::Rook => {
                Self::can_rook_attack(board, from, target)
            }
            PieceType::Cannon => {
                Self::can_cannon_attack(board, from, target)
            }
            PieceType::Pawn => {
                let dx = target.x as i8 - from.x as i8;
                let dy = target.y as i8 - from.y as i8;
                let forward = match piece.side {
                    Side::Red => 1,
                    Side::Black => -1,
                };
                let crossed = match piece.side {
                    Side::Red => from.y >= 5,
                    Side::Black => from.y <= 4,
                };
                
                if dx == 0 && dy == forward {
                    true
                } else if crossed && dy == 0 && dx.abs() == 1 {
                    true
                } else {
                    false
                }
            }
        }
    }

    /// 检查车是否能攻击目标
    fn can_rook_attack(board: &Board, from: Position, target: Position) -> bool {
        if from.x != target.x && from.y != target.y {
            return false;
        }

        let (dx, dy) = if from.x == target.x {
            (0, if target.y > from.y { 1 } else { -1 })
        } else {
            (if target.x > from.x { 1 } else { -1 }, 0)
        };

        let mut current = from;
        while let Some(next) = current.offset(dx, dy) {
            if next == target {
                return true;
            }
            if board.get(next).is_some() {
                return false;
            }
            current = next;
        }
        false
    }

    /// 检查炮是否能攻击目标
    fn can_cannon_attack(board: &Board, from: Position, target: Position) -> bool {
        if from.x != target.x && from.y != target.y {
            return false;
        }

        let (dx, dy) = if from.x == target.x {
            (0, if target.y > from.y { 1 } else { -1 })
        } else {
            (if target.x > from.x { 1 } else { -1 }, 0)
        };

        let mut current = from;
        let mut jumped = false;

        while let Some(next) = current.offset(dx, dy) {
            if next == target {
                return jumped; // 炮必须跳过一个棋子才能吃
            }
            if board.get(next).is_some() {
                if jumped {
                    return false; // 已经跳过一个，又遇到一个
                }
                jumped = true;
            }
            current = next;
        }
        false
    }

    /// 检查是否被将死
    pub fn is_checkmate(state: &BoardState) -> bool {
        // 如果没有被将军，不是将死
        if !Self::is_in_check(&state.board, state.current_turn) {
            return false;
        }

        // 如果有合法走法，不是将死
        Self::generate_legal(state).is_empty()
    }

    /// 检查是否是和棋（无子可动但未被将军）
    pub fn is_stalemate(state: &BoardState) -> bool {
        if Self::is_in_check(&state.board, state.current_turn) {
            return false;
        }
        Self::generate_legal(state).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::Fen;

    #[test]
    fn test_initial_moves() {
        let state = BoardState::initial();
        let moves = MoveGenerator::generate_legal(&state);
        
        // 初始局面红方应该有一些合法走法
        assert!(!moves.is_empty());
        
        // 检查是否包含炮二平五
        let cannon_move = moves.iter().find(|m| {
            m.from == Position::new_unchecked(7, 2)
                && m.to == Position::new_unchecked(4, 2)
        });
        assert!(cannon_move.is_some());
    }

    #[test]
    fn test_king_moves() {
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(4, 1),
            Some(Piece::new(PieceType::King, Side::Red)),
        );

        let mut moves = Vec::new();
        MoveGenerator::generate_king_moves(&board, Position::new_unchecked(4, 1), Side::Red, &mut moves);

        // 帅在九宫格中间，应该有4个方向
        assert_eq!(moves.len(), 4);
    }

    #[test]
    fn test_knight_moves() {
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(4, 4),
            Some(Piece::new(PieceType::Knight, Side::Red)),
        );

        let mut moves = Vec::new();
        MoveGenerator::generate_knight_moves(&board, Position::new_unchecked(4, 4), Side::Red, &mut moves);

        // 马在中间位置应该有8个方向
        assert_eq!(moves.len(), 8);
    }

    #[test]
    fn test_knight_blocked() {
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(4, 4),
            Some(Piece::new(PieceType::Knight, Side::Red)),
        );
        // 堵住一个马腿
        board.set(
            Position::new_unchecked(4, 5),
            Some(Piece::new(PieceType::Pawn, Side::Red)),
        );

        let mut moves = Vec::new();
        MoveGenerator::generate_knight_moves(&board, Position::new_unchecked(4, 4), Side::Red, &mut moves);

        // 应该少2个走法
        assert_eq!(moves.len(), 6);
    }

    #[test]
    fn test_check_detection() {
        // 创建一个红方被将军的局面
        let fen = "4k4/9/9/9/9/9/9/9/4r4/4K4 r 0 1";
        let state = Fen::parse(fen).unwrap();

        assert!(MoveGenerator::is_in_check(&state.board, Side::Red));
        assert!(!MoveGenerator::is_in_check(&state.board, Side::Black));
    }

    #[test]
    fn test_checkmate() {
        // 一个简单的将死局面：红方帅被黑方车将死
        let fen = "3k5/9/9/9/9/9/9/9/3rr4/3K5 r 0 1";
        let state = Fen::parse(fen).unwrap();

        assert!(MoveGenerator::is_checkmate(&state));
    }

    #[test]
    fn test_cannon_attack() {
        let mut board = Board::empty();
        // 炮在 (0, 0)
        board.set(
            Position::new_unchecked(0, 0),
            Some(Piece::new(PieceType::Cannon, Side::Red)),
        );
        // 炮架在 (0, 3)
        board.set(
            Position::new_unchecked(0, 3),
            Some(Piece::new(PieceType::Pawn, Side::Red)),
        );
        // 目标在 (0, 5)
        board.set(
            Position::new_unchecked(0, 5),
            Some(Piece::new(PieceType::King, Side::Black)),
        );

        assert!(MoveGenerator::can_cannon_attack(
            &board,
            Position::new_unchecked(0, 0),
            Position::new_unchecked(0, 5)
        ));

        // 没有炮架不能攻击
        assert!(!MoveGenerator::can_cannon_attack(
            &board,
            Position::new_unchecked(0, 0),
            Position::new_unchecked(0, 3)
        ));
    }
}

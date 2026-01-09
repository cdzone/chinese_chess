//! 棋局评估函数

use protocol::{Board, Piece, PieceType, Position, Side};

/// 评估器
pub struct Evaluator;

/// 棋子位置分值表（红方视角，黑方需要镜像）
/// 索引为 y * 9 + x
mod position_tables {
    /// 兵的位置分值
    pub const PAWN: [i32; 90] = [
        0,  0,  0,  0,  0,  0,  0,  0,  0,
        0,  0,  0,  0,  0,  0,  0,  0,  0,
        0,  0,  0,  0,  0,  0,  0,  0,  0,
        0,  0,  0,  0,  0,  0,  0,  0,  0,
        2,  6,  8, 12, 14, 12,  8,  6,  2,  // 过河后价值提升
       10, 20, 30, 40, 50, 40, 30, 20, 10,
       20, 40, 60, 80, 90, 80, 60, 40, 20,
       30, 60, 90,110,120,110, 90, 60, 30,
       40, 80,100,120,130,120,100, 80, 40,
        0,  0,  0,  0,  0,  0,  0,  0,  0,  // 不可能到达
    ];

    /// 马的位置分值
    pub const KNIGHT: [i32; 90] = [
        0, 10, 20, 30, 30, 30, 20, 10,  0,
       10, 30, 40, 50, 50, 50, 40, 30, 10,
       20, 40, 60, 70, 70, 70, 60, 40, 20,
       30, 50, 70, 80, 80, 80, 70, 50, 30,
       40, 60, 80, 90, 90, 90, 80, 60, 40,
       40, 60, 80, 90, 90, 90, 80, 60, 40,
       30, 50, 70, 80, 80, 80, 70, 50, 30,
       20, 40, 60, 70, 70, 70, 60, 40, 20,
       10, 30, 40, 50, 50, 50, 40, 30, 10,
        0, 10, 20, 30, 30, 30, 20, 10,  0,
    ];

    /// 炮的位置分值
    pub const CANNON: [i32; 90] = [
       10, 10, 10, 20, 30, 20, 10, 10, 10,
       10, 20, 30, 40, 50, 40, 30, 20, 10,
       10, 20, 30, 40, 50, 40, 30, 20, 10,
       10, 30, 40, 50, 60, 50, 40, 30, 10,
       10, 40, 50, 60, 70, 60, 50, 40, 10,
       10, 40, 50, 60, 70, 60, 50, 40, 10,
       10, 30, 40, 50, 60, 50, 40, 30, 10,
       10, 20, 30, 40, 50, 40, 30, 20, 10,
       10, 20, 30, 40, 50, 40, 30, 20, 10,
       10, 10, 10, 20, 30, 20, 10, 10, 10,
    ];

    /// 车的位置分值
    pub const ROOK: [i32; 90] = [
       10, 20, 20, 40, 50, 40, 20, 20, 10,
       20, 40, 50, 60, 70, 60, 50, 40, 20,
       20, 40, 50, 60, 70, 60, 50, 40, 20,
       30, 50, 60, 70, 80, 70, 60, 50, 30,
       40, 60, 70, 80, 90, 80, 70, 60, 40,
       40, 60, 70, 80, 90, 80, 70, 60, 40,
       30, 50, 60, 70, 80, 70, 60, 50, 30,
       20, 40, 50, 60, 70, 60, 50, 40, 20,
       20, 40, 50, 60, 70, 60, 50, 40, 20,
       10, 20, 20, 40, 50, 40, 20, 20, 10,
    ];
}

impl Evaluator {
    /// 评估棋局（红方视角，正值对红方有利）
    pub fn evaluate(board: &Board) -> i32 {
        let mut score = 0;

        for (pos, piece) in board.all_pieces() {
            let piece_score = Self::evaluate_piece(pos, piece);
            match piece.side {
                Side::Red => score += piece_score,
                Side::Black => score -= piece_score,
            }
        }

        score
    }

    /// 评估单个棋子的价值（包括位置分）
    fn evaluate_piece(pos: Position, piece: Piece) -> i32 {
        let base_value = piece.piece_type.value();
        let position_bonus = Self::position_bonus(pos, piece);
        base_value + position_bonus
    }

    /// 获取位置加成分
    fn position_bonus(pos: Position, piece: Piece) -> i32 {
        let index = match piece.side {
            Side::Red => pos.y as usize * 9 + pos.x as usize,
            // 黑方需要镜像（y 坐标翻转）
            Side::Black => (9 - pos.y as usize) * 9 + pos.x as usize,
        };

        match piece.piece_type {
            PieceType::Pawn => position_tables::PAWN[index],
            PieceType::Knight => position_tables::KNIGHT[index],
            PieceType::Cannon => position_tables::CANNON[index],
            PieceType::Rook => position_tables::ROOK[index],
            // 其他棋子暂时不加位置分
            _ => 0,
        }
    }

    /// 快速评估（仅计算子力差）
    pub fn evaluate_material(board: &Board) -> i32 {
        let mut score = 0;
        for (_, piece) in board.all_pieces() {
            let value = piece.piece_type.value();
            match piece.side {
                Side::Red => score += value,
                Side::Black => score -= value,
            }
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_evaluation() {
        let board = Board::initial();
        let score = Evaluator::evaluate(&board);
        // 初始局面应该是平衡的
        assert!(score.abs() < 100, "Initial position should be balanced, got {}", score);
    }

    #[test]
    fn test_material_evaluation() {
        let board = Board::initial();
        let score = Evaluator::evaluate_material(&board);
        // 初始局面子力应该相等
        assert_eq!(score, 0);
    }
}

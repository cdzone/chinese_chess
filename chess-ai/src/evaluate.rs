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
    use protocol::Fen;

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

    #[test]
    fn test_piece_values() {
        // 验证棋子基础分值
        assert_eq!(PieceType::King.value(), 10000);
        assert_eq!(PieceType::Rook.value(), 900);
        assert_eq!(PieceType::Knight.value(), 400);
        assert_eq!(PieceType::Cannon.value(), 450);
        assert_eq!(PieceType::Bishop.value(), 200);
        assert_eq!(PieceType::Advisor.value(), 200);
        assert_eq!(PieceType::Pawn.value(), 100);
    }

    #[test]
    fn test_material_advantage() {
        // 红方多一个车
        let fen = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABN1 w 0 1";
        let state = Fen::parse(fen).unwrap();
        let score = Evaluator::evaluate_material(&state.board);
        
        // 红方少一个车，分数应该为负
        assert!(score < 0, "红方少车应该分数为负: {}", score);
        assert!(score.abs() >= 900, "车的价值应该接近 1000: {}", score);
    }

    #[test]
    fn test_position_bonus_pawn() {
        // 过河兵比未过河兵价值高
        let fen = "4k4/9/9/9/4P4/9/9/9/9/4K4 w 0 1";
        let state = Fen::parse(fen).unwrap();
        let score1 = Evaluator::evaluate(&state.board);

        // 未过河兵
        let fen2 = "4k4/9/9/9/9/9/9/4P4/9/4K4 w 0 1";
        let state2 = Fen::parse(fen2).unwrap();
        let score2 = Evaluator::evaluate(&state2.board);

        // 过河兵应该分数更高
        assert!(score1 > score2, "过河兵应该比未过河兵价值高: {} vs {}", score1, score2);
    }

    #[test]
    fn test_position_bonus_knight() {
        // 中心位置的马比边角的马价值高
        let fen = "4k4/9/9/9/4N4/9/9/9/9/4K4 w 0 1";
        let state = Fen::parse(fen).unwrap();
        let score1 = Evaluator::evaluate(&state.board);

        // 边角的马
        let fen2 = "4k4/9/9/9/9/9/9/9/9/N3K4 w 0 1";
        let state2 = Fen::parse(fen2).unwrap();
        let score2 = Evaluator::evaluate(&state2.board);

        // 中心马应该分数更高
        assert!(score1 > score2, "中心马应该比边角马价值高: {} vs {}", score1, score2);
    }

    #[test]
    fn test_symmetry() {
        // 对称局面应该评估为 0
        let board = Board::initial();
        let score = Evaluator::evaluate_material(&board);
        assert_eq!(score, 0, "对称局面应该为 0");
    }

    #[test]
    fn test_black_mirror() {
        // 黑方棋子的位置分应该正确镜像
        // 红方兵在 (4, 5) 和 黑方卒在 (4, 4) 应该有相同的位置加成
        let fen = "4k4/9/9/9/4p4/4P4/9/9/9/4K4 w 0 1";
        let state = Fen::parse(fen).unwrap();
        let score = Evaluator::evaluate(&state.board);
        
        // 两个兵/卒在对称位置，分数应该接近 0
        println!("对称兵/卒局面分数: {}", score);
        assert!(score.abs() < 50, "对称兵/卒应该接近平衡: {}", score);
    }

    #[test]
    fn test_evaluate_endgame() {
        // 残局评估
        let fen = "4k4/9/9/9/9/9/9/9/4R4/4K4 w 0 1";
        let state = Fen::parse(fen).unwrap();
        let score = Evaluator::evaluate(&state.board);
        
        // 红方多一个车，应该大幅领先
        assert!(score > 500, "红方多车应该大幅领先: {}", score);
    }

    #[test]
    fn test_cannon_position() {
        // 炮在中心位置比边缘价值高
        let fen = "4k4/9/9/9/4C4/9/9/9/9/4K4 w 0 1";
        let state = Fen::parse(fen).unwrap();
        let score1 = Evaluator::evaluate(&state.board);

        let fen2 = "4k4/9/9/9/9/9/9/9/C8/4K4 w 0 1";
        let state2 = Fen::parse(fen2).unwrap();
        let score2 = Evaluator::evaluate(&state2.board);

        assert!(score1 > score2, "中心炮应该比边缘炮价值高: {} vs {}", score1, score2);
    }

    #[test]
    fn test_rook_position() {
        // 车在中心位置比边缘价值高
        let fen = "4k4/9/9/9/4R4/9/9/9/9/4K4 w 0 1";
        let state = Fen::parse(fen).unwrap();
        let score1 = Evaluator::evaluate(&state.board);

        let fen2 = "4k4/9/9/9/9/9/9/9/R8/4K4 w 0 1";
        let state2 = Fen::parse(fen2).unwrap();
        let score2 = Evaluator::evaluate(&state2.board);

        assert!(score1 > score2, "中心车应该比边缘车价值高: {} vs {}", score1, score2);
    }
}

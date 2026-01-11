//! 中文纵线表示法
//!
//! 红方：从右往左数，使用中文数字（一二三四五六七八九）
//! 黑方：从左往右数，使用阿拉伯数字（1-9）
//!
//! 格式：<棋子><起始列><动作><目标>
//! - 动作：进（向前）、退（向后）、平（横走）
//! - 目标：平移时为目标列，进退时为步数

use crate::board::Board;
use crate::moves::Move;
use crate::piece::{Position, Side};

/// 中文数字
const CHINESE_NUMBERS: [char; 9] = ['一', '二', '三', '四', '五', '六', '七', '八', '九'];

/// 纵线表示法
pub struct Notation;

impl Notation {
    /// 将走法转换为中文纵线表示法
    pub fn to_chinese(board: &Board, mv: &Move) -> Option<String> {
        let piece = board.get(mv.from)?;
        let side = piece.side;

        // 获取棋子名称
        let piece_char = piece.display_char();

        // 计算起始列（红方从右往左，黑方从左往右）
        let from_col = Self::column_notation(mv.from.x, side);

        // 判断动作类型
        let (action, target) = Self::action_and_target(mv, side);

        Some(format!("{}{}{}{}", piece_char, from_col, action, target))
    }

    /// 获取列的表示
    fn column_notation(x: u8, side: Side) -> char {
        match side {
            Side::Red => {
                // 红方从右往左：x=8 是一，x=0 是九
                CHINESE_NUMBERS[(8 - x) as usize]
            }
            Side::Black => {
                // 黑方从左往右：x=0 是 1，x=8 是 9
                char::from_digit((x + 1) as u32, 10).unwrap()
            }
        }
    }

    /// 获取动作和目标
    fn action_and_target(mv: &Move, side: Side) -> (char, char) {
        let dx = mv.to.x as i8 - mv.from.x as i8;
        let dy = mv.to.y as i8 - mv.from.y as i8;

        // 红方：y 增加是进，y 减少是退
        // 黑方：y 减少是进，y 增加是退
        let forward = match side {
            Side::Red => dy > 0,
            Side::Black => dy < 0,
        };

        if dy == 0 {
            // 平移
            let target_col = Self::column_notation(mv.to.x, side);
            ('平', target_col)
        } else if dx == 0 {
            // 直线进退
            let steps = dy.unsigned_abs();
            let target = match side {
                Side::Red => CHINESE_NUMBERS[(steps - 1) as usize],
                Side::Black => char::from_digit(steps as u32, 10).unwrap(),
            };
            let action = if forward { '進' } else { '退' };
            (action, target)
        } else {
            // 斜线移动（马、象、士）
            let target_col = Self::column_notation(mv.to.x, side);
            let action = if forward { '進' } else { '退' };
            (action, target_col)
        }
    }

    /// 处理同列多子的情况（前/后/中）
    /// 返回完整的棋谱记录
    pub fn to_chinese_with_disambiguation(board: &Board, mv: &Move) -> Option<String> {
        let piece = board.get(mv.from)?;
        let side = piece.side;

        // 查找同列同类型的棋子
        let same_column_pieces: Vec<Position> = (0..10)
            .filter_map(|y| {
                let pos = Position::new_unchecked(mv.from.x, y);
                board.get(pos).and_then(|p| {
                    if p.piece_type == piece.piece_type && p.side == side {
                        Some(pos)
                    } else {
                        None
                    }
                })
            })
            .collect();

        if same_column_pieces.len() <= 1 {
            // 没有同列同类型棋子，使用普通表示法
            return Self::to_chinese(board, mv);
        }

        // 需要消歧义
        let piece_char = piece.display_char();
        let (action, target) = Self::action_and_target(mv, side);

        // 按 y 坐标排序（红方从下到上，黑方从上到下）
        let mut sorted = same_column_pieces.clone();
        match side {
            Side::Red => sorted.sort_by_key(|p| p.y),
            Side::Black => sorted.sort_by_key(|p| std::cmp::Reverse(p.y)),
        }

        let position_idx = sorted.iter().position(|&p| p == mv.from)?;
        let position_char = if same_column_pieces.len() == 2 {
            if position_idx == 0 { '後' } else { '前' }
        } else {
            match position_idx {
                0 => '後',
                i if i == sorted.len() - 1 => '前',
                _ => '中',
            }
        };

        Some(format!("{}{}{}{}", position_char, piece_char, action, target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::piece::{Piece, PieceType};

    #[test]
    fn test_cannon_notation() {
        let board = Board::initial();

        // 炮二平五（红方右边的炮平到中间）
        let mv = Move::new(
            Position::new_unchecked(7, 2),
            Position::new_unchecked(4, 2),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "炮二平五");
    }

    #[test]
    fn test_pawn_notation() {
        let board = Board::initial();

        // 兵七进一（红方第三列的兵前进一步）
        let mv = Move::new(
            Position::new_unchecked(2, 3),
            Position::new_unchecked(2, 4),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "兵七進一");
    }

    #[test]
    fn test_knight_notation() {
        let board = Board::initial();

        // 马二进三（红方右边的马）
        let mv = Move::new(
            Position::new_unchecked(7, 0),
            Position::new_unchecked(6, 2),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "傌二進三");
    }

    #[test]
    fn test_black_notation() {
        let board = Board::initial();

        // 馬8進7（黑方左边的马）
        let mv = Move::new(
            Position::new_unchecked(1, 9),
            Position::new_unchecked(2, 7),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "馬2進3");
    }

    #[test]
    fn test_column_notation() {
        // 红方：x=8 是一，x=0 是九
        assert_eq!(Notation::column_notation(8, Side::Red), '一');
        assert_eq!(Notation::column_notation(4, Side::Red), '五');
        assert_eq!(Notation::column_notation(0, Side::Red), '九');

        // 黑方：x=0 是 1，x=8 是 9
        assert_eq!(Notation::column_notation(0, Side::Black), '1');
        assert_eq!(Notation::column_notation(4, Side::Black), '5');
        assert_eq!(Notation::column_notation(8, Side::Black), '9');
    }

    #[test]
    fn test_rook_retreat() {
        // 车退走法
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(0, 5),
            Some(Piece::new(PieceType::Rook, Side::Red)),
        );

        // 车九退三
        let mv = Move::new(
            Position::new_unchecked(0, 5),
            Position::new_unchecked(0, 2),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "俥九退三");
    }

    #[test]
    fn test_advisor_diagonal() {
        // 士斜走
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(4, 1),
            Some(Piece::new(PieceType::Advisor, Side::Red)),
        );

        // 仕五进四
        let mv = Move::new(
            Position::new_unchecked(4, 1),
            Position::new_unchecked(3, 2),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "仕五進六");
    }

    #[test]
    fn test_bishop_diagonal() {
        // 象斜走
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(2, 0),
            Some(Piece::new(PieceType::Bishop, Side::Red)),
        );

        // 相七进五
        let mv = Move::new(
            Position::new_unchecked(2, 0),
            Position::new_unchecked(4, 2),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "相七進五");
    }

    #[test]
    fn test_disambiguation_two_pieces() {
        // 同列两个兵
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(4, 5),
            Some(Piece::new(PieceType::Pawn, Side::Red)),
        );
        board.set(
            Position::new_unchecked(4, 6),
            Some(Piece::new(PieceType::Pawn, Side::Red)),
        );

        // 前兵进一
        let mv = Move::new(
            Position::new_unchecked(4, 6),
            Position::new_unchecked(4, 7),
        );
        let notation = Notation::to_chinese_with_disambiguation(&board, &mv).unwrap();
        assert_eq!(notation, "前兵進一");

        // 后兵进一
        let mv2 = Move::new(
            Position::new_unchecked(4, 5),
            Position::new_unchecked(4, 6),
        );
        let notation2 = Notation::to_chinese_with_disambiguation(&board, &mv2).unwrap();
        assert_eq!(notation2, "後兵進一");
    }

    #[test]
    fn test_disambiguation_three_pieces() {
        // 同列三个兵
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(4, 5),
            Some(Piece::new(PieceType::Pawn, Side::Red)),
        );
        board.set(
            Position::new_unchecked(4, 6),
            Some(Piece::new(PieceType::Pawn, Side::Red)),
        );
        board.set(
            Position::new_unchecked(4, 7),
            Some(Piece::new(PieceType::Pawn, Side::Red)),
        );

        // 中兵进一
        let mv = Move::new(
            Position::new_unchecked(4, 6),
            Position::new_unchecked(4, 7),
        );
        let notation = Notation::to_chinese_with_disambiguation(&board, &mv).unwrap();
        assert_eq!(notation, "中兵進一");
    }

    #[test]
    fn test_black_rook_advance() {
        // 黑车进
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(0, 9),
            Some(Piece::new(PieceType::Rook, Side::Black)),
        );

        // 車1進5
        let mv = Move::new(
            Position::new_unchecked(0, 9),
            Position::new_unchecked(0, 4),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "車1進5");
    }

    #[test]
    fn test_black_cannon_horizontal() {
        // 黑炮平移
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(1, 7),
            Some(Piece::new(PieceType::Cannon, Side::Black)),
        );

        // 砲2平5
        let mv = Move::new(
            Position::new_unchecked(1, 7),
            Position::new_unchecked(4, 7),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "砲2平5");
    }

    #[test]
    fn test_no_disambiguation_needed() {
        // 只有一个棋子，不需要消歧义
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(4, 5),
            Some(Piece::new(PieceType::Pawn, Side::Red)),
        );

        let mv = Move::new(
            Position::new_unchecked(4, 5),
            Position::new_unchecked(4, 6),
        );
        let notation = Notation::to_chinese_with_disambiguation(&board, &mv).unwrap();
        // 应该返回普通表示法
        assert_eq!(notation, "兵五進一");
    }

    #[test]
    fn test_black_knight_retreat() {
        // 黑马退
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(2, 7),
            Some(Piece::new(PieceType::Knight, Side::Black)),
        );

        // 馬3退2
        let mv = Move::new(
            Position::new_unchecked(2, 7),
            Position::new_unchecked(1, 9),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "馬3退2");
    }

    #[test]
    fn test_pawn_horizontal_after_river() {
        // 过河兵横走
        let mut board = Board::empty();
        board.set(
            Position::new_unchecked(4, 6),
            Some(Piece::new(PieceType::Pawn, Side::Red)),
        );

        // 兵五平六
        let mv = Move::new(
            Position::new_unchecked(4, 6),
            Position::new_unchecked(3, 6),
        );
        let notation = Notation::to_chinese(&board, &mv).unwrap();
        assert_eq!(notation, "兵五平六");
    }
}

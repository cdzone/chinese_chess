//! 棋子定义

use serde::{Deserialize, Serialize};

use crate::constants::{BOARD_HEIGHT, BOARD_WIDTH};

/// 棋子类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PieceType {
    /// 将/帅
    King,
    /// 士/仕
    Advisor,
    /// 象/相
    Bishop,
    /// 马/傌
    Knight,
    /// 车/俥
    Rook,
    /// 炮/砲
    Cannon,
    /// 兵/卒
    Pawn,
}

impl PieceType {
    /// 获取棋子的基础分值（用于 AI 评估）
    pub fn value(&self) -> i32 {
        match self {
            PieceType::King => 10000,
            PieceType::Rook => 900,
            PieceType::Cannon => 450,
            PieceType::Knight => 400,
            PieceType::Bishop => 200,
            PieceType::Advisor => 200,
            PieceType::Pawn => 100,
        }
    }

    /// 获取 FEN 字符（红方大写，黑方小写）
    pub fn to_fen_char(&self, side: Side) -> char {
        let c = match self {
            PieceType::King => 'k',
            PieceType::Advisor => 'a',
            PieceType::Bishop => 'b',
            PieceType::Knight => 'n',
            PieceType::Rook => 'r',
            PieceType::Cannon => 'c',
            PieceType::Pawn => 'p',
        };
        match side {
            Side::Red => c.to_ascii_uppercase(),
            Side::Black => c,
        }
    }

    /// 从 FEN 字符解析
    pub fn from_fen_char(c: char) -> Option<(PieceType, Side)> {
        let side = if c.is_ascii_uppercase() {
            Side::Red
        } else {
            Side::Black
        };
        let piece_type = match c.to_ascii_lowercase() {
            'k' => PieceType::King,
            'a' => PieceType::Advisor,
            'b' => PieceType::Bishop,
            'n' => PieceType::Knight,
            'r' => PieceType::Rook,
            'c' => PieceType::Cannon,
            'p' => PieceType::Pawn,
            _ => return None,
        };
        Some((piece_type, side))
    }
}

/// 阵营
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    /// 红方（先手，在下方）
    Red,
    /// 黑方（后手，在上方）
    Black,
}

impl Side {
    /// 获取对方阵营
    pub fn opponent(&self) -> Side {
        match self {
            Side::Red => Side::Black,
            Side::Black => Side::Red,
        }
    }

    /// 获取 FEN 字符
    pub fn to_fen_char(&self) -> char {
        match self {
            Side::Red => 'r',
            Side::Black => 'b',
        }
    }

    /// 从 FEN 字符解析
    pub fn from_fen_char(c: char) -> Option<Side> {
        match c {
            'r' | 'R' => Some(Side::Red),
            'b' | 'B' => Some(Side::Black),
            _ => None,
        }
    }
}

/// 棋子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Piece {
    pub piece_type: PieceType,
    pub side: Side,
}

impl Piece {
    /// 创建新棋子
    pub fn new(piece_type: PieceType, side: Side) -> Self {
        Self { piece_type, side }
    }

    /// 获取棋子显示的汉字
    pub fn display_char(&self) -> char {
        match (self.piece_type, self.side) {
            (PieceType::King, Side::Red) => '帥',
            (PieceType::King, Side::Black) => '將',
            (PieceType::Advisor, Side::Red) => '仕',
            (PieceType::Advisor, Side::Black) => '士',
            (PieceType::Bishop, Side::Red) => '相',
            (PieceType::Bishop, Side::Black) => '象',
            (PieceType::Knight, Side::Red) => '傌',
            (PieceType::Knight, Side::Black) => '馬',
            (PieceType::Rook, Side::Red) => '俥',
            (PieceType::Rook, Side::Black) => '車',
            (PieceType::Cannon, Side::Red) => '炮',
            (PieceType::Cannon, Side::Black) => '砲',
            (PieceType::Pawn, Side::Red) => '兵',
            (PieceType::Pawn, Side::Black) => '卒',
        }
    }

    /// 获取 FEN 字符
    pub fn to_fen_char(&self) -> char {
        self.piece_type.to_fen_char(self.side)
    }

    /// 从 FEN 字符解析
    pub fn from_fen_char(c: char) -> Option<Piece> {
        PieceType::from_fen_char(c).map(|(piece_type, side)| Piece { piece_type, side })
    }

    /// 获取棋子分值
    pub fn value(&self) -> i32 {
        self.piece_type.value()
    }
}

/// 棋盘位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// 列 (0-8)
    pub x: u8,
    /// 行 (0-9)
    pub y: u8,
}

impl Position {
    /// 创建新位置
    pub fn new(x: u8, y: u8) -> Option<Self> {
        if (x as usize) < BOARD_WIDTH && (y as usize) < BOARD_HEIGHT {
            Some(Self { x, y })
        } else {
            None
        }
    }

    /// 创建新位置（不检查边界，内部使用）
    pub const fn new_unchecked(x: u8, y: u8) -> Self {
        Self { x, y }
    }

    /// 检查位置是否在棋盘内
    pub fn is_valid(&self) -> bool {
        (self.x as usize) < BOARD_WIDTH && (self.y as usize) < BOARD_HEIGHT
    }

    /// 检查位置是否在红方区域（y: 0-4）
    pub fn is_red_side(&self) -> bool {
        self.y < 5
    }

    /// 检查位置是否在黑方区域（y: 5-9）
    pub fn is_black_side(&self) -> bool {
        self.y >= 5
    }

    /// 检查位置是否在九宫格内
    pub fn is_in_palace(&self, side: Side) -> bool {
        let in_x = (3..=5).contains(&self.x);
        let in_y = match side {
            Side::Red => (0..=2).contains(&self.y),
            Side::Black => (7..=9).contains(&self.y),
        };
        in_x && in_y
    }

    /// 获取偏移后的位置
    pub fn offset(&self, dx: i8, dy: i8) -> Option<Position> {
        let new_x = self.x as i8 + dx;
        let new_y = self.y as i8 + dy;
        if new_x >= 0 && (new_x as usize) < BOARD_WIDTH && new_y >= 0 && (new_y as usize) < BOARD_HEIGHT {
            Some(Position {
                x: new_x as u8,
                y: new_y as u8,
            })
        } else {
            None
        }
    }

    /// 转换为数组索引
    pub fn to_index(&self) -> usize {
        self.y as usize * 9 + self.x as usize
    }

    /// 从数组索引转换
    pub fn from_index(index: usize) -> Option<Self> {
        if index < BOARD_WIDTH * BOARD_HEIGHT {
            Some(Position {
                x: (index % BOARD_WIDTH) as u8,
                y: (index / BOARD_WIDTH) as u8,
            })
        } else {
            None
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_display_char() {
        let red_king = Piece::new(PieceType::King, Side::Red);
        assert_eq!(red_king.display_char(), '帥');

        let black_king = Piece::new(PieceType::King, Side::Black);
        assert_eq!(black_king.display_char(), '將');

        let red_pawn = Piece::new(PieceType::Pawn, Side::Red);
        assert_eq!(red_pawn.display_char(), '兵');

        let black_pawn = Piece::new(PieceType::Pawn, Side::Black);
        assert_eq!(black_pawn.display_char(), '卒');
    }

    #[test]
    fn test_piece_fen_char() {
        let red_king = Piece::new(PieceType::King, Side::Red);
        assert_eq!(red_king.to_fen_char(), 'K');

        let black_king = Piece::new(PieceType::King, Side::Black);
        assert_eq!(black_king.to_fen_char(), 'k');

        assert_eq!(
            Piece::from_fen_char('R'),
            Some(Piece::new(PieceType::Rook, Side::Red))
        );
        assert_eq!(
            Piece::from_fen_char('n'),
            Some(Piece::new(PieceType::Knight, Side::Black))
        );
    }

    #[test]
    fn test_position_valid() {
        assert!(Position::new(0, 0).is_some());
        assert!(Position::new(8, 9).is_some());
        assert!(Position::new(9, 0).is_none());
        assert!(Position::new(0, 10).is_none());
    }

    #[test]
    fn test_position_palace() {
        // 红方九宫格
        assert!(Position::new_unchecked(4, 0).is_in_palace(Side::Red));
        assert!(Position::new_unchecked(4, 2).is_in_palace(Side::Red));
        assert!(!Position::new_unchecked(4, 3).is_in_palace(Side::Red));

        // 黑方九宫格
        assert!(Position::new_unchecked(4, 9).is_in_palace(Side::Black));
        assert!(Position::new_unchecked(4, 7).is_in_palace(Side::Black));
        assert!(!Position::new_unchecked(4, 6).is_in_palace(Side::Black));
    }

    #[test]
    fn test_side_opponent() {
        assert_eq!(Side::Red.opponent(), Side::Black);
        assert_eq!(Side::Black.opponent(), Side::Red);
    }
}

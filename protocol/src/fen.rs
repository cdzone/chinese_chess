//! FEN 格式解析和生成
//!
//! 中国象棋 FEN 格式：
//! `<棋盘> <走子方> <无吃子步数> <回合数>`
//!
//! 示例：
//! `rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR r 0 1`

use crate::board::{Board, BoardState};
use crate::error::ChessError;
use crate::piece::{Piece, Position, Side};

/// 初始局面 FEN
pub const INITIAL_FEN: &str = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR r 0 1";

/// FEN 格式处理
pub struct Fen;

impl Fen {
    /// 解析 FEN 字符串为棋盘状态
    pub fn parse(fen: &str) -> Result<BoardState, ChessError> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ChessError::InvalidFen {
                reason: "Empty FEN string".to_string(),
            });
        }

        // 解析棋盘
        let board = Self::parse_board(parts[0])?;

        // 解析走子方（默认红方）
        let current_turn = if parts.len() > 1 {
            Side::from_fen_char(parts[1].chars().next().unwrap_or('r'))
                .unwrap_or(Side::Red)
        } else {
            Side::Red
        };

        // 解析无吃子步数（默认 0）
        let no_capture_count = if parts.len() > 2 {
            parts[2].parse().unwrap_or(0)
        } else {
            0
        };

        // 解析回合数（默认 1）
        let round = if parts.len() > 3 {
            parts[3].parse().unwrap_or(1)
        } else {
            1
        };

        Ok(BoardState {
            board,
            current_turn,
            no_capture_count,
            round,
            position_history: Vec::new(),
        })
    }

    /// 解析棋盘部分
    fn parse_board(board_str: &str) -> Result<Board, ChessError> {
        let mut board = Board::empty();
        let rows: Vec<&str> = board_str.split('/').collect();

        if rows.len() != 10 {
            return Err(ChessError::InvalidFen {
                reason: format!("Expected 10 rows, got {}", rows.len()),
            });
        }

        // FEN 从上到下是 y=9 到 y=0
        for (row_idx, row) in rows.iter().enumerate() {
            let y = 9 - row_idx as u8;
            let mut x = 0u8;

            for c in row.chars() {
                if x >= 9 {
                    return Err(ChessError::InvalidFen {
                        reason: format!("Row {} has too many columns", row_idx),
                    });
                }

                if c.is_ascii_digit() {
                    // 空格数量
                    let empty_count = c.to_digit(10).unwrap() as u8;
                    x += empty_count;
                } else if let Some(piece) = Piece::from_fen_char(c) {
                    board.set(Position::new_unchecked(x, y), Some(piece));
                    x += 1;
                } else {
                    return Err(ChessError::InvalidFen {
                        reason: format!("Invalid piece character: {}", c),
                    });
                }
            }

            if x != 9 {
                return Err(ChessError::InvalidFen {
                    reason: format!("Row {} has {} columns, expected 9", row_idx, x),
                });
            }
        }

        Ok(board)
    }

    /// 将棋盘状态转换为 FEN 字符串
    pub fn to_string(state: &BoardState) -> String {
        let board_str = Self::board_to_string(&state.board);
        format!(
            "{} {} {} {}",
            board_str,
            state.current_turn.to_fen_char(),
            state.no_capture_count,
            state.round
        )
    }

    /// 将棋盘转换为 FEN 棋盘部分
    pub fn board_to_string(board: &Board) -> String {
        let mut rows = Vec::with_capacity(10);

        // 从 y=9 到 y=0
        for y in (0..10).rev() {
            let mut row = String::new();
            let mut empty_count = 0;

            for x in 0..9 {
                if let Some(piece) = board.get(Position::new_unchecked(x, y)) {
                    if empty_count > 0 {
                        row.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    row.push(piece.to_fen_char());
                } else {
                    empty_count += 1;
                }
            }

            if empty_count > 0 {
                row.push_str(&empty_count.to_string());
            }

            rows.push(row);
        }

        rows.join("/")
    }

    /// 解析初始局面
    pub fn initial() -> BoardState {
        Self::parse(INITIAL_FEN).expect("Initial FEN should be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::PieceType;

    #[test]
    fn test_parse_initial_fen() {
        let state = Fen::parse(INITIAL_FEN).unwrap();

        // 检查走子方
        assert_eq!(state.current_turn, Side::Red);

        // 检查红方帅
        let king = state.board.get(Position::new_unchecked(4, 0));
        assert_eq!(king, Some(Piece::new(PieceType::King, Side::Red)));

        // 检查黑方将
        let king = state.board.get(Position::new_unchecked(4, 9));
        assert_eq!(king, Some(Piece::new(PieceType::King, Side::Black)));

        // 检查红方炮
        let cannon = state.board.get(Position::new_unchecked(1, 2));
        assert_eq!(cannon, Some(Piece::new(PieceType::Cannon, Side::Red)));

        // 检查黑方炮
        let cannon = state.board.get(Position::new_unchecked(1, 7));
        assert_eq!(cannon, Some(Piece::new(PieceType::Cannon, Side::Black)));
    }

    #[test]
    fn test_fen_roundtrip() {
        let state = Fen::initial();
        let fen = Fen::to_string(&state);
        let state2 = Fen::parse(&fen).unwrap();

        assert_eq!(state.board, state2.board);
        assert_eq!(state.current_turn, state2.current_turn);
        assert_eq!(state.no_capture_count, state2.no_capture_count);
        assert_eq!(state.round, state2.round);
    }

    #[test]
    fn test_parse_custom_fen() {
        // 测试一个自定义局面
        let fen = "4k4/9/9/9/9/9/9/9/9/4K4 b 10 5";
        let state = Fen::parse(fen).unwrap();

        assert_eq!(state.current_turn, Side::Black);
        assert_eq!(state.no_capture_count, 10);
        assert_eq!(state.round, 5);

        // 只有两个将
        let red_king = state.board.find_king(Side::Red);
        let black_king = state.board.find_king(Side::Black);
        assert_eq!(red_king, Some(Position::new_unchecked(4, 0)));
        assert_eq!(black_king, Some(Position::new_unchecked(4, 9)));
    }

    #[test]
    fn test_invalid_fen() {
        // 行数不对
        assert!(Fen::parse("4k4/9/9").is_err());

        // 列数不对
        assert!(Fen::parse("4k44/9/9/9/9/9/9/9/9/4K4 r").is_err());

        // 无效字符
        assert!(Fen::parse("4x4/9/9/9/9/9/9/9/9/4K4 r").is_err());
    }
}

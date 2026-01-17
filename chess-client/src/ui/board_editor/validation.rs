//! 棋局验证

use protocol::{Board, PieceType, Position, Side, MoveGenerator, BoardState};

/// 棋局验证结果
#[derive(Debug, Clone, Default)]
pub struct BoardValidation {
    /// 阻塞错误（必须修复才能开始）
    pub errors: Vec<String>,
    /// 非阻塞警告（可选择忽略）
    pub warnings: Vec<String>,
}

impl BoardValidation {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// 验证棋局
pub fn validate_board(board: &Board, first_turn: Side) -> BoardValidation {
    let mut result = BoardValidation::default();

    // 收集所有棋子
    let all_pieces = board.all_pieces();

    // 统计各方棋子数量
    let mut red_king = 0;
    let mut black_king = 0;
    let mut red_rook = 0;
    let mut black_rook = 0;
    let mut red_knight = 0;
    let mut black_knight = 0;
    let mut red_cannon = 0;
    let mut black_cannon = 0;
    let mut red_advisor = 0;
    let mut black_advisor = 0;
    let mut red_bishop = 0;
    let mut black_bishop = 0;
    let mut red_pawn = 0;
    let mut black_pawn = 0;

    for (pos, piece) in &all_pieces {
        match (piece.piece_type, piece.side) {
            (PieceType::King, Side::Red) => {
                red_king += 1;
                // 检查九宫位置
                if !is_in_red_palace(*pos) {
                    result.errors.push("帥必须在九宫内".to_string());
                }
            }
            (PieceType::King, Side::Black) => {
                black_king += 1;
                // 检查九宫位置
                if !is_in_black_palace(*pos) {
                    result.errors.push("將必须在九宫内".to_string());
                }
            }
            (PieceType::Advisor, Side::Red) => {
                red_advisor += 1;
                if !is_in_red_palace(*pos) {
                    result.errors.push("仕必须在九宫内".to_string());
                }
            }
            (PieceType::Advisor, Side::Black) => {
                black_advisor += 1;
                if !is_in_black_palace(*pos) {
                    result.errors.push("士必须在九宫内".to_string());
                }
            }
            (PieceType::Bishop, Side::Red) => {
                red_bishop += 1;
                if !is_in_red_half(*pos) {
                    result.errors.push("相必须在红方半场".to_string());
                } else if !is_valid_bishop_position(*pos, Side::Red) {
                    result.warnings.push("相在非标准位置".to_string());
                }
            }
            (PieceType::Bishop, Side::Black) => {
                black_bishop += 1;
                if !is_in_black_half(*pos) {
                    result.errors.push("象必须在黑方半场".to_string());
                } else if !is_valid_bishop_position(*pos, Side::Black) {
                    result.warnings.push("象在非标准位置".to_string());
                }
            }
            (PieceType::Rook, Side::Red) => red_rook += 1,
            (PieceType::Rook, Side::Black) => black_rook += 1,
            (PieceType::Knight, Side::Red) => red_knight += 1,
            (PieceType::Knight, Side::Black) => black_knight += 1,
            (PieceType::Cannon, Side::Red) => red_cannon += 1,
            (PieceType::Cannon, Side::Black) => black_cannon += 1,
            (PieceType::Pawn, Side::Red) => {
                red_pawn += 1;
                // 检查底线警告
                if pos.y == 0 {
                    result.warnings.push("兵的位置可能不合理（在底线）".to_string());
                }
            }
            (PieceType::Pawn, Side::Black) => {
                black_pawn += 1;
                // 检查底线警告
                if pos.y == 9 {
                    result.warnings.push("卒的位置可能不合理（在底线）".to_string());
                }
            }
        }
    }

    // 检查将帅存在
    if red_king == 0 {
        result.errors.push("红方必须有帥".to_string());
    }
    if black_king == 0 {
        result.errors.push("黑方必须有將".to_string());
    }

    // 检查将帅面对面
    if board.kings_facing() {
        result.errors.push("将帅不能面对面".to_string());
    }

    // 检查数量限制
    if red_rook > 2 {
        result.errors.push("红方車最多 2 个".to_string());
    }
    if black_rook > 2 {
        result.errors.push("黑方車最多 2 个".to_string());
    }
    if red_knight > 2 {
        result.errors.push("红方馬最多 2 个".to_string());
    }
    if black_knight > 2 {
        result.errors.push("黑方馬最多 2 个".to_string());
    }
    if red_cannon > 2 {
        result.errors.push("红方炮最多 2 个".to_string());
    }
    if black_cannon > 2 {
        result.errors.push("黑方砲最多 2 个".to_string());
    }
    if red_advisor > 2 {
        result.errors.push("红方仕最多 2 个".to_string());
    }
    if black_advisor > 2 {
        result.errors.push("黑方士最多 2 个".to_string());
    }
    if red_bishop > 2 {
        result.errors.push("红方相最多 2 个".to_string());
    }
    if black_bishop > 2 {
        result.errors.push("黑方象最多 2 个".to_string());
    }
    if red_pawn > 5 {
        result.errors.push("红方兵最多 5 个".to_string());
    }
    if black_pawn > 5 {
        result.errors.push("黑方卒最多 5 个".to_string());
    }

    // 检查先手方是否有合法走法
    if result.errors.is_empty() {
        let state = BoardState::from_board(board.clone(), first_turn);
        let legal_moves = MoveGenerator::generate_legal(&state);
        if legal_moves.is_empty() {
            let side_name = if first_turn == Side::Red { "红" } else { "黑" };
            result.errors.push(format!("当前局面：{}方已无合法走法", side_name));
        }

        // 检查是否被将军（警告）
        if is_in_check(board, first_turn) {
            let side_name = if first_turn == Side::Red { "红" } else { "黑" };
            result.warnings.push(format!("当前局面：{}方被将军", side_name));
        }
    }

    result
}

/// 检查是否在红方九宫内
fn is_in_red_palace(pos: Position) -> bool {
    pos.x >= 3 && pos.x <= 5 && pos.y <= 2
}

/// 检查是否在黑方九宫内
fn is_in_black_palace(pos: Position) -> bool {
    pos.x >= 3 && pos.x <= 5 && pos.y >= 7
}

/// 检查是否在红方半场
fn is_in_red_half(pos: Position) -> bool {
    pos.y <= 4
}

/// 检查是否在黑方半场
fn is_in_black_half(pos: Position) -> bool {
    pos.y >= 5
}

/// 检查相/象是否在标准位置（田字位）
fn is_valid_bishop_position(pos: Position, side: Side) -> bool {
    let valid_positions = if side == Side::Red {
        // 红相的 7 个合法位置
        [(2, 0), (6, 0), (0, 2), (4, 2), (8, 2), (2, 4), (6, 4)]
    } else {
        // 黑象的 7 个合法位置
        [(2, 9), (6, 9), (0, 7), (4, 7), (8, 7), (2, 5), (6, 5)]
    };

    valid_positions.iter().any(|&(x, y)| pos.x == x && pos.y == y)
}

/// 检查指定方是否被将军
fn is_in_check(board: &Board, side: Side) -> bool {
    let king_pos = match board.find_king(side) {
        Some(pos) => pos,
        None => return false,
    };

    // 检查对方所有棋子是否能攻击到将/帅
    let opponent = side.opponent();

    // 生成对方所有伪合法走法
    let moves = MoveGenerator::generate_pseudo_legal(board, opponent);

    // 检查是否有走法能吃到将/帅
    moves.iter().any(|mv| mv.to == king_pos)
}

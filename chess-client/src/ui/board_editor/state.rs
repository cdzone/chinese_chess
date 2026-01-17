//! 棋盘编辑器状态

use bevy::prelude::*;
use protocol::{Board, Piece, Side, Difficulty};

/// 对手类型
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum OpponentType {
    #[default]
    AI,
    LocalPvP,
}

/// 棋盘编辑器状态
#[derive(Resource)]
pub struct BoardEditorState {
    /// 当前棋盘布局
    pub board: Board,
    /// 当前选中的棋子（准备放置）
    pub selected_piece: Option<Piece>,
    /// 先手方
    pub first_turn: Side,
    /// 对手类型
    pub opponent_type: OpponentType,
    /// AI 难度（人机模式）
    pub ai_difficulty: Difficulty,
    /// 玩家执子方（人机模式）
    pub player_side: Side,
    /// 是否有未保存的修改
    pub is_modified: bool,
    /// 布局名称（用于保存）
    pub layout_name: String,
    /// 当前验证错误
    pub validation_errors: Vec<String>,
    /// 当前验证警告
    pub validation_warnings: Vec<String>,
    /// 是否显示警告确认对话框
    pub show_warning_dialog: bool,
}

impl Default for BoardEditorState {
    fn default() -> Self {
        Self {
            board: Board::empty(),
            selected_piece: None,
            first_turn: Side::Red,
            opponent_type: OpponentType::AI,
            ai_difficulty: Difficulty::Medium,
            player_side: Side::Red,
            is_modified: false,
            layout_name: String::new(),
            validation_errors: Vec::new(),
            validation_warnings: Vec::new(),
            show_warning_dialog: false,
        }
    }
}

impl BoardEditorState {
    /// 重置为空棋盘
    pub fn clear(&mut self) {
        self.board = Board::empty();
        self.selected_piece = None;
        self.is_modified = true;
        self.validation_errors.clear();
        self.validation_warnings.clear();
    }

    /// 重置为标准开局
    pub fn set_initial(&mut self) {
        self.board = Board::initial();
        self.selected_piece = None;
        self.is_modified = true;
        self.validation_errors.clear();
        self.validation_warnings.clear();
    }

    /// 仅放置将帅
    pub fn set_kings_only(&mut self) {
        self.board = Board::empty();
        // 红帅在 (4, 0)
        self.board.set(
            protocol::Position::new_unchecked(4, 0),
            Some(protocol::Piece::new(protocol::PieceType::King, Side::Red)),
        );
        // 黑将在 (4, 9)
        self.board.set(
            protocol::Position::new_unchecked(4, 9),
            Some(protocol::Piece::new(protocol::PieceType::King, Side::Black)),
        );
        self.selected_piece = None;
        self.is_modified = true;
        self.validation_errors.clear();
        self.validation_warnings.clear();
    }

    /// 放置棋子到指定位置
    pub fn place_piece(&mut self, x: u8, y: u8) {
        if let Some(piece) = self.selected_piece {
            if let Some(pos) = protocol::Position::new(x, y) {
                self.board.set(pos, Some(piece));
                self.is_modified = true;
            }
        }
    }

    /// 移除指定位置的棋子
    pub fn remove_piece(&mut self, x: u8, y: u8) {
        if let Some(pos) = protocol::Position::new(x, y) {
            if self.board.get(pos).is_some() {
                self.board.set(pos, None);
                self.is_modified = true;
            }
        }
    }

    /// 选择棋子
    pub fn select_piece(&mut self, piece: Piece) {
        self.selected_piece = Some(piece);
    }

    /// 取消选择
    pub fn deselect(&mut self) {
        self.selected_piece = None;
    }
}

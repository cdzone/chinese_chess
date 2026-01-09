//! 客户端游戏状态

use bevy::prelude::*;
use protocol::{BoardState, MoveGenerator, Position, RoomType, Side};

/// 客户端游戏状态
#[derive(Resource, Default)]
pub struct ClientGame {
    /// 当前游戏状态
    pub game_state: Option<BoardState>,
    /// 玩家所属阵营
    pub player_side: Option<Side>,
    /// 房间类型
    pub room_type: Option<RoomType>,
    /// 选中的棋子位置
    pub selected_piece: Option<Position>,
    /// 合法走法目标位置
    pub valid_moves: Vec<Position>,
    /// 最后一步走法 (from, to)
    pub last_move: Option<(Position, Position)>,
    /// 红方剩余时间 (毫秒)
    pub red_time_ms: u64,
    /// 黑方剩余时间 (毫秒)
    pub black_time_ms: u64,
    /// 棋谱记录
    pub move_history: Vec<MoveRecord>,
    /// 游戏是否暂停
    pub is_paused: bool,
    /// 是否在等待对方响应悔棋
    pub waiting_undo_response: bool,
}

/// 走法记录
#[derive(Clone, Debug)]
pub struct MoveRecord {
    pub notation: String,
    pub from: Position,
    pub to: Position,
}

impl ClientGame {
    /// 重置游戏状态
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// 初始化新游戏
    pub fn start_game(&mut self, state: BoardState, side: Side, room_type: RoomType) {
        self.game_state = Some(state);
        self.player_side = Some(side);
        self.room_type = Some(room_type);
        self.selected_piece = None;
        self.valid_moves.clear();
        self.last_move = None;
        self.move_history.clear();
        self.is_paused = false;
        self.waiting_undo_response = false;
    }

    /// 是否轮到玩家走棋
    pub fn is_my_turn(&self) -> bool {
        if let (Some(state), Some(side)) = (&self.game_state, self.player_side) {
            state.current_turn == side && !self.is_paused
        } else {
            false
        }
    }

    /// 选择棋子
    pub fn select_piece(&mut self, x: u8, y: u8) {
        let Some(pos) = Position::new(x, y) else {
            return;
        };

        // 检查是否是自己的棋子
        if let Some(state) = &self.game_state {
            if let Some(piece) = state.board.get(pos) {
                if Some(piece.side) == self.player_side && self.is_my_turn() {
                    self.selected_piece = Some(pos);
                    // 计算合法走法
                    self.valid_moves = self.calculate_valid_moves(pos);
                    return;
                }
            }
        }

        // 如果点击的是合法落点，执行走棋
        if self.valid_moves.contains(&pos) {
            // 这里不直接执行，而是通过事件系统
            return;
        }

        // 否则清除选择
        self.clear_selection();
    }

    /// 计算指定位置棋子的合法走法
    fn calculate_valid_moves(&self, from: Position) -> Vec<Position> {
        if let Some(state) = &self.game_state {
            let moves = MoveGenerator::generate_legal(state);
            moves
                .into_iter()
                .filter(|m| m.from == from)
                .map(|m| m.to)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 清除选择
    pub fn clear_selection(&mut self) {
        self.selected_piece = None;
        self.valid_moves.clear();
    }

    /// 更新游戏状态（收到服务器消息后）
    pub fn update_state(&mut self, new_state: BoardState, from: Position, to: Position, notation: String) {
        self.game_state = Some(new_state);
        self.last_move = Some((from, to));
        self.move_history.push(MoveRecord { notation, from, to });
        self.clear_selection();
    }

    /// 更新时间
    pub fn update_time(&mut self, red_time_ms: u64, black_time_ms: u64) {
        self.red_time_ms = red_time_ms;
        self.black_time_ms = black_time_ms;
    }

    /// 悔棋
    pub fn undo(&mut self, new_state: BoardState, steps: usize) {
        self.game_state = Some(new_state);
        // 移除历史记录
        for _ in 0..steps {
            self.move_history.pop();
        }
        // 更新最后走法
        self.last_move = self.move_history.last().map(|r| (r.from, r.to));
        self.clear_selection();
        self.waiting_undo_response = false;
    }

    /// 是否是 PvE 模式
    pub fn is_pve(&self) -> bool {
        matches!(self.room_type, Some(RoomType::PvE(_)))
    }

    /// 获取当前走子方
    pub fn current_turn(&self) -> Option<Side> {
        self.game_state.as_ref().map(|s| s.current_turn)
    }

    /// 获取玩家剩余时间
    pub fn my_time_ms(&self) -> u64 {
        match self.player_side {
            Some(Side::Red) => self.red_time_ms,
            Some(Side::Black) => self.black_time_ms,
            None => 0,
        }
    }

    /// 获取对手剩余时间
    pub fn opponent_time_ms(&self) -> u64 {
        match self.player_side {
            Some(Side::Red) => self.black_time_ms,
            Some(Side::Black) => self.red_time_ms,
            None => 0,
        }
    }
}

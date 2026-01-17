//! 客户端游戏状态

use bevy::prelude::*;
use protocol::{BoardState, Difficulty, GameResult, MoveGenerator, Position, RoomId, Side};

/// 游戏模式
///
/// 与服务端的 RoomType 分离，客户端使用 GameMode 区分本地和在线模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameMode {
    /// 本地单机 AI（完全离线，AI 在客户端运行）
    LocalPvE {
        difficulty: Difficulty,
    },
    /// 本地双人对战（同一设备）
    LocalPvP,
    /// 在线 AI 对战（AI 在服务端运行，保留作为备选）
    OnlinePvE {
        room_id: RoomId,
        difficulty: Difficulty,
    },
    /// 在线玩家对战
    OnlinePvP {
        room_id: RoomId,
    },
}

impl GameMode {
    /// 是否是本地模式（不需要网络）
    pub fn is_local(&self) -> bool {
        matches!(self, GameMode::LocalPvE { .. } | GameMode::LocalPvP)
    }

    /// 是否是 PvE 模式（包括本地和在线）
    pub fn is_pve(&self) -> bool {
        matches!(self, GameMode::LocalPvE { .. } | GameMode::OnlinePvE { .. })
    }

    /// 是否是在线模式
    pub fn is_online(&self) -> bool {
        !self.is_local()
    }

    /// 获取难度（仅 PvE 模式有效）
    pub fn difficulty(&self) -> Option<Difficulty> {
        match self {
            GameMode::LocalPvE { difficulty } => Some(*difficulty),
            GameMode::OnlinePvE { difficulty, .. } => Some(*difficulty),
            GameMode::OnlinePvP { .. } | GameMode::LocalPvP => None,
        }
    }

    /// 获取房间 ID（仅在线模式有效）
    pub fn room_id(&self) -> Option<RoomId> {
        match self {
            GameMode::LocalPvE { .. } | GameMode::LocalPvP => None,
            GameMode::OnlinePvE { room_id, .. } => Some(*room_id),
            GameMode::OnlinePvP { room_id } => Some(*room_id),
        }
    }
}

/// 客户端游戏状态
#[derive(Resource, Default)]
pub struct ClientGame {
    /// 当前游戏状态
    pub game_state: Option<BoardState>,
    /// 玩家所属阵营
    pub player_side: Option<Side>,
    /// 游戏模式
    pub game_mode: Option<GameMode>,
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
    /// 状态历史（用于本地悔棋）
    pub state_history: Vec<BoardState>,
    /// 游戏是否暂停
    pub is_paused: bool,
    /// 是否在等待对方响应悔棋
    pub waiting_undo_response: bool,
    /// 游戏结果
    pub game_result: Option<GameResult>,
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
    pub fn start_game(&mut self, state: BoardState, side: Side, mode: GameMode) {
        self.game_state = Some(state.clone());
        self.player_side = Some(side);
        self.game_mode = Some(mode);
        self.selected_piece = None;
        self.valid_moves.clear();
        self.last_move = None;
        self.move_history.clear();
        self.state_history.clear();
        self.state_history.push(state); // 保存初始状态
        self.is_paused = false;
        self.waiting_undo_response = false;
        self.game_result = None;
    }

    /// 初始化本地 PvE 游戏（无需网络）
    pub fn start_local_pve(&mut self, difficulty: Difficulty) {
        let state = BoardState::initial();
        self.start_game(state, Side::Red, GameMode::LocalPvE { difficulty });
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

    /// 更新游戏状态（收到服务器消息后或本地走棋后）
    pub fn update_state(&mut self, new_state: BoardState, from: Position, to: Position, notation: String) {
        // P1 修复：保存新状态到历史（不是旧状态）
        // state_history 记录的是每一步走完后的状态，用于悔棋恢复
        self.state_history.push(new_state.clone());
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

    /// 悔棋（在线模式，由服务器提供新状态）
    pub fn undo(&mut self, new_state: BoardState, steps: usize) {
        self.game_state = Some(new_state);
        // 移除历史记录
        for _ in 0..steps {
            self.move_history.pop();
            self.state_history.pop();
        }
        // 更新最后走法
        self.last_move = self.move_history.last().map(|r| (r.from, r.to));
        self.clear_selection();
        self.waiting_undo_response = false;
    }

    /// 本地悔棋（本地 PvE 模式，撤销 2 步：玩家 + AI）
    pub fn local_undo(&mut self) -> bool {
        // 本地 PvE 模式需要撤销 2 步（玩家走法 + AI 走法）
        // 但如果只走了 1 步（玩家刚走完，AI 还没走），只撤销 1 步
        let steps = if self.move_history.len() >= 2 {
            2
        } else if self.move_history.len() == 1 {
            1
        } else {
            return false; // 没有可撤销的走法
        };

        // P1 修复：移除走法记录和对应的状态历史
        for _ in 0..steps {
            self.move_history.pop();
            self.state_history.pop();
        }

        // 恢复到上一个状态
        // state_history[0] 是初始状态，之后每个元素是每步走完后的状态
        if let Some(prev_state) = self.state_history.last() {
            self.game_state = Some(prev_state.clone());
            self.last_move = self.move_history.last().map(|r| (r.from, r.to));
            self.clear_selection();
            self.game_result = None; // 清除游戏结果
            true
        } else {
            false
        }
    }

    /// 是否可以悔棋（游戏进行中）
    pub fn can_undo(&self) -> bool {
        !self.move_history.is_empty() && self.game_result.is_none()
    }

    /// 是否可以在终盘悔棋（游戏结束后）
    pub fn can_undo_at_game_over(&self) -> bool {
        !self.move_history.is_empty()
    }

    /// 是否是 PvE 模式（包括本地和在线）
    pub fn is_pve(&self) -> bool {
        self.game_mode.as_ref().map_or(false, |m| m.is_pve())
    }

    /// 是否是本地模式
    pub fn is_local(&self) -> bool {
        self.game_mode.as_ref().map_or(false, |m| m.is_local())
    }

    /// 是否需要 AI 走棋（本地 PvE 模式且轮到 AI）
    pub fn should_ai_move(&self) -> bool {
        match &self.game_mode {
            Some(GameMode::LocalPvE { .. }) => {
                if self.is_paused || self.game_result.is_some() {
                    return false;
                }
                // AI 方是玩家的对手
                let ai_side = self.player_side.map(|s| s.opponent());
                self.current_turn() == ai_side
            }
            _ => false,
        }
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

    /// 设置游戏结果
    pub fn set_result(&mut self, result: GameResult) {
        self.game_result = Some(result);
    }

    /// 判断玩家是否获胜
    pub fn is_player_win(&self) -> Option<bool> {
        match (&self.game_result, self.player_side) {
            (Some(GameResult::RedWin(_)), Some(Side::Red)) => Some(true),
            (Some(GameResult::BlackWin(_)), Some(Side::Black)) => Some(true),
            (Some(GameResult::RedWin(_)), Some(Side::Black)) => Some(false),
            (Some(GameResult::BlackWin(_)), Some(Side::Red)) => Some(false),
            (Some(GameResult::Draw(_)), _) => None, // 和棋返回 None
            _ => None,
        }
    }

    /// 获取总步数
    pub fn total_moves(&self) -> usize {
        self.move_history.len()
    }

    /// 获取回合数
    pub fn total_rounds(&self) -> usize {
        (self.move_history.len() + 1) / 2
    }
}

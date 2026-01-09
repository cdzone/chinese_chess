//! 房间系统

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use protocol::{
    BoardState, GameResult, Move, MoveGenerator, PlayerId, RoomId, RoomInfo,
    RoomState, RoomType, Side, WinReason,
};

use crate::game::GameTimer;

/// 房间
pub struct Room {
    pub id: RoomId,
    pub room_type: RoomType,
    pub state: RoomState,
    /// 红方玩家 ID
    pub red_player: Option<PlayerId>,
    /// 黑方玩家 ID
    pub black_player: Option<PlayerId>,
    /// 棋盘状态
    pub game_state: Option<BoardState>,
    /// 计时器
    pub timer: Option<GameTimer>,
    /// 走法历史
    pub move_history: Vec<Move>,
    /// 无吃子计数历史（用于悔棋恢复）
    pub no_capture_history: Vec<u32>,
    /// 创建时间
    pub created_at: Instant,
    /// 悔棋请求方（如果有）
    pub undo_requested_by: Option<Side>,
}

impl Room {
    /// 创建新房间
    pub fn new(id: RoomId, room_type: RoomType) -> Self {
        Self {
            id,
            room_type,
            state: RoomState::Waiting,
            red_player: None,
            black_player: None,
            game_state: None,
            timer: None,
            move_history: Vec::new(),
            no_capture_history: Vec::new(),
            created_at: Instant::now(),
            undo_requested_by: None,
        }
    }

    /// 获取房间信息（用于列表展示）
    pub fn info(&self, red_name: Option<String>, black_name: Option<String>) -> RoomInfo {
        RoomInfo {
            id: self.id,
            room_type: self.room_type,
            red_player: red_name,
            black_player: black_name,
            state: self.state,
        }
    }

    /// 检查房间是否已满
    pub fn is_full(&self) -> bool {
        self.red_player.is_some() && self.black_player.is_some()
    }

    /// 检查玩家是否在房间中
    pub fn has_player(&self, player_id: PlayerId) -> bool {
        self.red_player == Some(player_id) || self.black_player == Some(player_id)
    }

    /// 获取玩家的颜色
    pub fn get_player_side(&self, player_id: PlayerId) -> Option<Side> {
        if self.red_player == Some(player_id) {
            Some(Side::Red)
        } else if self.black_player == Some(player_id) {
            Some(Side::Black)
        } else {
            None
        }
    }

    /// 获取指定颜色的玩家 ID
    pub fn get_player_id(&self, side: Side) -> Option<PlayerId> {
        match side {
            Side::Red => self.red_player,
            Side::Black => self.black_player,
        }
    }

    /// 获取对手 ID
    pub fn get_opponent_id(&self, player_id: PlayerId) -> Option<PlayerId> {
        if self.red_player == Some(player_id) {
            self.black_player
        } else if self.black_player == Some(player_id) {
            self.red_player
        } else {
            None
        }
    }

    /// 添加玩家到房间
    pub fn add_player(&mut self, player_id: PlayerId, preferred_side: Option<Side>) -> Option<Side> {
        // 如果指定了颜色偏好
        if let Some(side) = preferred_side {
            match side {
                Side::Red if self.red_player.is_none() => {
                    self.red_player = Some(player_id);
                    return Some(Side::Red);
                }
                Side::Black if self.black_player.is_none() => {
                    self.black_player = Some(player_id);
                    return Some(Side::Black);
                }
                _ => {}
            }
        }

        // 否则分配空位
        if self.red_player.is_none() {
            self.red_player = Some(player_id);
            Some(Side::Red)
        } else if self.black_player.is_none() {
            self.black_player = Some(player_id);
            Some(Side::Black)
        } else {
            None // 房间已满
        }
    }

    /// 移除玩家
    pub fn remove_player(&mut self, player_id: PlayerId) -> Option<Side> {
        if self.red_player == Some(player_id) {
            self.red_player = None;
            Some(Side::Red)
        } else if self.black_player == Some(player_id) {
            self.black_player = None;
            Some(Side::Black)
        } else {
            None
        }
    }

    /// 开始游戏
    pub fn start_game(&mut self) {
        self.game_state = Some(BoardState::initial());
        self.timer = Some(GameTimer::new());
        self.state = RoomState::Playing;
        self.move_history.clear();
        self.no_capture_history.clear();
    }

    /// 暂停游戏（仅 PvE）
    pub fn pause(&mut self) -> bool {
        if matches!(self.room_type, RoomType::PvE(_)) && self.state == RoomState::Playing {
            if let Some(timer) = &mut self.timer {
                timer.pause();
            }
            self.state = RoomState::Paused;
            true
        } else {
            false
        }
    }

    /// 继续游戏（仅 PvE）
    pub fn resume(&mut self) -> bool {
        if matches!(self.room_type, RoomType::PvE(_)) && self.state == RoomState::Paused {
            if let Some(timer) = &mut self.timer {
                timer.resume();
            }
            self.state = RoomState::Playing;
            true
        } else {
            false
        }
    }

    /// 结束游戏
    pub fn finish(&mut self, _result: GameResult) {
        self.state = RoomState::Finished;
        if let Some(timer) = &mut self.timer {
            timer.stop();
        }
    }

    /// 执行走棋
    pub fn make_move(&mut self, mv: Move) -> Result<(), &'static str> {
        let game_state = self.game_state.as_mut().ok_or("游戏未开始")?;
        
        // 验证走法合法性
        let legal_moves = MoveGenerator::generate_legal(game_state);
        if !legal_moves.iter().any(|m| m.from == mv.from && m.to == mv.to) {
            return Err("无效走法");
        }

        // 记录当前无吃子计数（用于悔棋恢复）
        self.no_capture_history.push(game_state.no_capture_count);

        // 执行走法
        let captured = game_state.board.move_piece(mv.from, mv.to);
        
        // 更新无吃子计数
        if captured.is_some() {
            game_state.no_capture_count = 0;
        } else {
            game_state.no_capture_count += 1;
        }

        // 切换走子方
        game_state.switch_turn();

        // 更新计时器
        if let Some(timer) = &mut self.timer {
            timer.switch_turn();
        }

        // 记录走法
        let mut recorded_move = mv;
        recorded_move.captured = captured;
        self.move_history.push(recorded_move);

        // 清除悔棋请求
        self.undo_requested_by = None;

        Ok(())
    }

    /// 悔棋
    pub fn undo_move(&mut self) -> Result<Move, &'static str> {
        let last_move = self.move_history.pop().ok_or("没有可悔的棋")?;
        let game_state = self.game_state.as_mut().ok_or("游戏未开始")?;

        // 恢复棋子位置
        let piece = game_state.board.get(last_move.to);
        game_state.board.set(last_move.from, piece);
        
        // 恢复被吃的棋子
        if let Some(captured_piece) = last_move.captured {
            game_state.board.set(last_move.to, Some(captured_piece));
        } else {
            game_state.board.set(last_move.to, None);
        }

        // 恢复无吃子计数
        if let Some(prev_count) = self.no_capture_history.pop() {
            game_state.no_capture_count = prev_count;
        }

        // 切换回上一方
        game_state.switch_turn();

        // 同步更新计时器状态
        if let Some(timer) = &mut self.timer {
            timer.switch_turn();
        }

        // 清除悔棋请求
        self.undo_requested_by = None;

        Ok(last_move)
    }

    /// 检查游戏是否结束
    pub fn check_game_over(&self) -> Option<GameResult> {
        let game_state = self.game_state.as_ref()?;
        
        // 检查绝杀
        let legal_moves = MoveGenerator::generate_legal(game_state);
        if legal_moves.is_empty() {
            if MoveGenerator::is_in_check(&game_state.board, game_state.current_turn) {
                // 被将死
                return Some(match game_state.current_turn {
                    Side::Red => GameResult::BlackWin(WinReason::Checkmate),
                    Side::Black => GameResult::RedWin(WinReason::Checkmate),
                });
            } else {
                // 困毙（和棋）
                return Some(GameResult::Draw(protocol::DrawReason::Stalemate));
            }
        }

        // 检查超时
        if let Some(timer) = &self.timer {
            if timer.red_time_ms() == 0 {
                return Some(GameResult::BlackWin(WinReason::Timeout));
            }
            if timer.black_time_ms() == 0 {
                return Some(GameResult::RedWin(WinReason::Timeout));
            }
        }

        // 检查 60 回合无吃子
        if game_state.no_capture_count >= 120 {
            return Some(GameResult::Draw(protocol::DrawReason::FiftyMoves));
        }

        None
    }

    /// 生成棋谱记录
    pub fn generate_game_record(&self, red_name: &str, black_name: &str) -> Option<protocol::GameRecord> {
        use protocol::{Board, GameRecord, MoveRecord, Notation};
        
        let _game_state = self.game_state.as_ref()?;
        
        let mut record = GameRecord::new(red_name.to_string(), black_name.to_string());
        
        // 从初始棋盘开始重放，每步走棋前生成记谱
        let mut board = Board::initial();
        for mv in &self.move_history {
            // 在走棋前用当前棋盘状态生成记谱
            let notation = Notation::to_chinese(&board, mv).unwrap_or("未知".to_string());
            let move_record = MoveRecord::new(mv.from, mv.to, notation);
            record.add_move(move_record);
            
            // 执行走法更新棋盘
            board.move_piece(mv.from, mv.to);
        }
        
        // 如果游戏结束，设置结果
        if let Some(result) = self.check_game_over() {
            record.set_result(result);
        }
        
        Some(record)
    }

    /// 获取当前时间状态
    pub fn get_time_state(&self) -> (u64, u64) {
        if let Some(timer) = &self.timer {
            (timer.red_time_ms(), timer.black_time_ms())
        } else {
            (0, 0)
        }
    }
}

/// 房间管理器
pub struct RoomManager {
    rooms: HashMap<RoomId, Room>,
    next_id: AtomicU64,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            next_id: AtomicU64::new(1),
        }
    }

    /// 生成新的房间 ID
    fn generate_id(&self) -> RoomId {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// 创建房间
    pub fn create(&mut self, room_type: RoomType) -> RoomId {
        let id = self.generate_id();
        let room = Room::new(id, room_type);
        self.rooms.insert(id, room);
        id
    }

    /// 获取房间
    pub fn get(&self, room_id: RoomId) -> Option<&Room> {
        self.rooms.get(&room_id)
    }

    /// 获取房间（可变）
    pub fn get_mut(&mut self, room_id: RoomId) -> Option<&mut Room> {
        self.rooms.get_mut(&room_id)
    }

    /// 移除房间
    pub fn remove(&mut self, room_id: RoomId) -> Option<Room> {
        self.rooms.remove(&room_id)
    }

    /// 获取可加入的房间列表（Waiting 状态的 PvP 房间）
    pub fn list_joinable(&self) -> Vec<&Room> {
        self.rooms
            .values()
            .filter(|r| r.state == RoomState::Waiting && matches!(r.room_type, RoomType::PvP))
            .collect()
    }

    /// 查找玩家所在的房间
    pub fn find_player_room(&self, player_id: PlayerId) -> Option<RoomId> {
        self.rooms
            .values()
            .find(|r| r.has_player(player_id))
            .map(|r| r.id)
    }

    /// 获取房间数量
    pub fn count(&self) -> usize {
        self.rooms.len()
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::Difficulty;

    #[test]
    fn test_create_room() {
        let mut manager = RoomManager::new();
        
        let id1 = manager.create(RoomType::PvP);
        let id2 = manager.create(RoomType::PvE(Difficulty::Medium));
        
        assert_ne!(id1, id2);
        assert_eq!(manager.count(), 2);
    }

    #[test]
    fn test_add_player() {
        let mut room = Room::new(1, RoomType::PvP);
        
        // 第一个玩家加入
        let side1 = room.add_player(100, None);
        assert_eq!(side1, Some(Side::Red));
        
        // 第二个玩家加入
        let side2 = room.add_player(200, None);
        assert_eq!(side2, Some(Side::Black));
        
        // 第三个玩家无法加入
        let side3 = room.add_player(300, None);
        assert_eq!(side3, None);
        
        assert!(room.is_full());
    }

    #[test]
    fn test_preferred_side() {
        let mut room = Room::new(1, RoomType::PvP);
        
        // 第一个玩家选择黑方
        let side1 = room.add_player(100, Some(Side::Black));
        assert_eq!(side1, Some(Side::Black));
        
        // 第二个玩家只能是红方
        let side2 = room.add_player(200, None);
        assert_eq!(side2, Some(Side::Red));
    }

    #[test]
    fn test_start_game() {
        let mut room = Room::new(1, RoomType::PvP);
        room.add_player(100, None);
        room.add_player(200, None);
        
        room.start_game();
        
        assert_eq!(room.state, RoomState::Playing);
        assert!(room.game_state.is_some());
        assert!(room.timer.is_some());
    }

    #[test]
    fn test_list_joinable() {
        let mut manager = RoomManager::new();
        
        // 创建 3 个 PvP 房间
        let id1 = manager.create(RoomType::PvP);
        let id2 = manager.create(RoomType::PvP);
        let _id3 = manager.create(RoomType::PvE(Difficulty::Easy));
        
        // 让一个房间开始游戏
        {
            let room = manager.get_mut(id1).unwrap();
            room.add_player(100, None);
            room.add_player(200, None);
            room.start_game();
        }
        
        // 只有一个可加入的房间
        let joinable = manager.list_joinable();
        assert_eq!(joinable.len(), 1);
        assert_eq!(joinable[0].id, id2);
    }
}

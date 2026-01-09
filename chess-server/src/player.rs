//! 玩家管理

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use protocol::{PlayerId, RoomId};

/// 玩家状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStatus {
    /// 在线，在大厅
    Online,
    /// 在线，在房间中
    InRoom(RoomId),
    /// 断线中（保留房间）
    Disconnected(RoomId),
}

/// 玩家信息
#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub nickname: String,
    pub status: PlayerStatus,
}

impl Player {
    pub fn new(id: PlayerId, nickname: String) -> Self {
        Self {
            id,
            nickname,
            status: PlayerStatus::Online,
        }
    }
}

/// 玩家管理器
pub struct PlayerManager {
    /// 玩家 ID -> 玩家信息
    players: HashMap<PlayerId, Player>,
    /// 昵称 -> 玩家 ID（用于昵称唯一性检查）
    nickname_to_id: HashMap<String, PlayerId>,
    /// ID 生成器
    next_id: AtomicU64,
}

impl PlayerManager {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            nickname_to_id: HashMap::new(),
            next_id: AtomicU64::new(1),
        }
    }

    /// 生成新的玩家 ID
    fn generate_id(&self) -> PlayerId {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// 验证昵称
    pub fn validate_nickname(nickname: &str) -> Result<(), &'static str> {
        if nickname.is_empty() {
            return Err("昵称不能为空");
        }
        if nickname.chars().count() > 20 {
            return Err("昵称不能超过20个字符");
        }
        Ok(())
    }

    /// 登录玩家
    pub fn login(&mut self, nickname: String) -> Result<PlayerId, &'static str> {
        Self::validate_nickname(&nickname)?;

        // 检查昵称是否已被占用
        if self.nickname_to_id.contains_key(&nickname) {
            return Err("昵称已被占用");
        }

        let id = self.generate_id();
        let player = Player::new(id, nickname.clone());
        
        self.players.insert(id, player);
        self.nickname_to_id.insert(nickname, id);
        
        Ok(id)
    }

    /// 玩家断线
    pub fn disconnect(&mut self, player_id: PlayerId) -> Option<RoomId> {
        if let Some(player) = self.players.get_mut(&player_id) {
            match player.status {
                PlayerStatus::InRoom(room_id) => {
                    player.status = PlayerStatus::Disconnected(room_id);
                    Some(room_id)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// 玩家重连
    pub fn reconnect(&mut self, player_id: PlayerId) -> Option<RoomId> {
        if let Some(player) = self.players.get_mut(&player_id) {
            match player.status {
                PlayerStatus::Disconnected(room_id) => {
                    player.status = PlayerStatus::InRoom(room_id);
                    Some(room_id)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    /// 移除玩家（彻底离线）
    pub fn remove(&mut self, player_id: PlayerId) -> Option<Player> {
        if let Some(player) = self.players.remove(&player_id) {
            self.nickname_to_id.remove(&player.nickname);
            Some(player)
        } else {
            None
        }
    }

    /// 获取玩家
    pub fn get(&self, player_id: PlayerId) -> Option<&Player> {
        self.players.get(&player_id)
    }

    /// 获取玩家（可变）
    pub fn get_mut(&mut self, player_id: PlayerId) -> Option<&mut Player> {
        self.players.get_mut(&player_id)
    }

    /// 设置玩家状态
    pub fn set_status(&mut self, player_id: PlayerId, status: PlayerStatus) {
        if let Some(player) = self.players.get_mut(&player_id) {
            player.status = status;
        }
    }

    /// 获取玩家昵称
    pub fn get_nickname(&self, player_id: PlayerId) -> Option<&str> {
        self.players.get(&player_id).map(|p| p.nickname.as_str())
    }

    /// 检查玩家是否存在
    pub fn exists(&self, player_id: PlayerId) -> bool {
        self.players.contains_key(&player_id)
    }

    /// 获取在线玩家数量
    pub fn online_count(&self) -> usize {
        self.players.len()
    }
}

impl Default for PlayerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login() {
        let mut manager = PlayerManager::new();
        
        let id1 = manager.login("玩家1".to_string()).unwrap();
        assert!(id1 > 0);
        
        let id2 = manager.login("玩家2".to_string()).unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_duplicate_nickname() {
        let mut manager = PlayerManager::new();
        
        manager.login("玩家1".to_string()).unwrap();
        let result = manager.login("玩家1".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_nickname() {
        let mut manager = PlayerManager::new();
        
        // 空昵称
        assert!(manager.login("".to_string()).is_err());
        
        // 超长昵称
        let long_name = "a".repeat(21);
        assert!(manager.login(long_name).is_err());
    }

    #[test]
    fn test_disconnect_reconnect() {
        let mut manager = PlayerManager::new();
        
        let id = manager.login("玩家1".to_string()).unwrap();
        manager.set_status(id, PlayerStatus::InRoom(100));
        
        // 断线
        let room_id = manager.disconnect(id);
        assert_eq!(room_id, Some(100));
        
        let player = manager.get(id).unwrap();
        assert!(matches!(player.status, PlayerStatus::Disconnected(100)));
        
        // 重连
        let room_id = manager.reconnect(id);
        assert_eq!(room_id, Some(100));
        
        let player = manager.get(id).unwrap();
        assert!(matches!(player.status, PlayerStatus::InRoom(100)));
    }
}

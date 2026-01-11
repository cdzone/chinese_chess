//! 棋局存储系统
//!
//! 提供跨平台的棋局保存和加载功能

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use protocol::{GameRecord, BoardState};

/// 存储管理器
pub struct StorageManager {
    saves_dir: PathBuf,
}

impl StorageManager {
    /// 创建存储管理器
    pub fn new() -> Result<Self> {
        let saves_dir = get_saves_directory()?;
        
        // 确保目录存在
        if !saves_dir.exists() {
            fs::create_dir_all(&saves_dir)
                .with_context(|| format!("无法创建存储目录: {:?}", saves_dir))?;
        }

        Ok(Self { saves_dir })
    }

    /// 保存棋局
    pub fn save_game(
        &self,
        red_player: &str,
        black_player: &str,
        game_record: &mut GameRecord,
        game_state: &BoardState,
        red_time_ms: u64,
        black_time_ms: u64,
    ) -> Result<String> {
        // 生成文件名
        let timestamp = Utc::now();
        let filename = generate_filename(&timestamp, red_player, black_player);
        let filepath = self.saves_dir.join(&filename);

        // 添加保存信息
        game_record.save_info = Some(protocol::SaveInfo {
            saved_at: timestamp,
            game_state: match game_state.current_turn {
                protocol::Side::Red => "red_turn".to_string(),
                protocol::Side::Black => "black_turn".to_string(),
            },
            red_time_remaining_ms: red_time_ms,
            black_time_remaining_ms: black_time_ms,
        });

        // 序列化并保存
        let json_content = game_record.to_json()
            .context("序列化棋谱失败")?;

        fs::write(&filepath, json_content)
            .with_context(|| format!("写入文件失败: {:?}", filepath))?;

        // 返回文件名（不含路径）
        Ok(filename)
    }

    /// 加载棋局
    pub fn load_game(&self, game_id: &str) -> Result<GameRecord> {
        let filepath = self.saves_dir.join(game_id);
        
        if !filepath.exists() {
            anyhow::bail!("棋局文件不存在: {}", game_id);
        }

        let content = fs::read_to_string(&filepath)
            .with_context(|| format!("读取文件失败: {:?}", filepath))?;

        GameRecord::from_json(&content)
            .context("解析棋谱文件失败")
    }

    /// 列出所有保存的棋局
    pub fn list_saved_games(&self) -> Result<Vec<SavedGameInfo>> {
        let mut games = Vec::new();

        if !self.saves_dir.exists() {
            return Ok(games);
        }

        let entries = fs::read_dir(&self.saves_dir)
            .with_context(|| format!("读取存储目录失败: {:?}", self.saves_dir))?;

        for entry in entries {
            let entry = entry.context("读取目录项失败")?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    // 尝试解析文件获取基本信息
                    match self.load_game(filename) {
                        Ok(record) => {
                            games.push(SavedGameInfo {
                                game_id: filename.to_string(),
                                red_player: record.metadata.red_player,
                                black_player: record.metadata.black_player,
                                saved_at: record.save_info
                                    .map(|s| s.saved_at)
                                    .unwrap_or_else(|| {
                                        // 使用文件修改时间作为后备
                                        entry.metadata()
                                            .and_then(|m| m.modified())
                                            .map(DateTime::from)
                                            .unwrap_or_else(|_| Utc::now())
                                    }),
                                move_count: record.moves.len(),
                            });
                        }
                        Err(_) => {
                            // 跳过损坏的文件
                            continue;
                        }
                    }
                }
            }
        }

        // 按保存时间倒序排列
        games.sort_by(|a, b| b.saved_at.cmp(&a.saved_at));
        Ok(games)
    }

    /// 删除保存的棋局
    pub fn delete_game(&self, game_id: &str) -> Result<()> {
        let filepath = self.saves_dir.join(game_id);
        
        if filepath.exists() {
            fs::remove_file(&filepath)
                .with_context(|| format!("删除文件失败: {:?}", filepath))?;
        }

        Ok(())
    }

    /// 获取存储目录路径
    pub fn saves_directory(&self) -> &Path {
        &self.saves_dir
    }
}

/// 保存的棋局信息
#[derive(Debug, Clone)]
pub struct SavedGameInfo {
    /// 棋局 ID（文件名）
    pub game_id: String,
    /// 红方玩家
    pub red_player: String,
    /// 黑方玩家
    pub black_player: String,
    /// 保存时间
    pub saved_at: DateTime<Utc>,
    /// 走法数量
    pub move_count: usize,
}

/// 获取跨平台存储目录
fn get_saves_directory() -> Result<PathBuf> {
    let app_data_dir = dirs::data_dir()
        .context("无法获取应用数据目录")?;
    
    Ok(app_data_dir.join("chinese-chess").join("saves"))
}

/// 生成文件名
fn generate_filename(timestamp: &DateTime<Utc>, red_player: &str, black_player: &str) -> String {
    let timestamp_str = timestamp.format("%Y%m%d_%H%M%S").to_string();
    
    // 清理玩家名称中的特殊字符
    let clean_red = sanitize_filename(red_player);
    let clean_black = sanitize_filename(black_player);
    
    format!("{}_{}vs{}.json", timestamp_str, clean_red, clean_black)
}

/// 清理文件名中的特殊字符
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new().expect("无法创建存储管理器")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::{MoveRecord, Position};
    use tempfile::TempDir;

    fn create_test_storage() -> (StorageManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager {
            saves_dir: temp_dir.path().to_path_buf(),
        };
        (storage, temp_dir)
    }

    #[test]
    fn test_save_and_load_game() {
        let (storage, _temp_dir) = create_test_storage();
        
        let mut record = GameRecord::new("玩家1".to_string(), "AI-中等".to_string());
        record.add_move(MoveRecord::new(
            Position::new_unchecked(7, 2),
            Position::new_unchecked(4, 2),
            "炮二平五".to_string(),
        ));

        let game_state = BoardState::initial();
        
        // 保存棋局
        let game_id = storage.save_game(
            "玩家1",
            "AI-中等", 
            &mut record,
            &game_state,
            600000,
            590000
        ).unwrap();

        // 加载棋局
        let loaded = storage.load_game(&game_id).unwrap();
        assert_eq!(loaded.metadata.red_player, "玩家1");
        assert_eq!(loaded.moves.len(), 1);
        assert!(loaded.save_info.is_some());
    }

    #[test]
    fn test_list_saved_games() {
        let (storage, _temp_dir) = create_test_storage();
        
        // 保存几个棋局
        for i in 1..=3 {
            let mut record = GameRecord::new(
                format!("玩家{}", i),
                "AI".to_string()
            );
            let game_state = BoardState::initial();
            
            storage.save_game(
                &format!("玩家{}", i),
                "AI",
                &mut record,
                &game_state,
                600000,
                600000
            ).unwrap();
        }

        let games = storage.list_saved_games().unwrap();
        assert_eq!(games.len(), 3);
        
        // 验证排序（最新的在前）
        for i in 0..games.len()-1 {
            assert!(games[i].saved_at >= games[i+1].saved_at);
        }
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("正常名称"), "正常名称");
        assert_eq!(sanitize_filename("包含/特殊\\字符"), "包含_特殊_字符");
        assert_eq!(sanitize_filename("AI:中等?"), "AI_中等_");
    }

    #[test]
    fn test_generate_filename() {
        let timestamp = DateTime::parse_from_rfc3339("2026-01-09T15:30:22Z")
            .unwrap()
            .with_timezone(&Utc);
        
        let filename = generate_filename(&timestamp, "玩家1", "AI-中等");
        assert!(filename.starts_with("20260109_153022_"));
        assert!(filename.contains("玩家1vs"));
        assert!(filename.ends_with(".json"));
    }

    #[test]
    fn test_save_load_integration() {
        let (storage, _temp_dir) = create_test_storage();
        
        // 创建一个包含多个走法的棋局
        let mut record = GameRecord::new("测试玩家".to_string(), "AI-困难".to_string());
        
        // 添加几个走法
        record.add_move(MoveRecord::new(
            Position::new_unchecked(7, 2),
            Position::new_unchecked(4, 2),
            "炮二平五".to_string(),
        ));
        record.add_move(MoveRecord::new(
            Position::new_unchecked(1, 9),
            Position::new_unchecked(2, 7),
            "馬8進7".to_string(),
        ));
        record.add_move(MoveRecord::new(
            Position::new_unchecked(6, 0),
            Position::new_unchecked(5, 2),
            "馬三進四".to_string(),
        ));

        let game_state = BoardState::initial();
        
        // 保存棋局
        let game_id = storage.save_game(
            "测试玩家",
            "AI-困难", 
            &mut record,
            &game_state,
            480000,  // 8分钟
            540000   // 9分钟
        ).unwrap();

        // 验证文件名格式
        assert!(game_id.contains("测试玩家vs"));
        assert!(game_id.contains("AI-困难"));
        assert!(game_id.ends_with(".json"));

        // 加载并验证
        let loaded = storage.load_game(&game_id).unwrap();
        assert_eq!(loaded.metadata.red_player, "测试玩家");
        assert_eq!(loaded.metadata.black_player, "AI-困难");
        assert_eq!(loaded.moves.len(), 3);
        
        // 验证保存信息
        let save_info = loaded.save_info.unwrap();
        assert_eq!(save_info.red_time_remaining_ms, 480000);
        assert_eq!(save_info.black_time_remaining_ms, 540000);
        assert_eq!(save_info.game_state, "red_turn");

        // 验证走法
        assert_eq!(loaded.moves[0].notation, "炮二平五");
        assert_eq!(loaded.moves[1].notation, "馬8進7");
        assert_eq!(loaded.moves[2].notation, "馬三進四");

        // 测试列表功能
        let games = storage.list_saved_games().unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].game_id, game_id);
        assert_eq!(games[0].move_count, 3);

        // 测试删除功能
        storage.delete_game(&game_id).unwrap();
        let games_after_delete = storage.list_saved_games().unwrap();
        assert_eq!(games_after_delete.len(), 0);
    }
}
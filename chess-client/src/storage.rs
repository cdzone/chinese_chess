//! 本地棋局存储系统
//!
//! 提供跨平台的棋局保存和加载功能（客户端本地存储）

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use protocol::{BoardState, GameRecord, Side};

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
        player_side: Side,
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
                Side::Red => "red_turn".to_string(),
                Side::Black => "black_turn".to_string(),
            },
            player_side: match player_side {
                Side::Red => "red".to_string(),
                Side::Black => "black".to_string(),
            },
            red_time_remaining_ms: red_time_ms,
            black_time_remaining_ms: black_time_ms,
        });

        // 序列化并保存
        let json_content = game_record.to_json().context("序列化棋谱失败")?;

        fs::write(&filepath, json_content)
            .with_context(|| format!("写入文件失败: {:?}", filepath))?;

        tracing::info!("棋局已保存: {}", filename);

        // 返回文件名（不含路径）
        Ok(filename)
    }

    /// 加载棋局
    pub fn load_game(&self, game_id: &str) -> Result<GameRecord> {
        let filepath = self.saves_dir.join(game_id);

        if !filepath.exists() {
            anyhow::bail!("棋局文件不存在: {}", game_id);
        }

        let content =
            fs::read_to_string(&filepath).with_context(|| format!("读取文件失败: {:?}", filepath))?;

        GameRecord::from_json(&content).context("解析棋谱文件失败")
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
                                saved_at: record
                                    .save_info
                                    .as_ref()
                                    .map(|s| s.saved_at)
                                    .unwrap_or_else(|| {
                                        // 使用文件修改时间作为后备
                                        entry
                                            .metadata()
                                            .and_then(|m| m.modified())
                                            .map(|t| DateTime::from(t))
                                            .unwrap_or_else(|_| Utc::now())
                                    }),
                                move_count: record.moves.len(),
                                ai_difficulty: record.metadata.ai_difficulty.clone(),
                            });
                        }
                        Err(e) => {
                            tracing::warn!("跳过损坏的棋局文件 {}: {}", filename, e);
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
            fs::remove_file(&filepath).with_context(|| format!("删除文件失败: {:?}", filepath))?;
            tracing::info!("棋局已删除: {}", game_id);
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
    /// AI 难度（如果是 PvE 模式）
    pub ai_difficulty: Option<String>,
}

impl SavedGameInfo {
    /// 格式化保存时间
    pub fn formatted_time(&self) -> String {
        self.saved_at.format("%Y-%m-%d %H:%M").to_string()
    }

    /// 获取显示名称
    pub fn display_name(&self) -> String {
        if let Some(ref diff) = self.ai_difficulty {
            format!("{} vs AI({})", self.red_player, diff)
        } else {
            format!("{} vs {}", self.red_player, self.black_player)
        }
    }
}

/// 获取跨平台存储目录
fn get_saves_directory() -> Result<PathBuf> {
    let app_data_dir = dirs::data_dir().context("无法获取应用数据目录")?;

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

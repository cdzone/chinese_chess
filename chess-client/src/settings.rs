//! 游戏设置模块
//!
//! 提供设置数据结构、持久化和 Bevy Resource 集成

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, VideoModeSelection, WindowMode};
use protocol::Difficulty;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 本地对局时间限制
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimeLimit {
    /// 无限制
    #[default]
    Unlimited,
    /// 10 分钟
    TenMinutes,
    /// 30 分钟
    ThirtyMinutes,
    /// 60 分钟
    SixtyMinutes,
}

impl TimeLimit {
    /// 转换为毫秒
    pub fn to_millis(self) -> u64 {
        match self {
            TimeLimit::Unlimited => u64::MAX,
            TimeLimit::TenMinutes => 10 * 60 * 1000,
            TimeLimit::ThirtyMinutes => 30 * 60 * 1000,
            TimeLimit::SixtyMinutes => 60 * 60 * 1000,
        }
    }

    /// 显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TimeLimit::Unlimited => "无限制",
            TimeLimit::TenMinutes => "10 分钟",
            TimeLimit::ThirtyMinutes => "30 分钟",
            TimeLimit::SixtyMinutes => "60 分钟",
        }
    }

    /// 所有选项
    pub fn all() -> &'static [TimeLimit] {
        &[
            TimeLimit::Unlimited,
            TimeLimit::TenMinutes,
            TimeLimit::ThirtyMinutes,
            TimeLimit::SixtyMinutes,
        ]
    }

    /// 下一个选项
    pub fn next(self) -> Self {
        match self {
            TimeLimit::Unlimited => TimeLimit::TenMinutes,
            TimeLimit::TenMinutes => TimeLimit::ThirtyMinutes,
            TimeLimit::ThirtyMinutes => TimeLimit::SixtyMinutes,
            TimeLimit::SixtyMinutes => TimeLimit::Unlimited,
        }
    }

    /// 上一个选项
    pub fn prev(self) -> Self {
        match self {
            TimeLimit::Unlimited => TimeLimit::SixtyMinutes,
            TimeLimit::TenMinutes => TimeLimit::Unlimited,
            TimeLimit::ThirtyMinutes => TimeLimit::TenMinutes,
            TimeLimit::SixtyMinutes => TimeLimit::ThirtyMinutes,
        }
    }
}

/// 翻转棋盘视角
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BoardFlip {
    /// 红方视角（红方在下）
    #[default]
    RedBottom,
    /// 黑方视角（黑方在下）
    BlackBottom,
    /// 自动（跟随执棋方）
    Auto,
}

impl BoardFlip {
    pub fn display_name(&self) -> &'static str {
        match self {
            BoardFlip::RedBottom => "红方视角",
            BoardFlip::BlackBottom => "黑方视角",
            BoardFlip::Auto => "自动",
        }
    }

    pub fn next(self) -> Self {
        match self {
            BoardFlip::RedBottom => BoardFlip::BlackBottom,
            BoardFlip::BlackBottom => BoardFlip::Auto,
            BoardFlip::Auto => BoardFlip::RedBottom,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            BoardFlip::RedBottom => BoardFlip::Auto,
            BoardFlip::BlackBottom => BoardFlip::RedBottom,
            BoardFlip::Auto => BoardFlip::BlackBottom,
        }
    }
}

/// 窗口分辨率预设
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Resolution {
    /// 1024x768
    R1024x768,
    /// 1280x720
    #[default]
    R1280x720,
    /// 1920x1080
    R1920x1080,
    /// 2560x1440
    R2560x1440,
}

impl Resolution {
    pub fn width(&self) -> u32 {
        match self {
            Resolution::R1024x768 => 1024,
            Resolution::R1280x720 => 1280,
            Resolution::R1920x1080 => 1920,
            Resolution::R2560x1440 => 2560,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Resolution::R1024x768 => 768,
            Resolution::R1280x720 => 720,
            Resolution::R1920x1080 => 1080,
            Resolution::R2560x1440 => 1440,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Resolution::R1024x768 => "1024 x 768",
            Resolution::R1280x720 => "1280 x 720",
            Resolution::R1920x1080 => "1920 x 1080",
            Resolution::R2560x1440 => "2560 x 1440",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Resolution::R1024x768 => Resolution::R1280x720,
            Resolution::R1280x720 => Resolution::R1920x1080,
            Resolution::R1920x1080 => Resolution::R2560x1440,
            Resolution::R2560x1440 => Resolution::R1024x768,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Resolution::R1024x768 => Resolution::R2560x1440,
            Resolution::R1280x720 => Resolution::R1024x768,
            Resolution::R1920x1080 => Resolution::R1280x720,
            Resolution::R2560x1440 => Resolution::R1920x1080,
        }
    }
}

/// 全屏模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FullscreenMode {
    /// 窗口模式
    #[default]
    Windowed,
    /// 无边框全屏
    BorderlessFullscreen,
    /// 独占全屏
    ExclusiveFullscreen,
}

impl FullscreenMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            FullscreenMode::Windowed => "窗口",
            FullscreenMode::BorderlessFullscreen => "无边框全屏",
            FullscreenMode::ExclusiveFullscreen => "独占全屏",
        }
    }

    pub fn to_window_mode(self) -> WindowMode {
        match self {
            FullscreenMode::Windowed => WindowMode::Windowed,
            FullscreenMode::BorderlessFullscreen => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            FullscreenMode::ExclusiveFullscreen => WindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current),
        }
    }

    pub fn next(self) -> Self {
        match self {
            FullscreenMode::Windowed => FullscreenMode::BorderlessFullscreen,
            FullscreenMode::BorderlessFullscreen => FullscreenMode::ExclusiveFullscreen,
            FullscreenMode::ExclusiveFullscreen => FullscreenMode::Windowed,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            FullscreenMode::Windowed => FullscreenMode::ExclusiveFullscreen,
            FullscreenMode::BorderlessFullscreen => FullscreenMode::Windowed,
            FullscreenMode::ExclusiveFullscreen => FullscreenMode::BorderlessFullscreen,
        }
    }
}

/// 帧率限制
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FrameRateLimit {
    /// 30 FPS
    Fps30,
    /// 60 FPS
    #[default]
    Fps60,
    /// 120 FPS
    Fps120,
    /// 无限制
    Unlimited,
}

impl FrameRateLimit {
    pub fn display_name(&self) -> &'static str {
        match self {
            FrameRateLimit::Fps30 => "30 FPS",
            FrameRateLimit::Fps60 => "60 FPS",
            FrameRateLimit::Fps120 => "120 FPS",
            FrameRateLimit::Unlimited => "无限制",
        }
    }

    pub fn to_fps(&self) -> Option<f64> {
        match self {
            FrameRateLimit::Fps30 => Some(30.0),
            FrameRateLimit::Fps60 => Some(60.0),
            FrameRateLimit::Fps120 => Some(120.0),
            FrameRateLimit::Unlimited => None,
        }
    }

    pub fn next(self) -> Self {
        match self {
            FrameRateLimit::Fps30 => FrameRateLimit::Fps60,
            FrameRateLimit::Fps60 => FrameRateLimit::Fps120,
            FrameRateLimit::Fps120 => FrameRateLimit::Unlimited,
            FrameRateLimit::Unlimited => FrameRateLimit::Fps30,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            FrameRateLimit::Fps30 => FrameRateLimit::Unlimited,
            FrameRateLimit::Fps60 => FrameRateLimit::Fps30,
            FrameRateLimit::Fps120 => FrameRateLimit::Fps60,
            FrameRateLimit::Unlimited => FrameRateLimit::Fps120,
        }
    }
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LogLevel {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn display_name(&self) -> &'static str {
        match self {
            LogLevel::Error => "Error",
            LogLevel::Warn => "Warn",
            LogLevel::Info => "Info",
            LogLevel::Debug => "Debug",
            LogLevel::Trace => "Trace",
        }
    }

    pub fn next(self) -> Self {
        match self {
            LogLevel::Error => LogLevel::Warn,
            LogLevel::Warn => LogLevel::Info,
            LogLevel::Info => LogLevel::Debug,
            LogLevel::Debug => LogLevel::Trace,
            LogLevel::Trace => LogLevel::Error,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            LogLevel::Error => LogLevel::Trace,
            LogLevel::Warn => LogLevel::Error,
            LogLevel::Info => LogLevel::Warn,
            LogLevel::Debug => LogLevel::Info,
            LogLevel::Trace => LogLevel::Debug,
        }
    }
}

/// 游戏设置
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct GameSettings {
    // === 游戏设置 ===
    /// 本地对局时间限制
    pub time_limit: TimeLimit,
    /// AI 思考时间上限（秒）
    pub ai_timeout_secs: u32,
    /// 默认 AI 难度
    pub default_difficulty: Difficulty,
    /// 棋子动画速度（0.5-2.0）
    pub animation_speed: f32,
    /// 走子提示
    pub show_move_hints: bool,
    /// 翻转棋盘
    pub board_flip: BoardFlip,

    // === 显示设置 ===
    /// 窗口分辨率
    pub resolution: Resolution,
    /// 全屏模式
    pub fullscreen_mode: FullscreenMode,
    /// 垂直同步
    pub vsync: bool,
    /// 帧率限制
    pub frame_rate_limit: FrameRateLimit,

    // === 音频设置（预留）===
    /// 主音量（0-100）
    pub master_volume: u32,
    /// 音效音量（0-100）
    pub sfx_volume: u32,
    /// 背景音乐开关
    pub music_enabled: bool,

    // === 网络设置 ===
    /// 默认服务器地址
    pub server_address: String,
    /// 默认昵称
    pub nickname: String,

    // === 高级设置 ===
    /// 日志级别
    pub log_level: LogLevel,
    /// 显示 FPS
    pub show_fps: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            // 游戏设置
            time_limit: TimeLimit::default(),
            ai_timeout_secs: 10,
            default_difficulty: Difficulty::Medium,
            animation_speed: 1.0,
            show_move_hints: true,
            board_flip: BoardFlip::default(),

            // 显示设置
            resolution: Resolution::default(),
            fullscreen_mode: FullscreenMode::default(),
            vsync: true,
            frame_rate_limit: FrameRateLimit::default(),

            // 音频设置
            master_volume: 100,
            sfx_volume: 100,
            music_enabled: true,

            // 网络设置
            server_address: "127.0.0.1:9527".to_string(),
            nickname: "玩家".to_string(),

            // 高级设置
            log_level: LogLevel::default(),
            show_fps: false,
        }
    }
}

impl GameSettings {
    /// 获取设置文件路径
    pub fn settings_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("chinese-chess");
            path.push("settings.json");
            path
        })
    }

    /// 从文件加载设置
    pub fn load() -> Self {
        let Some(path) = Self::settings_path() else {
            tracing::warn!("无法获取配置目录，使用默认设置");
            return Self::default();
        };

        if !path.exists() {
            tracing::info!("设置文件不存在，使用默认设置");
            return Self::default();
        }

        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(settings) => {
                    tracing::info!("已加载设置: {:?}", path);
                    settings
                }
                Err(e) => {
                    tracing::warn!("设置文件格式无效: {}，使用默认设置", e);
                    Self::default()
                }
            },
            Err(e) => {
                tracing::warn!("无法读取设置文件: {}，使用默认设置", e);
                Self::default()
            }
        }
    }

    /// 保存设置到文件
    pub fn save(&self) -> Result<(), String> {
        let Some(path) = Self::settings_path() else {
            return Err("无法获取配置目录".to_string());
        };

        // 确保目录存在
        if let Some(parent) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                return Err(format!("无法创建配置目录: {}", e));
            }
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化设置失败: {}", e))?;

        std::fs::write(&path, content)
            .map_err(|e| format!("写入设置文件失败: {}", e))?;

        tracing::info!("设置已保存: {:?}", path);
        Ok(())
    }

    /// 获取垂直同步的 PresentMode
    pub fn present_mode(&self) -> PresentMode {
        if self.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        }
    }
}

/// 设置插件
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        // 启动时加载设置
        let settings = GameSettings::load();
        app.insert_resource(settings);
    }
}

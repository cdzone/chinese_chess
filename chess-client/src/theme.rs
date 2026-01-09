//! 主题和配色方案
//!
//! 定义棋盘、棋子的颜色配置

use bevy::prelude::*;

/// 主题插件
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColorTheme::classic());
    }
}

/// 颜色主题配置
#[derive(Resource, Clone, Debug)]
pub struct ColorTheme {
    pub name: String,

    // 棋盘
    pub board_background: Color,
    pub board_lines: Color,
    pub river_text: Color,

    // 棋子
    pub piece_background: Color,
    pub piece_border: Color,
    pub red_piece_text: Color,
    pub black_piece_text: Color,

    // 交互高亮
    pub selected_highlight: Color,
    pub valid_move_indicator: Color,
    pub last_move_highlight: Color,
    pub check_warning: Color,
}

impl ColorTheme {
    /// 经典木质配色
    pub fn classic() -> Self {
        Self {
            name: "经典木质".to_string(),

            // 棋盘 - 温暖的木质色调
            board_background: Color::srgb_u8(222, 184, 135), // #DEB887 原木色
            board_lines: Color::srgb_u8(93, 64, 55),         // #5D4037 深褐色
            river_text: Color::srgb_u8(93, 64, 55),          // #5D4037 深褐色

            // 棋子 - 统一底色，文字颜色区分
            piece_background: Color::srgb_u8(255, 248, 231), // #FFF8E7 象牙白
            piece_border: Color::srgb_u8(78, 52, 46),        // #4E342E 深棕色
            red_piece_text: Color::srgb_u8(198, 40, 40),     // #C62828 朱红色
            black_piece_text: Color::srgb_u8(33, 33, 33),    // #212121 墨黑色

            // 交互高亮
            selected_highlight: Color::srgb_u8(255, 213, 79),  // #FFD54F 金黄色
            valid_move_indicator: Color::srgba_u8(129, 199, 132, 180), // #81C784 淡绿色半透明
            last_move_highlight: Color::srgba_u8(100, 181, 246, 150), // #64B5F6 淡蓝色
            check_warning: Color::srgb_u8(244, 67, 54),        // #F44336 红色警告
        }
    }

    /// 高对比度配色
    #[allow(dead_code)]
    pub fn high_contrast() -> Self {
        Self {
            name: "高对比度".to_string(),

            board_background: Color::srgb_u8(245, 245, 220), // #F5F5DC 浅米色
            board_lines: Color::srgb_u8(0, 0, 0),            // #000000 纯黑色
            river_text: Color::srgb_u8(0, 0, 0),

            piece_background: Color::srgb_u8(255, 255, 255), // 纯白
            piece_border: Color::srgb_u8(0, 0, 0),           // 纯黑
            red_piece_text: Color::srgb_u8(183, 28, 28),     // #B71C1C 深红色
            black_piece_text: Color::srgb_u8(0, 0, 0),       // #000000 纯黑色

            selected_highlight: Color::srgb_u8(255, 235, 59),
            valid_move_indicator: Color::srgba_u8(0, 200, 83, 200),
            last_move_highlight: Color::srgba_u8(33, 150, 243, 180),
            check_warning: Color::srgb_u8(213, 0, 0),
        }
    }
}

/// 显示设置
#[derive(Resource, Clone, Debug)]
pub struct DisplaySettings {
    /// UI 缩放比例 (1.0 = 100%)
    pub ui_scale: f32,
    /// 棋子文字加粗
    pub bold_piece_text: bool,
    /// 线条粗细 (1=细, 2=中, 3=粗)
    pub line_thickness: u8,
    /// 字体大小等级 (1=小, 2=中, 3=大, 4=特大)
    pub font_size_level: u8,
    /// 高对比度模式
    pub high_contrast: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            bold_piece_text: false,
            line_thickness: 2,
            font_size_level: 2,
            high_contrast: false,
        }
    }
}

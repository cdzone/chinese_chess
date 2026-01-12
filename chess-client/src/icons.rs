//! 图标符号映射模块
//!
//! 提供跨字体兼容的符号常量，解决 SourceHanSansSC 不支持 Emoji 的问题。
//!
//! # 使用说明
//!
//! 在 UI 中需要显示图标时，使用本模块提供的常量，而不是直接写 Emoji：
//!
//! ```ignore
//! use crate::icons;
//!
//! // 正确：使用常量
//! Text::new(icons::CLOSE);
//!
//! // 错误：直接使用 Emoji（可能显示为方框）
//! Text::new("✕");
//! ```
//!
//! # 字体兼容性
//!
//! 本模块中的所有符号均已验证在 SourceHanSansSC 字体中可正常显示。
//! 如需添加新符号，请先测试字体兼容性。

// ============================================================================
// 通用操作符号
// ============================================================================

/// 关闭按钮 - 乘号 (U+00D7)
pub const CLOSE: &str = "×";

/// 确认/成功 - 根号/对勾 (U+221A)
pub const CHECK: &str = "√";

/// 取消/删除 - 叉号（使用乘号）
pub const CANCEL: &str = "×";

/// 添加/新增 - 加号 (U+002B)
pub const ADD: &str = "+";

/// 减少 - 减号 (U+2212)
pub const MINUS: &str = "−";

/// 左箭头 (U+2190)
pub const ARROW_LEFT: &str = "←";

/// 右箭头 (U+2192)
pub const ARROW_RIGHT: &str = "→";

/// 上箭头 (U+2191)
pub const ARROW_UP: &str = "↑";

/// 下箭头 (U+2193)
pub const ARROW_DOWN: &str = "↓";

/// 左三角（上一个）(U+25C0)
pub const PREV: &str = "◀";

/// 右三角（下一个）(U+25B6)
pub const NEXT: &str = "▶";

// ============================================================================
// 状态指示符号
// ============================================================================

/// 精彩/优秀 - 实心星 (U+2605)
pub const STAR: &str = "★";

/// 空心星（用于评分）(U+2606)
pub const STAR_EMPTY: &str = "☆";

/// 错误/失误 - 乘号 (U+00D7)
pub const ERROR: &str = "×";

/// 警告/注意 - 实心菱形 (U+25C6)
pub const WARNING: &str = "◆";

/// 信息/提示 - 圆圈 (U+25CE)
pub const INFO: &str = "◎";

/// 等待/加载 - 圆点 (U+25CF)
pub const LOADING: &str = "●";

/// 成功 - 根号/对勾 (U+221A)
pub const SUCCESS: &str = "√";

// ============================================================================
// 游戏相关符号
// ============================================================================

/// 转折点/关键 - 菱形 (U+25C6)
pub const TURNING_POINT: &str = "◆";

/// 精彩走法 - 星号
pub const BRILLIANT: &str = "★";

/// 失误走法 - 乘号 (U+00D7)
pub const MISTAKE: &str = "×";

/// 红方标记 - 实心圆 (U+25CF)
pub const RED_MARKER: &str = "●";

/// 黑方标记 - 空心圆 (U+25CB)
pub const BLACK_MARKER: &str = "○";

/// 胜利
pub const VICTORY: &str = "★";

/// 失败
pub const DEFEAT: &str = "✗";

/// 平局 - 等号 (U+003D)
pub const DRAW: &str = "=";

// ============================================================================
// 装饰符号
// ============================================================================

/// 分隔线装饰 - 横线 (U+2500)
pub const LINE_H: &str = "─";

/// 竖线 (U+2502)
pub const LINE_V: &str = "│";

/// 左上角 (U+250C)
pub const CORNER_TL: &str = "┌";

/// 右上角 (U+2510)
pub const CORNER_TR: &str = "┐";

/// 左下角 (U+2514)
pub const CORNER_BL: &str = "└";

/// 右下角 (U+2518)
pub const CORNER_BR: &str = "┘";

/// 项目符号 - 实心圆点 (U+2022)
pub const BULLET: &str = "•";

/// 空心项目符号 (U+25E6)
pub const BULLET_EMPTY: &str = "◦";

// ============================================================================
// 辅助函数
// ============================================================================

/// 生成星级评分字符串
///
/// # 参数
/// - `score`: 评分（0.0 - max_stars）
/// - `max_stars`: 最大星数
///
/// # 示例
/// ```ignore
/// let rating = icons::star_rating(3.5, 5);
/// assert_eq!(rating, "★★★★☆");
/// ```
pub fn star_rating(score: f32, max_stars: u32) -> String {
    let full_stars = score.floor() as u32;
    let half_star = (score - score.floor()) >= 0.5;
    let empty_stars = max_stars.saturating_sub(full_stars + if half_star { 1 } else { 0 });

    let mut result = String::new();
    for _ in 0..full_stars {
        result.push_str(STAR);
    }
    if half_star {
        result.push_str(STAR); // 半星也用实心星表示，或可用其他符号
    }
    for _ in 0..empty_stars {
        result.push_str(STAR_EMPTY);
    }
    result
}

/// 获取关键时刻类型对应的图标
pub fn moment_type_icon(moment_type: &str) -> &'static str {
    match moment_type {
        "brilliant" => BRILLIANT,
        "mistake" => MISTAKE,
        "turning_point" => TURNING_POINT,
        _ => INFO,
    }
}

/// 获取评价等级对应的图标
pub fn evaluation_icon(evaluation: &str) -> &'static str {
    match evaluation {
        "好" | "优" | "excellent" | "good" => STAR,
        "中" | "average" | "fair" => INFO,
        "差" | "poor" | "bad" => WARNING,
        _ => BULLET,
    }
}

// ============================================================================
// Emoji 到 Unicode 符号映射表（供参考）
// ============================================================================
//
// 以下是常见 Emoji 到 SourceHanSansSC 兼容符号的映射：
//
// | Emoji | Unicode 替代 | 常量名        | 说明           |
// |-------|-------------|--------------|----------------|
// | 🌟    | ★ (U+2605)  | STAR         | 星星/精彩      |
// | ⭐    | ★ (U+2605)  | STAR         | 星星           |
// | ❌    | ✗ (U+2717)  | ERROR        | 错误/失误      |
// | ✕     | × (U+00D7)  | CLOSE        | 关闭           |
// | ✔️    | ✓ (U+2713)  | CHECK        | 确认/成功      |
// | ⚡    | ◆ (U+25C6)  | WARNING      | 闪电/转折      |
// | ⏳    | ◎ (U+25CE)  | INFO         | 沙漏/等待      |
// | ➕    | + (U+002B)  | ADD          | 添加           |
// | ➖    | − (U+2212)  | MINUS        | 减少           |
// | ⬅️    | ← (U+2190)  | ARROW_LEFT   | 左箭头         |
// | ➡️    | → (U+2192)  | ARROW_RIGHT  | 右箭头         |
// | ⬆️    | ↑ (U+2191)  | ARROW_UP     | 上箭头         |
// | ⬇️    | ↓ (U+2193)  | ARROW_DOWN   | 下箭头         |
// | ◀️    | ◀ (U+25C0)  | PREV         | 上一个         |
// | ▶️    | ▶ (U+25B6)  | NEXT         | 下一个         |
// | 🔴    | ● (U+25CF)  | RED_MARKER   | 红色圆点       |
// | ⚪    | ○ (U+25CB)  | BLACK_MARKER | 白色/空心圆点  |
// | ℹ️    | ◎ (U+25CE)  | INFO         | 信息           |
// | ⚠️    | ◆ (U+25C6)  | WARNING      | 警告           |
// | 🏆    | ★ (U+2605)  | VICTORY      | 奖杯/胜利      |
// | 💀    | ✗ (U+2717)  | DEFEAT       | 失败           |
//
// 添加新符号时，请：
// 1. 在 https://www.unicode.org/charts/ 查找合适的 Unicode 字符
// 2. 测试该字符在 SourceHanSansSC 字体中是否正常显示
// 3. 在本模块添加常量并更新映射表

// ============================================================================
// 运行时 Emoji 替换（用于处理外部输入如 LLM 返回内容）
// ============================================================================

/// Emoji 到 Unicode 符号的映射表
const EMOJI_REPLACEMENTS: &[(&str, &str)] = &[
    // 星星类
    ("🌟", STAR),
    ("⭐", STAR),
    ("🏆", STAR),
    // 错误/关闭类
    ("❌", ERROR),
    ("✕", CLOSE),
    ("💀", ERROR),
    // 确认类
    ("✔️", CHECK),
    ("✔", CHECK),
    // 警告/转折类
    ("⚡", WARNING),
    ("⚠️", WARNING),
    ("⚠", WARNING),
    // 信息/等待类
    ("⏳", INFO),
    ("ℹ️", INFO),
    ("ℹ", INFO),
    // 箭头类
    ("⬅️", ARROW_LEFT),
    ("⬅", ARROW_LEFT),
    ("➡️", ARROW_RIGHT),
    ("➡", ARROW_RIGHT),
    ("⬆️", ARROW_UP),
    ("⬆", ARROW_UP),
    ("⬇️", ARROW_DOWN),
    ("⬇", ARROW_DOWN),
    ("◀️", PREV),
    ("▶️", NEXT),
    // 加减类
    ("➕", ADD),
    ("➖", MINUS),
    // 圆点类
    ("🔴", RED_MARKER),
    ("⚪", BLACK_MARKER),
    // 其他常见 LLM 输出
    ("👍", CHECK),      // 点赞 → 对勾
    ("👎", ERROR),      // 踩 → 叉号
    ("💡", INFO),       // 灵感 → 信息
    ("🎯", STAR),       // 目标 → 星
    ("📌", BULLET),     // 标记 → 项目符号
    ("🔥", WARNING),    // 热门 → 菱形
    ("💪", STAR),       // 加油 → 星
    ("👀", INFO),       // 注意 → 信息
];

/// 将字符串中的 Emoji 替换为字体兼容的 Unicode 符号
///
/// 用于处理外部输入（如 LLM 返回的内容），确保在 SourceHanSansSC 字体中正常显示。
///
/// # 示例
/// ```
/// use chess_client::icons;
///
/// let text = "🌟 精彩走法！❌ 这是失误";
/// let safe_text = icons::sanitize(text);
/// assert_eq!(safe_text, "★ 精彩走法！× 这是失误");
/// ```
pub fn sanitize(text: &str) -> String {
    let mut result = text.to_string();
    let mut replaced = false;
    for (emoji, replacement) in EMOJI_REPLACEMENTS {
        if result.contains(emoji) {
            result = result.replace(emoji, replacement);
            replaced = true;
        }
    }
    if replaced {
        tracing::debug!("Sanitized text: replaced emojis");
    }
    result
}

/// 将字符串中的 Emoji 替换为字体兼容的 Unicode 符号（接受所有权版本）
///
/// 与 `sanitize` 功能相同，接受 `String` 所有权。
/// 注意：由于 `replace` 的实现，仍可能有内存分配。
pub fn sanitize_owned(mut text: String) -> String {
    for (emoji, replacement) in EMOJI_REPLACEMENTS {
        if text.contains(emoji) {
            text = text.replace(emoji, replacement);
        }
    }
    text
}

/// 对 GameAnalysis 中的所有文本字段进行 Emoji 替换
///
/// 在显示 LLM 分析结果前调用此函数，确保所有文本在 SourceHanSansSC 字体中正常显示。
///
/// # 示例
/// ```ignore
/// let analysis = llm_engine.analyze_game(...).await?;
/// let safe_analysis = icons::sanitize_analysis(analysis);
/// // 现在可以安全地显示 safe_analysis
/// ```
#[cfg(feature = "llm")]
pub fn sanitize_analysis(mut analysis: chess_ai::llm::GameAnalysis) -> chess_ai::llm::GameAnalysis {
    // 开局评价
    if let Some(ref mut name) = analysis.opening_review.name {
        *name = sanitize(name);
    }
    analysis.opening_review.evaluation = sanitize(&analysis.opening_review.evaluation);
    analysis.opening_review.comment = sanitize(&analysis.opening_review.comment);

    // 关键时刻
    for moment in &mut analysis.key_moments {
        moment.move_notation = sanitize(&moment.move_notation);
        moment.analysis = sanitize(&moment.analysis);
    }

    // 残局评价
    analysis.endgame_review.evaluation = sanitize(&analysis.endgame_review.evaluation);
    analysis.endgame_review.comment = sanitize(&analysis.endgame_review.comment);

    // 建议
    for suggestion in &mut analysis.suggestions.red {
        *suggestion = sanitize(suggestion);
    }
    for suggestion in &mut analysis.suggestions.black {
        *suggestion = sanitize(suggestion);
    }

    // 不足
    for weakness in &mut analysis.weaknesses.red {
        *weakness = sanitize(weakness);
    }
    for weakness in &mut analysis.weaknesses.black {
        *weakness = sanitize(weakness);
    }

    // 总评
    analysis.overall_rating.summary = sanitize(&analysis.overall_rating.summary);

    analysis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_star_rating() {
        assert_eq!(star_rating(0.0, 5), "☆☆☆☆☆");
        assert_eq!(star_rating(3.0, 5), "★★★☆☆");
        assert_eq!(star_rating(5.0, 5), "★★★★★");
        assert_eq!(star_rating(3.5, 5), "★★★★☆");
    }

    #[test]
    fn test_moment_type_icon() {
        assert_eq!(moment_type_icon("brilliant"), "★");
        assert_eq!(moment_type_icon("mistake"), "×");
        assert_eq!(moment_type_icon("turning_point"), "◆");
    }

    #[test]
    fn test_sanitize() {
        assert_eq!(sanitize("🌟 精彩"), "★ 精彩");
        assert_eq!(sanitize("❌ 失误"), "× 失误");
        assert_eq!(sanitize("⚡ 转折点"), "◆ 转折点");
        assert_eq!(sanitize("普通文本"), "普通文本");
        assert_eq!(sanitize("🌟❌⚡"), "★×◆");
    }

    #[test]
    fn test_sanitize_mixed() {
        let input = "第10步 🌟 红方走出精彩一招，但第15步 ❌ 出现失误";
        let expected = "第10步 ★ 红方走出精彩一招，但第15步 × 出现失误";
        assert_eq!(sanitize(input), expected);
    }
}

//! 对局复盘分析数据结构
//!
//! 定义 LLM 生成的对局分析报告结构

use serde::{Deserialize, Serialize};
use protocol::Side;

/// 对局复盘分析结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameAnalysis {
    /// 开局评价
    pub opening_review: OpeningReview,
    /// 关键时刻（精彩/失误/转折点）
    pub key_moments: Vec<KeyMoment>,
    /// 残局评价
    pub endgame_review: EndgameReview,
    /// 不足与有待提升
    #[serde(default)]
    pub weaknesses: Weaknesses,
    /// 改进建议
    pub suggestions: Suggestions,
    /// 整体评分
    pub overall_rating: OverallRating,
}

/// 开局评价
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpeningReview {
    /// 开局名称（如有）
    #[serde(default)]
    pub name: Option<String>,
    /// 评价（好/中/差）
    pub evaluation: String,
    /// 点评
    pub comment: String,
}

/// 关键时刻
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyMoment {
    /// 第几步
    pub move_number: u32,
    /// 哪方
    pub side: MomentSide,
    /// 走法记号
    #[serde(rename = "move")]
    pub move_notation: String,
    /// 类型
    #[serde(rename = "type")]
    pub moment_type: MomentType,
    /// 分析
    pub analysis: String,
}

/// 走棋方（用于 JSON 解析）
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MomentSide {
    Red,
    Black,
}

impl From<Side> for MomentSide {
    fn from(side: Side) -> Self {
        match side {
            Side::Red => MomentSide::Red,
            Side::Black => MomentSide::Black,
        }
    }
}

impl From<MomentSide> for Side {
    fn from(side: MomentSide) -> Self {
        match side {
            MomentSide::Red => Side::Red,
            MomentSide::Black => Side::Black,
        }
    }
}

/// 关键时刻类型
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MomentType {
    /// 精彩走法
    Brilliant,
    /// 失误
    Mistake,
    /// 转折点
    TurningPoint,
}

impl MomentType {
    /// 获取显示图标
    pub fn icon(&self) -> &'static str {
        match self {
            MomentType::Brilliant => "🌟",
            MomentType::Mistake => "❌",
            MomentType::TurningPoint => "⚡",
        }
    }

    /// 获取中文名称
    pub fn display_name(&self) -> &'static str {
        match self {
            MomentType::Brilliant => "精彩",
            MomentType::Mistake => "失误",
            MomentType::TurningPoint => "转折",
        }
    }
}

/// 残局评价
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EndgameReview {
    /// 评价
    pub evaluation: String,
    /// 点评
    pub comment: String,
}

/// 改进建议
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Suggestions {
    /// 给红方的建议
    pub red: Vec<String>,
    /// 给黑方的建议
    pub black: Vec<String>,
}

/// 不足与提升
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Weaknesses {
    /// 红方的不足
    #[serde(default)]
    pub red: Vec<String>,
    /// 黑方的不足
    #[serde(default)]
    pub black: Vec<String>,
}

/// 整体评分
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OverallRating {
    /// 红方棋力评分 (0-10)
    pub red_play_quality: f32,
    /// 黑方棋力评分 (0-10)
    pub black_play_quality: f32,
    /// 对局精彩度 (0-10)
    pub game_quality: f32,
    /// 总结
    pub summary: String,
}

impl OverallRating {
    /// 将评分转换为星级显示（10星制）
    pub fn stars(score: f32) -> String {
        let full_stars = score.floor() as usize;
        let half_star = (score - score.floor()) >= 0.5;
        let empty_stars = 10 - full_stars - if half_star { 1 } else { 0 };

        let mut result = String::new();
        for _ in 0..full_stars {
            result.push('★');
        }
        if half_star {
            result.push('☆');
        }
        for _ in 0..empty_stars {
            result.push('☆');
        }
        result
    }
}

impl Default for GameAnalysis {
    fn default() -> Self {
        Self {
            opening_review: OpeningReview {
                name: None,
                evaluation: "中".to_string(),
                comment: "无法分析开局".to_string(),
            },
            key_moments: Vec::new(),
            endgame_review: EndgameReview {
                evaluation: "中".to_string(),
                comment: "无法分析残局".to_string(),
            },
            weaknesses: Weaknesses::default(),
            suggestions: Suggestions {
                red: Vec::new(),
                black: Vec::new(),
            },
            overall_rating: OverallRating {
                red_play_quality: 5.0,
                black_play_quality: 5.0,
                game_quality: 5.0,
                summary: "无法生成总结".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moment_type_display() {
        assert_eq!(MomentType::Brilliant.icon(), "🌟");
        assert_eq!(MomentType::Mistake.display_name(), "失误");
        assert_eq!(MomentType::TurningPoint.display_name(), "转折");
    }

    #[test]
    fn test_stars_display() {
        assert_eq!(OverallRating::stars(7.5), "★★★★★★★☆☆☆");
        assert_eq!(OverallRating::stars(10.0), "★★★★★★★★★★");
        assert_eq!(OverallRating::stars(0.0), "☆☆☆☆☆☆☆☆☆☆");
        assert_eq!(OverallRating::stars(3.0), "★★★☆☆☆☆☆☆☆");
    }

    #[test]
    fn test_json_serialization() {
        let analysis = GameAnalysis::default();
        let json = serde_json::to_string(&analysis).unwrap();
        assert!(json.contains("opening_review"));
        assert!(json.contains("overall_rating"));
    }

    #[test]
    fn test_json_deserialization() {
        let json = r#"{
            "opening_review": {
                "name": "中炮对屏风马",
                "evaluation": "好",
                "comment": "标准开局"
            },
            "key_moments": [
                {
                    "move_number": 15,
                    "side": "red",
                    "move": "車一進三",
                    "type": "brilliant",
                    "analysis": "精彩的弃子战术"
                }
            ],
            "endgame_review": {
                "evaluation": "好",
                "comment": "残局处理得当"
            },
            "suggestions": {
                "red": ["注意防守"],
                "black": ["加强中局计算"]
            },
            "overall_rating": {
                "red_play_quality": 7.5,
                "black_play_quality": 6.0,
                "game_quality": 7.0,
                "summary": "精彩对局"
            }
        }"#;

        let analysis: GameAnalysis = serde_json::from_str(json).unwrap();
        assert_eq!(analysis.opening_review.name, Some("中炮对屏风马".to_string()));
        assert_eq!(analysis.key_moments.len(), 1);
        assert_eq!(analysis.key_moments[0].moment_type, MomentType::Brilliant);
        assert_eq!(analysis.overall_rating.red_play_quality, 7.5);
    }

    #[test]
    fn test_moment_side_conversion() {
        let red: MomentSide = Side::Red.into();
        assert_eq!(red, MomentSide::Red);

        let black: Side = MomentSide::Black.into();
        assert_eq!(black, Side::Black);
    }
}

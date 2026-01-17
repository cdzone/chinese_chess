//! LLM 提示模板
//!
//! 定义用于与 LLM 交互的提示格式，包括：
//! - 系统提示（角色设定）
//! - 棋局状态格式化
//! - 走法请求格式
//! - 对局复盘分析格式

use protocol::{Board, BoardState, Move, Side, Piece, Notation, Fen};

/// LLM 提示模板
pub struct PromptTemplate;

impl PromptTemplate {
    /// 系统提示：设定 LLM 作为象棋专家的角色
    pub fn system_prompt() -> &'static str {
        r#"你是一位中国象棋专家。你的任务是分析棋局并给出最佳走法。

规则提醒：
- 红方在下（y=0-4），黑方在上（y=5-9）
- 将/帅只能在九宫格内移动（x=3-5）
- 士/仕只能在九宫格内斜走
- 象/相不能过河，走田字，可被塞象眼
- 马走日字，可被蹩马腿
- 车走直线，不限距离
- 炮走直线，吃子需要隔一个棋子（炮架）
- 兵/卒过河前只能前进，过河后可左右平移

请严格按照要求的 JSON 格式返回走法。"#
    }

    /// 格式化棋局状态为 LLM 可理解的文本
    pub fn format_board_state(state: &BoardState) -> String {
        let mut result = String::new();
        
        // 棋盘可视化
        result.push_str("当前棋盘状态：\n");
        result.push_str(&Self::visualize_board(&state.board));
        result.push('\n');
        
        // FEN 表示
        result.push_str(&format!("FEN: {}\n", Fen::to_string(state)));
        
        // 当前走子方
        let side_name = match state.current_turn {
            Side::Red => "红方",
            Side::Black => "黑方",
        };
        result.push_str(&format!("轮到: {}\n", side_name));
        
        // 回合数
        result.push_str(&format!("回合: {}\n", state.round));
        
        result
    }

    /// 可视化棋盘
    fn visualize_board(board: &Board) -> String {
        use protocol::Position;
        let mut result = String::new();
        
        // 列标
        result.push_str("    0   1   2   3   4   5   6   7   8\n");
        result.push_str("  ┌───┬───┬───┬───┬───┬───┬───┬───┬───┐\n");
        
        for y in (0..10).rev() {
            result.push_str(&format!("{} │", y));
            for x in 0..9 {
                let pos = Position::new_unchecked(x, y);
                let cell = board.get(pos);
                let ch = match cell {
                    Some(piece) => Self::piece_to_char(&piece),
                    None => "  ".to_string(),
                };
                result.push_str(&format!("{}│", ch));
            }
            result.push('\n');
            
            if y == 5 {
                result.push_str("  ├───┴───┴───┴───┴───┴───┴───┴───┴───┤\n");
                result.push_str("  │         楚  河      汉  界         │\n");
                result.push_str("  ├───┬───┬───┬───┬───┬───┬───┬───┬───┤\n");
            } else if y > 0 {
                result.push_str("  ├───┼───┼───┼───┼───┼───┼───┼───┼───┤\n");
            }
        }
        result.push_str("  └───┴───┴───┴───┴───┴───┴───┴───┴───┘\n");
        
        result
    }

    /// 棋子转换为显示字符
    fn piece_to_char(piece: &Piece) -> String {
        let ch = piece.display_char();
        format!("{} ", ch)
    }

    /// 格式化走法历史
    #[allow(dead_code)]
    pub fn format_move_history(board: &Board, moves: &[Move]) -> String {
        if moves.is_empty() {
            return "历史走法: 无\n".to_string();
        }
        
        let mut result = String::from("历史走法:\n");
        let mut temp_board = board.clone();
        
        for (i, mv) in moves.iter().enumerate() {
            let notation = Notation::to_chinese(&temp_board, mv)
                .unwrap_or_else(|| format!("({},{})->({},{})", mv.from.x, mv.from.y, mv.to.x, mv.to.y));
            
            if i % 2 == 0 {
                result.push_str(&format!("{}. {}", i / 2 + 1, notation));
            } else {
                result.push_str(&format!("  {}\n", notation));
            }
            
            // 更新临时棋盘
            temp_board.move_piece(mv.from, mv.to);
        }
        
        if moves.len() % 2 == 1 {
            result.push('\n');
        }
        
        result
    }

    /// 生成走法请求提示
    pub fn move_request_prompt(state: &BoardState, move_history: &[Move]) -> String {
        let mut prompt = String::new();
        
        // 棋局状态
        prompt.push_str(&Self::format_board_state(state));
        prompt.push('\n');
        
        // 走法历史（最近10步）
        let recent_moves: Vec<Move> = move_history.iter().rev().take(10).rev().cloned().collect();
        if !recent_moves.is_empty() {
            // 需要从初始棋盘推演到当前状态前的棋盘
            prompt.push_str("最近走法: ");
            for (i, mv) in recent_moves.iter().enumerate() {
                if i > 0 {
                    prompt.push_str(", ");
                }
                prompt.push_str(&format!("({},{})->({},{})", mv.from.x, mv.from.y, mv.to.x, mv.to.y));
            }
            prompt.push('\n');
        }
        
        // 走法请求
        prompt.push_str("\n请分析当前局势，给出最佳走法。\n");
        prompt.push_str("返回格式（严格 JSON）:\n");
        prompt.push_str(r#"{"from": [x1, y1], "to": [x2, y2], "reason": "简短说明"}"#);
        prompt.push_str("\n\n注意：\n");
        prompt.push_str("- x 范围 0-8，y 范围 0-9\n");
        prompt.push_str("- 红方在下(y=0-4)，黑方在上(y=5-9)\n");
        prompt.push_str("- 只返回 JSON，不要其他文字\n");
        
        prompt
    }

    /// 生成对局总结请求提示
    pub fn game_summary_prompt(state: &BoardState, move_history: &[Move], result: &str) -> String {
        let mut prompt = String::new();
        
        prompt.push_str("请对以下中国象棋对局进行总结分析：\n\n");
        
        // 最终棋盘状态
        prompt.push_str(&Self::format_board_state(state));
        prompt.push('\n');
        
        // 完整走法历史
        prompt.push_str("完整走法记录:\n");
        for (i, mv) in move_history.iter().enumerate() {
            let side = if i % 2 == 0 { "红" } else { "黑" };
            prompt.push_str(&format!("{}. {} ({},{})->({},{})\n", 
                i / 2 + 1, side, mv.from.x, mv.from.y, mv.to.x, mv.to.y));
        }
        prompt.push('\n');
        
        // 对局结果
        prompt.push_str(&format!("对局结果: {}\n\n", result));
        
        // 请求总结
        prompt.push_str("请从以下几个方面进行分析：\n");
        prompt.push_str("1. 开局阶段评价\n");
        prompt.push_str("2. 中局关键转折点\n");
        prompt.push_str("3. 残局处理\n");
        prompt.push_str("4. 双方的精彩走法和失误\n");
        prompt.push_str("5. 总体评价和改进建议\n");
        
        prompt
    }

    /// 生成对局复盘分析提示（结构化 JSON 输出）
    pub fn game_analysis_prompt(
        state: &BoardState,
        initial_board: &Board,
        move_history: &[Move],
        result: &str,
        red_player: &str,
        black_player: &str,
    ) -> String {
        let mut prompt = String::new();

        prompt.push_str("请对以下中国象棋对局进行复盘分析：\n\n");

        // 对局信息
        prompt.push_str("对局信息：\n");
        prompt.push_str(&format!("- 红方：{}\n", red_player));
        prompt.push_str(&format!("- 黑方：{}\n", black_player));
        prompt.push_str(&format!("- 结果：{}\n", result));
        prompt.push_str(&format!("- 总步数：{}\n\n", move_history.len()));

        // 完整走法历史（带中文记谱）
        // 使用传入的初始棋盘而不是标准初始布局
        prompt.push_str("完整走法记录：\n");
        let mut temp_board = initial_board.clone();
        for (i, mv) in move_history.iter().enumerate() {
            let notation = Notation::to_chinese(&temp_board, mv)
                .unwrap_or_else(|| format!("({},{})->({},{})", mv.from.x, mv.from.y, mv.to.x, mv.to.y));

            if i % 2 == 0 {
                prompt.push_str(&format!("{}. {}", i / 2 + 1, notation));
            } else {
                prompt.push_str(&format!("  {}\n", notation));
            }

            // 更新临时棋盘
            temp_board.move_piece(mv.from, mv.to);
        }
        if move_history.len() % 2 == 1 {
            prompt.push('\n');
        }
        prompt.push('\n');

        // 最终棋盘状态
        prompt.push_str("最终棋盘状态：\n");
        prompt.push_str(&Self::visualize_board(&state.board));
        prompt.push('\n');

        // 请求结构化 JSON 输出
        prompt.push_str("请从以下几个方面进行分析，并以严格的 JSON 格式返回：\n\n");
        prompt.push_str(r#"{
  "opening_review": {
    "name": "开局名称（如有，如'中炮对屏风马'，没有则为null）",
    "evaluation": "好/中/差",
    "comment": "开局阶段点评"
  },
  "key_moments": [
    {
      "move_number": 步数,
      "side": "red或black",
      "move": "走法记号如'車一進三'",
      "type": "brilliant或mistake或turning_point",
      "analysis": "这步棋的分析说明"
    }
  ],
  "endgame_review": {
    "evaluation": "好/中/差",
    "comment": "残局阶段点评"
  },
  "weaknesses": {
    "red": ["红方的不足1：具体描述哪些方面需要提升", "不足2"],
    "black": ["黑方的不足1：具体描述哪些方面需要提升", "不足2"]
  },
  "suggestions": {
    "red": ["给红方的改进建议1", "建议2"],
    "black": ["给黑方的改进建议1", "建议2"]
  },
  "overall_rating": {
    "red_play_quality": 0-10的评分,
    "black_play_quality": 0-10的评分,
    "game_quality": 0-10的评分,
    "summary": "整体对局总结"
  }
}
"#);

        prompt.push_str("\n注意：\n");
        prompt.push_str("- key_moments 最多选取 5 个最重要的时刻\n");
        prompt.push_str("- weaknesses 指出双方在本局中暴露的不足，即使获胜方也要分析其可改进之处\n");
        prompt.push_str("- suggestions 给出针对性的提升建议\n");
        prompt.push_str("- 评分使用 0-10 的浮点数\n");
        prompt.push_str("- 只返回 JSON，不要其他文字\n");

        prompt
    }

    /// 复盘分析的系统提示
    pub fn analysis_system_prompt() -> &'static str {
        r#"你是一位资深的中国象棋教练和分析师。你的任务是对完整的象棋对局进行复盘分析，帮助棋手提高棋力。

分析要点：
1. 开局评价：识别开局类型，评价双方布局是否合理
2. 关键时刻：找出对局中的精彩走法、失误和转折点
3. 残局评价：评价残局阶段的处理
4. 改进建议：针对双方给出具体、可操作的提升建议
5. 整体评分：客观评价双方的棋力表现

请严格按照要求的 JSON 格式返回分析结果。"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::Position;

    #[test]
    fn test_system_prompt() {
        let prompt = PromptTemplate::system_prompt();
        assert!(prompt.contains("中国象棋专家"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_format_board_state() {
        let state = BoardState::initial();
        let formatted = PromptTemplate::format_board_state(&state);
        
        assert!(formatted.contains("当前棋盘状态"));
        assert!(formatted.contains("FEN:"));
        assert!(formatted.contains("轮到: 红方"));
        assert!(formatted.contains("楚  河"));
    }

    #[test]
    fn test_move_request_prompt() {
        let state = BoardState::initial();
        let moves = vec![];
        let prompt = PromptTemplate::move_request_prompt(&state, &moves);
        
        assert!(prompt.contains("请分析当前局势"));
        assert!(prompt.contains(r#""from""#));
        assert!(prompt.contains(r#""to""#));
    }

    #[test]
    fn test_game_summary_prompt() {
        let state = BoardState::initial();
        let moves = vec![
            Move::new(Position::new_unchecked(1, 2), Position::new_unchecked(4, 2)),
        ];
        let prompt = PromptTemplate::game_summary_prompt(&state, &moves, "红方胜");
        
        assert!(prompt.contains("对局进行总结分析"));
        assert!(prompt.contains("红方胜"));
        assert!(prompt.contains("开局阶段评价"));
    }

    #[test]
    fn test_game_analysis_prompt() {
        let state = BoardState::initial();
        let initial_board = Board::initial();
        let moves = vec![
            Move::new(Position::new_unchecked(1, 2), Position::new_unchecked(4, 2)),
            Move::new(Position::new_unchecked(1, 7), Position::new_unchecked(2, 5)),
        ];
        let prompt = PromptTemplate::game_analysis_prompt(
            &state, &initial_board, &moves, "红方胜（将死）", "玩家1", "AI-困难"
        );

        assert!(prompt.contains("复盘分析"));
        assert!(prompt.contains("红方：玩家1"));
        assert!(prompt.contains("黑方：AI-困难"));
        assert!(prompt.contains("opening_review"));
        assert!(prompt.contains("key_moments"));
        assert!(prompt.contains("overall_rating"));
    }

    #[test]
    fn test_analysis_system_prompt() {
        let prompt = PromptTemplate::analysis_system_prompt();
        assert!(prompt.contains("中国象棋教练"));
        assert!(prompt.contains("复盘分析"));
        assert!(prompt.contains("JSON"));
    }
}

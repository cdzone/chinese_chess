//! 复盘分析测试
//!
//! 运行方式:
//! ```bash
//! cargo run -p chess-ai --features llm --example test_analysis
//! ```

use chess_ai::llm::{LlmEngine, OllamaClient, OllamaConfig};
use protocol::{BoardState, Move, Position};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== 复盘分析测试 ===\n");

    // 1. 健康检查
    println!("1. 检查 Ollama 服务...");
    let client = OllamaClient::new(OllamaConfig::default())?;

    let start = std::time::Instant::now();
    match client.health_check().await {
        Ok(()) => {
            let elapsed = start.elapsed();
            println!("   健康检查耗时: {:?}", elapsed);
            println!("   ✅ Ollama 服务正常\n");
        }
        Err(e) => {
            let elapsed = start.elapsed();
            println!("   健康检查耗时: {:?}", elapsed);
            println!("   ❌ Ollama 服务不可用: {}", e);
            return Ok(());
        }
    }

    // 2. 创建引擎并测试 check_available
    println!("2. 测试 LlmEngine::check_available()...");
    let config = OllamaConfig {
        model: "qwen3:30b-a3b".to_string(),
        max_tokens: 4096,
        timeout_secs: 120,
        ..Default::default()
    };
    let mut engine = LlmEngine::new(config)?;
    
    let start = std::time::Instant::now();
    match engine.check_available().await {
        Ok(()) => {
            let elapsed = start.elapsed();
            println!("   check_available() 耗时: {:?}", elapsed);
            println!("   结果: ✅ 可用\n");
        }
        Err(e) => {
            let elapsed = start.elapsed();
            println!("   check_available() 耗时: {:?}", elapsed);
            println!("   结果: ❌ 不可用: {}\n", e);
            return Ok(());
        }
    }

    // 3. 模拟一局简单对局
    println!("3. 模拟对局历史...");
    let moves = vec![
        Move::new(Position::new(4, 6).unwrap(), Position::new(4, 5).unwrap()), // 兵五进一
        Move::new(Position::new(4, 3).unwrap(), Position::new(4, 4).unwrap()), // 卒5进1
        Move::new(Position::new(1, 0).unwrap(), Position::new(2, 2).unwrap()), // 马二进三
        Move::new(Position::new(1, 9).unwrap(), Position::new(2, 7).unwrap()), // 马2进3
    ];
    
    for mv in &moves {
        engine.add_move(*mv);
    }
    println!("   添加了 {} 步走法\n", moves.len());

    // 4. 执行复盘分析
    println!("4. 执行复盘分析 (可能需要 30-60 秒)...");
    let state = BoardState::initial();
    
    let start = std::time::Instant::now();
    match engine.analyze_game(&state, "红方胜", "测试玩家", "AI 对手").await {
        Ok(analysis) => {
            let elapsed = start.elapsed();
            println!("   ✅ 分析完成，耗时: {:?}\n", elapsed);
            
            println!("=== 分析结果 ===\n");
            println!("整体评分: {}", chess_ai::llm::OverallRating::stars(analysis.overall_rating.game_quality));
            println!("评价: {}\n", analysis.overall_rating.summary);
            
            println!("开局评价:");
            if let Some(name) = &analysis.opening_review.name {
                println!("  名称: {}", name);
            }
            println!("  评价: {}", analysis.opening_review.evaluation);
            println!("  点评: {}\n", analysis.opening_review.comment);
            
            println!("关键时刻: {} 个", analysis.key_moments.len());
            for (i, moment) in analysis.key_moments.iter().enumerate() {
                println!("  {}. {} [第{}步] {} - {}", 
                    i + 1, 
                    moment.moment_type.icon(),
                    moment.move_number,
                    moment.move_notation,
                    moment.analysis
                );
            }
            
            println!("\n改进建议:");
            println!("  红方: {:?}", analysis.suggestions.red);
            println!("  黑方: {:?}", analysis.suggestions.black);
        }
        Err(e) => {
            println!("   ❌ 分析失败: {}", e);
        }
    }

    println!("\n=== 测试完成 ===");
    Ok(())
}

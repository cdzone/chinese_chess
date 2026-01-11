//! LLM 功能测试示例
//!
//! 运行方式:
//! ```bash
//! cd chinese-chess
//! cargo run -p chess-ai --features llm --example llm_test
//!
//! # 指定模型
//! cargo run -p chess-ai --features llm --example llm_test -- qwen3:30b-a3b
//! ```

use chess_ai::llm::{LlmEngine, OllamaClient, OllamaConfig};
use protocol::BoardState;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    println!("=== 中国象棋 LLM 测试 ===\n");

    // 1. 检查 Ollama 服务
    println!("1. 检查 Ollama 服务...");
    let client = OllamaClient::new(OllamaConfig::default())?;

    if !client.health_check().await? {
        println!("   ❌ Ollama 服务未运行，请先执行: ollama serve");
        return Ok(());
    }
    println!("   ✅ Ollama 服务正常\n");

    // 2. 列出可用模型
    println!("2. 可用模型:");
    let models = client.list_models().await?;
    for model in &models {
        println!("   - {}", model);
    }
    println!();

    // 3. 选择模型（命令行参数或自动选择）
    let args: Vec<String> = env::args().collect();
    let model = if args.len() > 1 {
        args[1].clone()
    } else {
        // 自动选择第一个非 embedding 模型
        models
            .iter()
            .find(|m| !m.contains("embed"))
            .cloned()
            .unwrap_or_else(|| "qwen2.5:7b".to_string())
    };
    println!("3. 使用模型: {}\n", model);

    // 4. 测试走法生成
    println!("4. 测试走法生成...");
    let state = BoardState::initial();
    let config = OllamaConfig {
        model: model.clone(),
        max_tokens: 2048,  // 推理模型需要更多 tokens
        ..Default::default()
    };
    let engine = LlmEngine::new(config)?;

    println!("   初始棋盘，红方先行");
    println!("   请求 LLM 生成走法 (可能需要几秒钟)...\n");

    match engine.generate_move(&state).await {
        Ok(mv) => {
            println!("   ✅ LLM 生成走法: ({},{}) -> ({},{})",
                mv.from.x, mv.from.y, mv.to.x, mv.to.y);

            // 获取棋子名称
            if let Some(piece) = state.board.get(mv.from) {
                println!("   棋子: {:?}", piece.piece_type);
            }
        }
        Err(e) => {
            println!("   ❌ 生成失败: {}", e);
            println!("\n   提示: 可能是模型不支持或响应格式问题");
            println!("   建议尝试: cargo run -p chess-ai --features llm --example llm_test -- qwen3:30b-a3b");
        }
    }

    println!("\n=== 测试完成 ===");
    Ok(())
}

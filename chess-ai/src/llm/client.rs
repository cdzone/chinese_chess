//! Ollama REST API 客户端
//!
//! 与本地 Ollama 服务通信，发送提示并获取 LLM 响应。

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[cfg(feature = "llm")]
use anyhow::Context;
#[cfg(feature = "llm")]
use tracing::{debug, info, warn};

/// Ollama 客户端配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Ollama 服务地址，默认 http://localhost:11434
    pub base_url: String,
    /// 使用的模型名称，如 "llama3.2", "qwen2.5", "deepseek-r1"
    pub model: String,
    /// 生成温度，0.0-1.0，越低越确定性
    pub temperature: f32,
    /// 最大生成 token 数
    pub max_tokens: u32,
    /// 请求超时（秒）
    pub timeout_secs: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            model: "qwen2.5:7b".to_string(),
            temperature: 0.3,
            max_tokens: 512,
            timeout_secs: 60,
        }
    }
}

/// Ollama API 请求体
#[cfg(feature = "llm")]
#[derive(Serialize)]
struct GenerateRequest {
    model: String,
    prompt: String,
    system: Option<String>,
    stream: bool,
    options: GenerateOptions,
    /// 禁用 thinking 模式（qwen3 等模型支持）
    #[serde(skip_serializing_if = "Option::is_none")]
    think: Option<bool>,
}

#[cfg(feature = "llm")]
#[derive(Serialize)]
struct GenerateOptions {
    temperature: f32,
    num_predict: u32,
}

/// Ollama API 响应体
#[cfg(feature = "llm")]
#[derive(Deserialize)]
struct GenerateResponse {
    #[serde(default)]
    response: String,
    #[allow(dead_code)]
    done: bool,
    #[serde(default)]
    total_duration: u64,
    #[serde(default)]
    eval_count: u32,
    /// deepseek-r1 等推理模型的思考过程
    #[serde(default)]
    thinking: Option<String>,
}

/// Ollama 客户端
#[cfg(feature = "llm")]
pub struct OllamaClient {
    config: OllamaConfig,
    client: reqwest::Client,
}

#[cfg(feature = "llm")]
impl OllamaClient {
    /// 创建新的 Ollama 客户端
    pub fn new(config: OllamaConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client })
    }

    /// 使用默认配置创建客户端
    pub fn with_defaults() -> Result<Self> {
        Self::new(OllamaConfig::default())
    }

    /// 检查 Ollama 服务是否可用
    /// 使用独立的短超时客户端，避免长时间等待
    /// 返回 Ok(()) 表示服务可用，Err 表示不可用并包含具体原因
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/api/tags", self.config.base_url);
        
        // 创建一个短超时的客户端用于健康检查（5秒）
        let health_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .context("Failed to create health check client")?;
        
        let resp = health_client.get(&url).send().await
            .context(format!("无法连接到 Ollama 服务 ({})", self.config.base_url))?;
        
        if resp.status().is_success() {
            info!("Ollama health check passed");
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Ollama 服务返回错误状态: {} ({})", 
                resp.status(), 
                self.config.base_url
            ))
        }
    }

    /// 列出可用模型
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.config.base_url);
        
        #[derive(Deserialize)]
        struct TagsResponse {
            models: Vec<ModelInfo>,
        }
        
        #[derive(Deserialize)]
        struct ModelInfo {
            name: String,
        }
        
        let resp: TagsResponse = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to list models")?
            .json()
            .await
            .context("Failed to parse models response")?;
        
        Ok(resp.models.into_iter().map(|m| m.name).collect())
    }

    /// 发送生成请求
    pub async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String> {
        let url = format!("{}/api/generate", self.config.base_url);
        
        // 检测是否是 qwen3 模型，如果是则禁用 thinking 模式
        let is_qwen3 = self.config.model.to_lowercase().contains("qwen3");
        
        let request = GenerateRequest {
            model: self.config.model.clone(),
            prompt: prompt.to_string(),
            system: system.map(|s| s.to_string()),
            stream: false,
            options: GenerateOptions {
                temperature: self.config.temperature,
                num_predict: self.config.max_tokens,
            },
            // 对于 qwen3 等支持 thinking 的模型，显式禁用以获得完整的 JSON 输出
            think: if is_qwen3 { Some(false) } else { None },
        };

        debug!("Sending request to Ollama: model={}, prompt_len={}, think={:?}", 
            self.config.model, prompt.len(), request.think);

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send generate request")?;
        
        let response_text = response.text().await
            .context("Failed to read response body")?;
        
        // 安全截取（避免切到多字节字符中间）
        let preview: String = response_text.chars().take(500).collect();
        debug!("Raw Ollama response: {}", preview);
        
        let resp: GenerateResponse = serde_json::from_str(&response_text)
            .context("Failed to parse generate response")?;

        info!("Ollama response: tokens={}, duration={}ms", 
            resp.eval_count, resp.total_duration / 1_000_000);
        
        // 优先使用 response，如果为空则尝试从 thinking 中提取
        let output = if !resp.response.is_empty() {
            resp.response
        } else if let Some(thinking) = resp.thinking {
            // deepseek-r1 等模型可能把内容放在 thinking 字段
            debug!("Using thinking field as response");
            thinking
        } else {
            warn!("LLM returned empty response!");
            String::new()
        };
        
        if !output.is_empty() {
            let output_preview: String = output.chars().take(200).collect();
            debug!("LLM output: {}", output_preview);
        }

        Ok(output)
    }

    /// 获取当前配置
    pub fn config(&self) -> &OllamaConfig {
        &self.config
    }

    /// 设置模型
    pub fn set_model(&mut self, model: String) {
        self.config.model = model;
    }

    /// 设置温度
    pub fn set_temperature(&mut self, temperature: f32) {
        self.config.temperature = temperature.clamp(0.0, 1.0);
    }
}

/// 非 LLM feature 时的占位实现
#[cfg(not(feature = "llm"))]
pub struct OllamaClient {
    _config: OllamaConfig,
}

#[cfg(not(feature = "llm"))]
impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Result<Self> {
        Ok(Self { _config: config })
    }

    pub fn with_defaults() -> Result<Self> {
        Self::new(OllamaConfig::default())
    }

    pub async fn health_check(&self) -> Result<bool> {
        anyhow::bail!("LLM feature not enabled. Compile with --features llm")
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        anyhow::bail!("LLM feature not enabled. Compile with --features llm")
    }

    pub async fn generate(&self, _prompt: &str, _system: Option<&str>) -> Result<String> {
        anyhow::bail!("LLM feature not enabled. Compile with --features llm")
    }

    pub fn config(&self) -> &OllamaConfig {
        &self._config
    }

    pub fn set_model(&mut self, model: String) {
        self._config.model = model;
    }

    pub fn set_temperature(&mut self, temperature: f32) {
        self._config.temperature = temperature.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OllamaConfig::default();
        assert_eq!(config.base_url, "http://localhost:11434");
        assert!(config.temperature >= 0.0 && config.temperature <= 1.0);
    }

    #[test]
    fn test_client_creation() {
        let config = OllamaConfig::default();
        let _client = OllamaClient::new(config);
    }
}

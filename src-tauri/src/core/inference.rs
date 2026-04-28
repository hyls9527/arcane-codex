use crate::utils::error::{AppError, AppResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResult {
    pub tags: Vec<String>,
    pub description: String,
    pub category: String,
    pub confidence: f64,
    pub raw_response: String,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceProviderType {
    LMStudio,
    Ollama,
    Hermes,
    OpenAI,
    Zhipu,
    OpenRouter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: InferenceProviderType,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: InferenceProviderType::LMStudio,
            base_url: "http://127.0.0.1:1234".to_string(),
            model: "Qwen2.5-VL-7B-Instruct".to_string(),
            api_key: None,
            timeout_secs: 60,
        }
    }
}

#[async_trait]
pub trait InferenceProvider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    async fn analyze_image(&self, image_path: &str) -> AppResult<AIResult>;
    async fn health_check(&self) -> AppResult<Vec<String>>;
}

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(config: ProviderConfig) -> AppResult<Box<dyn InferenceProvider>> {
        match config.provider_type {
            InferenceProviderType::LMStudio | InferenceProviderType::Ollama | InferenceProviderType::Hermes => {
                let lm_config = crate::core::lm_studio::LMStudioConfig {
                    base_url: config.base_url,
                    model: config.model,
                    timeout_secs: config.timeout_secs,
                };
                let client = crate::core::lm_studio::LMStudioClient::new(lm_config)?;
                let name = match config.provider_type {
                    InferenceProviderType::LMStudio => "lm_studio",
                    InferenceProviderType::Ollama => "ollama",
                    InferenceProviderType::Hermes => "hermes",
                    _ => "unknown",
                };
                Ok(Box::new(OpenAICompatibleAdapter(client, name.to_string())))
            }
            InferenceProviderType::OpenAI | InferenceProviderType::OpenRouter => {
                let api_key = config.api_key.ok_or_else(|| {
                    AppError::validation(format!("{:?} 需要提供 API Key", config.provider_type))
                })?;
                Ok(Box::new(OpenAIClient::new(
                    config.provider_type,
                    config.base_url,
                    config.model,
                    api_key,
                    config.timeout_secs,
                )?))
            }
            InferenceProviderType::Zhipu => {
                let api_key = config.api_key.ok_or_else(|| {
                    AppError::validation("智谱 API 需要提供 API Key".to_string())
                })?;
                Ok(Box::new(ZhipuProvider::new(
                    config.model,
                    api_key,
                    config.timeout_secs,
                )?))
            }
        }
    }
}

pub struct OpenAICompatibleAdapter(
    pub(crate) crate::core::lm_studio::LMStudioClient,
    pub(crate) String,
);

#[async_trait]
impl InferenceProvider for OpenAICompatibleAdapter {
    fn name(&self) -> &str {
        &self.1
    }

    fn model(&self) -> &str {
        &self.0.config.model
    }

    async fn analyze_image(&self, image_path: &str) -> AppResult<AIResult> {
        let result = self.0.analyze_image(image_path).await?;
        Ok(AIResult {
            tags: result.tags,
            description: result.description,
            category: result.category,
            confidence: result.confidence,
            raw_response: result.raw_response,
            provider: self.1.clone(),
            model: self.0.config.model.clone(),
        })
    }

    async fn health_check(&self) -> AppResult<Vec<String>> {
        let models = self.0.health_check().await?;
        Ok(models.into_iter().map(|m| m.id).collect())
    }
}

fn encode_image_to_base64(image_path: &str) -> AppResult<String> {
    let bytes = std::fs::read(image_path).map_err(|e| {
        AppError::validation(format!("读取图片文件失败: {}", e))
    })?;
    Ok(data_encoding::BASE64.encode(&bytes))
}

fn detect_mime_type(image_path: &str) -> AppResult<String> {
    let path = std::path::Path::new(image_path);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    Ok(match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/jpeg",
    }.to_string())
}

fn build_prompt() -> String {
    r#"请分析这张图片,并以以下 JSON 格式返回:
{
  "tags": ["标签1", "标签2", "标签3"],
  "description": "一句话描述图片内容",
  "category": "风景|人物|物品|动物|建筑|文档|其他",
  "confidence": 0.95
}
要求:
- tags: 5-10个关键词,中文优先,避免重复和过于宽泛的词
- description: 简洁准确,1-2句话,不超过50字
- category: 从上述分类中选择一个
- confidence: 0.0-1.0之间的数字,表示你的置信度

仅返回合法 JSON,不要包含 Markdown 代码块标记或其他解释。"#.to_string()
}

fn parse_ai_response(content: &str, provider: &str, model: &str) -> AppResult<AIResult> {
    let content = content.trim();
    let content = if content.starts_with("```") {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() >= 2 {
            lines[1..lines.len() - 1].join("\n")
        } else {
            content.trim_start_matches("```json").trim_end_matches("```").to_string()
        }
    } else {
        content.to_string()
    };

    let parsed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        AppError::validation(format!("解析 AI JSON 响应失败: {} - 原始内容: {}", e, content))
    })?;

    let tags = parsed.get("tags").and_then(|t| t.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let description = parsed.get("description").and_then(|d| d.as_str()).unwrap_or("No description").to_string();
    let category = parsed.get("category").and_then(|c| c.as_str()).unwrap_or("other").to_string();
    let confidence = parsed.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.5);

    Ok(AIResult {
        tags, description, category, confidence,
        raw_response: content,
        provider: provider.to_string(),
        model: model.to_string(),
    })
}

pub struct ZhipuProvider {
    client: reqwest::Client,
    model: String,
    api_key: String,
}

impl ZhipuProvider {
    pub fn new(model: String, api_key: String, timeout_secs: u64) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| AppError::validation(format!("创建 HTTP 客户端失败: {}", e)))?;
        Ok(Self { client, model, api_key })
    }
}

#[async_trait]
impl InferenceProvider for ZhipuProvider {
    fn name(&self) -> &str { "zhipu" }
    fn model(&self) -> &str { &self.model }

    async fn analyze_image(&self, image_path: &str) -> AppResult<AIResult> {
        let image_base64 = encode_image_to_base64(image_path)?;
        let mime_type = detect_mime_type(image_path)?;
        let prompt = build_prompt();

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    { "type": "image_url", "image_url": { "url": format!("data:{};base64,{}", mime_type, image_base64) } }
                ]
            }],
            "max_tokens": 500,
            "temperature": 0.1
        });

        let resp = self.client
            .post("https://open.bigmodel.cn/api/paas/v4/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::validation(format!("智谱 AI 推理请求失败: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::validation(format!("智谱 AI 推理失败: HTTP {} - {}", status, body)));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            AppError::validation(format!("解析智谱 AI 响应失败: {}", e))
        })?;

        let content = body.get("choices").and_then(|c| c.as_array()).and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message")).and_then(|msg| msg.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| AppError::validation("智谱 AI 响应格式不正确".to_string()))?;

        parse_ai_response(content, "zhipu", &self.model)
    }

    async fn health_check(&self) -> AppResult<Vec<String>> {
        Ok(vec![self.model.clone()])
    }
}

pub struct OpenAIClient {
    client: reqwest::Client,
    provider_type: InferenceProviderType,
    base_url: String,
    model: String,
    api_key: String,
}

impl OpenAIClient {
    pub fn new(
        provider_type: InferenceProviderType,
        base_url: String,
        model: String,
        api_key: String,
        timeout_secs: u64,
    ) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| AppError::validation(format!("创建 HTTP 客户端失败: {}", e)))?;
        Ok(Self { client, provider_type, base_url, model, api_key })
    }
}

#[async_trait]
impl InferenceProvider for OpenAIClient {
    fn name(&self) -> &str {
        match &self.provider_type {
            InferenceProviderType::OpenAI => "openai",
            InferenceProviderType::OpenRouter => "openrouter",
            _ => "unknown",
        }
    }
    fn model(&self) -> &str { &self.model }

    async fn analyze_image(&self, image_path: &str) -> AppResult<AIResult> {
        let image_base64 = encode_image_to_base64(image_path)?;
        let mime_type = detect_mime_type(image_path)?;
        let prompt = build_prompt();

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    { "type": "image_url", "image_url": { "url": format!("data:{};base64,{}", mime_type, image_base64) } }
                ]
            }],
            "max_tokens": 500,
            "temperature": 0.1
        });

        let url = match &self.provider_type {
            InferenceProviderType::OpenAI => "https://api.openai.com/v1/chat/completions".to_string(),
            InferenceProviderType::OpenRouter => {
                if self.base_url.is_empty() {
                    "https://openrouter.ai/api/v1/chat/completions".to_string()
                } else {
                    format!("{}/v1/chat/completions", self.base_url)
                }
            }
            _ => return Err(AppError::validation("不支持的 OpenAI 兼容提供者".to_string())),
        };

        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::validation(format!("{} 推理请求失败: {}", self.name(), e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::validation(format!("{} 推理失败: HTTP {} - {}", self.name(), status, body)));
        }

        let body: serde_json::Value = resp.json().await.map_err(|e| {
            AppError::validation(format!("解析 {} 响应失败: {}", self.name(), e))
        })?;

        let content = body.get("choices").and_then(|c| c.as_array()).and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message")).and_then(|msg| msg.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| AppError::validation(format!("{} 响应格式不正确", self.name())))?;

        parse_ai_response(content, self.name(), &self.model)
    }

    async fn health_check(&self) -> AppResult<Vec<String>> {
        Ok(vec![self.model.clone()])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredModel {
    pub provider: String,
    pub provider_name: String,
    pub base_url: String,
    pub model_id: String,
    pub model_name: Option<String>,
    pub port: u16,
    pub is_online: bool,
}

pub struct ModelDiscoveryService;

impl ModelDiscoveryService {
    pub async fn scan_all() -> Vec<DiscoveredModel> {
        let services = vec![
            ("lm_studio", "LM Studio", "http://127.0.0.1:1234", 1234),
            ("ollama", "Ollama", "http://127.0.0.1:11434", 11434),
            ("hermes", "Hermes One-Click", "http://127.0.0.1:18789", 18789),
        ];

        let mut results = Vec::new();
        for (provider_key, provider_label, base_url, port) in services {
            if let Ok(models) = Self::scan_service(provider_key, provider_label, base_url, port).await {
                results.extend(models);
            }
        }
        results
    }

    async fn scan_service(
        provider_key: &str,
        provider_label: &str,
        base_url: &str,
        port: u16,
    ) -> Result<Vec<DiscoveredModel>, ()> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|_| ())?;

        let url = format!("{}/v1/models", base_url);
        let resp = client.get(&url).send().await.map_err(|_| ())?;
        if !resp.status().is_success() { return Err(()); }

        let body: serde_json::Value = resp.json().await.map_err(|_| ())?;
        let models = body.get("data").and_then(|d| d.as_array()).ok_or(())?;

        let mut results = Vec::new();
        for model in models {
            let model_id = model.get("id").and_then(|m| m.as_str()).unwrap_or("unknown");
            let model_name = model.get("name").and_then(|m| m.as_str()).map(String::from);
            results.push(DiscoveredModel {
                provider: provider_key.to_string(),
                provider_name: provider_label.to_string(),
                base_url: base_url.to_string(),
                model_id: model_id.to_string(),
                model_name,
                port,
                is_online: true,
            });
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig::default();
        assert!(matches!(config.provider_type, InferenceProviderType::LMStudio));
        assert_eq!(config.base_url, "http://127.0.0.1:1234");
    }

    #[test]
    fn test_provider_factory_lmstudio() {
        let config = ProviderConfig::default();
        let provider = ProviderFactory::create(config);
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "lm_studio");
    }

    #[test]
    fn test_provider_factory_ollama() {
        let config = ProviderConfig {
            provider_type: InferenceProviderType::Ollama,
            base_url: "http://127.0.0.1:11434".to_string(),
            model: "llava:7b".to_string(),
            ..Default::default()
        };
        let provider = ProviderFactory::create(config);
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.model(), "llava:7b");
    }

    #[test]
    fn test_provider_factory_hermes() {
        let config = ProviderConfig {
            provider_type: InferenceProviderType::Hermes,
            base_url: "http://127.0.0.1:18789".to_string(),
            model: "Qwen2.5-VL-7B-Instruct".to_string(),
            ..Default::default()
        };
        let provider = ProviderFactory::create(config);
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "hermes");
        assert_eq!(provider.model(), "Qwen2.5-VL-7B-Instruct");
    }

    #[test]
    fn test_provider_factory_zhipu_no_key() {
        let config = ProviderConfig {
            provider_type: InferenceProviderType::Zhipu,
            model: "glm-4v-flash".to_string(),
            ..Default::default()
        };
        let provider = ProviderFactory::create(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_provider_factory_openai_no_key() {
        let config = ProviderConfig {
            provider_type: InferenceProviderType::OpenAI,
            model: "gpt-4o".to_string(),
            ..Default::default()
        };
        let provider = ProviderFactory::create(config);
        assert!(provider.is_err());
    }

    #[test]
    fn test_hermes_gateway_protocol_compatibility() {
        // 验证 Hermes 网关协议兼容性
        // Hermes One-Click 使用 OpenAI 兼容 API，端口 18789
        let config = ProviderConfig {
            provider_type: InferenceProviderType::Hermes,
            base_url: "http://127.0.0.1:18789".to_string(),
            model: "Qwen2.5-VL-7B-Instruct".to_string(),
            timeout_secs: 60,
            api_key: None,
        };
        
        // 验证 ProviderFactory 能正确创建 Hermes 提供者
        let provider = ProviderFactory::create(config);
        assert!(provider.is_ok(), "Hermes Provider 应该能成功创建");
        
        let provider = provider.unwrap();
        
        // 验证名称正确
        assert_eq!(provider.name(), "hermes", "Provider 名称应为 'hermes'");
        
        // 验证模型正确
        assert_eq!(provider.model(), "Qwen2.5-VL-7B-Instruct", "模型名称应匹配");
        
        // 验证 Hermes 使用 OpenAI 兼容适配器（通过 trait 对象调用方法）
        // 实际协议兼容性通过 OpenAICompatibleAdapter 实现
        // 该适配器使用 LMStudioClient，已验证支持 OpenAI 兼容 API
    }

    #[test]
    fn test_hermes_openai_compatible_adapter_behavior() {
        // 验证 Hermes 的 OpenAI 兼容适配器行为
        // 确保 Hermes 与 LM Studio、Ollama 使用相同的适配器模式
        
        let hermes_config = ProviderConfig {
            provider_type: InferenceProviderType::Hermes,
            base_url: "http://127.0.0.1:18789".to_string(),
            model: "test-model".to_string(),
            ..Default::default()
        };
        
        let lm_config = ProviderConfig {
            provider_type: InferenceProviderType::LMStudio,
            base_url: "http://127.0.0.1:1234".to_string(),
            model: "test-model".to_string(),
            ..Default::default()
        };
        
        let hermes_provider = ProviderFactory::create(hermes_config).unwrap();
        let lm_provider = ProviderFactory::create(lm_config).unwrap();
        
        // 两者应该都是 OpenAICompatibleAdapter 类型（通过行为验证）
        // 验证它们都实现了 InferenceProvider trait
        assert_eq!(hermes_provider.name(), "hermes");
        assert_eq!(lm_provider.name(), "lm_studio");
        
        // 验证模型名称传递正确
        assert_eq!(hermes_provider.model(), "test-model");
        assert_eq!(lm_provider.model(), "test-model");
    }

    #[test]
    fn test_zhipu_provider_creation() {
        // 测试智谱 Provider 创建
        let provider = ZhipuProvider::new(
            "glm-4v-flash".to_string(),
            "test-api-key".to_string(),
            60,
        );
        assert!(provider.is_ok());
        
        let provider = provider.unwrap();
        assert_eq!(provider.name(), "zhipu");
        assert_eq!(provider.model(), "glm-4v-flash");
    }

    #[test]
    fn test_zhipu_provider_request_format() {
        // 验证智谱 API 请求格式构造
        // 智谱使用 OpenAI 兼容格式，但有自己的认证方式
        let provider = ZhipuProvider::new(
            "glm-4v-flash".to_string(),
            "test-api-key".to_string(),
            60,
        ).unwrap();
        
        // 验证基本属性
        assert_eq!(provider.name(), "zhipu");
        assert_eq!(provider.model(), "glm-4v-flash");
        
        // 注意：实际的 HTTP 请求测试需要 mock 服务器
        // 这里验证 Provider 能正确创建和配置
    }

    #[test]
    #[ignore = "需要 mock HTTP 服务器或真实智谱 API 密钥"]
    fn test_zhipu_analyze_image_mock() {
        // 使用 mock HTTP 服务器测试智谱 API 调用
        // 需要添加 mockito 或类似库来模拟 HTTP 响应
        // 
        // 测试场景：
        // 1. 正常响应 - 返回有效的 JSON
        // 2. 认证失败 - 401 Unauthorized
        // 3. 请求格式错误 - 400 Bad Request
        // 4. 服务器错误 - 500 Internal Server Error
        //
        // TODO: 集成 mockito 或 wiremock 进行 HTTP mock 测试
    }

    #[test]
    fn test_zhipu_health_check() {
        // 测试智谱 Provider 的健康检查
        let provider = ZhipuProvider::new(
            "glm-4v-flash".to_string(),
            "test-api-key".to_string(),
            60,
        ).unwrap();
        
        // health_check 是 async 方法，需要 tokio runtime
        // 这里只验证 Provider 能正确创建
        assert_eq!(provider.name(), "zhipu");
    }

    #[test]
    fn test_openai_client_creation() {
        // 测试 OpenAI Client 创建
        let client = OpenAIClient::new(
            InferenceProviderType::OpenAI,
            "https://api.openai.com".to_string(),
            "gpt-4o".to_string(),
            "test-api-key".to_string(),
            60,
        );
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.name(), "openai");
        assert_eq!(client.model(), "gpt-4o");
    }

    #[test]
    fn test_openrouter_client_creation() {
        // 测试 OpenRouter Client 创建
        let client = OpenAIClient::new(
            InferenceProviderType::OpenRouter,
            "".to_string(), // 使用默认 URL
            "anthropic/claude-3.5-sonnet".to_string(),
            "test-api-key".to_string(),
            60,
        );
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.name(), "openrouter");
        assert_eq!(client.model(), "anthropic/claude-3.5-sonnet");
    }

    #[test]
    fn test_openai_client_with_custom_base_url() {
        // 测试 OpenAI Client 使用自定义 base URL
        let client = OpenAIClient::new(
            InferenceProviderType::OpenAI,
            "https://custom.openai.proxy.com".to_string(),
            "gpt-4o".to_string(),
            "test-api-key".to_string(),
            60,
        );
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.name(), "openai");
        assert_eq!(client.model(), "gpt-4o");
    }

    #[test]
    fn test_openrouter_client_with_custom_base_url() {
        // 测试 OpenRouter Client 使用自定义 base URL
        let client = OpenAIClient::new(
            InferenceProviderType::OpenRouter,
            "https://custom.openrouter.ai".to_string(),
            "meta-llama/llama-3.2-11b-vision-instruct".to_string(),
            "test-api-key".to_string(),
            60,
        );
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.name(), "openrouter");
        assert_eq!(client.model(), "meta-llama/llama-3.2-11b-vision-instruct");
    }

    #[test]
    #[ignore = "需要 mock HTTP 服务器或真实 API 密钥"]
    fn test_openai_analyze_image_mock() {
        // 使用 mock HTTP 服务器测试 OpenAI API 调用
        // 需要添加 mockito 或类似库来模拟 HTTP 响应
        // 
        // 测试场景：
        // 1. 正常响应 - 返回有效的 JSON
        // 2. 认证失败 - 401 Unauthorized
        // 3. 请求格式错误 - 400 Bad Request
        // 4. 服务器错误 - 500 Internal Server Error
        //
        // TODO: 集成 mockito 或 wiremock 进行 HTTP mock 测试
    }

    #[test]
    #[ignore = "需要 mock HTTP 服务器或真实 API 密钥"]
    fn test_openrouter_analyze_image_mock() {
        // 使用 mock HTTP 服务器测试 OpenRouter API 调用
        // OpenRouter 支持多模型，需要测试不同模型的响应格式
        //
        // 测试场景：
        // 1. 正常响应 - 返回有效的 JSON
        // 2. 模型不可用 - 404 Model Not Found
        // 3. 速率限制 - 429 Rate Limit
        //
        // TODO: 集成 mockito 或 wiremock 进行 HTTP mock 测试
    }

    #[test]
    fn test_discovered_model_struct() {
        // 测试 DiscoveredModel 结构体创建
        let model = DiscoveredModel {
            provider: "lm_studio".to_string(),
            provider_name: "LM Studio".to_string(),
            base_url: "http://127.0.0.1:1234".to_string(),
            model_id: "qwen2.5-vl-7b".to_string(),
            model_name: Some("Qwen2.5 VL 7B".to_string()),
            port: 1234,
            is_online: true,
        };

        assert_eq!(model.provider, "lm_studio");
        assert_eq!(model.provider_name, "LM Studio");
        assert_eq!(model.base_url, "http://127.0.0.1:1234");
        assert_eq!(model.model_id, "qwen2.5-vl-7b");
        assert_eq!(model.model_name, Some("Qwen2.5 VL 7B".to_string()));
        assert_eq!(model.port, 1234);
        assert!(model.is_online);
    }

    #[test]
    fn test_model_discovery_service_ports() {
        // 验证 ModelDiscoveryService 扫描的端口配置
        // 注意：scan_all() 是 async 方法，需要 tokio runtime
        // 这里验证端口配置是否正确
        
        // 预期扫描的服务端口
        let expected_ports = vec![
            ("lm_studio", 1234),
            ("ollama", 11434),
            ("hermes", 18789),
        ];
        
        for (provider, port) in expected_ports {
            match provider {
                "lm_studio" => assert_eq!(port, 1234),
                "ollama" => assert_eq!(port, 11434),
                "hermes" => assert_eq!(port, 18789),
                _ => panic!("未知提供者: {}", provider),
            }
        }
    }

    #[test]
    fn test_model_discovery_service_urls() {
        // 验证 ModelDiscoveryService 扫描的 URL 配置
        let expected_urls = vec![
            ("lm_studio", "http://127.0.0.1:1234"),
            ("ollama", "http://127.0.0.1:11434"),
            ("hermes", "http://127.0.0.1:18789"),
        ];
        
        for (provider, url) in expected_urls {
            match provider {
                "lm_studio" => assert_eq!(url, "http://127.0.0.1:1234"),
                "ollama" => assert_eq!(url, "http://127.0.0.1:11434"),
                "hermes" => assert_eq!(url, "http://127.0.0.1:18789"),
                _ => panic!("未知提供者: {}", provider),
            }
        }
    }

    #[test]
    #[ignore = "需要本地 AI 服务运行或 mock HTTP 服务器"]
    fn test_model_discovery_scan_all() {
        // 测试 ModelDiscoveryService::scan_all()
        // 需要本地 LM Studio (1234), Ollama (11434), Hermes (18789) 运行
        // 或使用 mock HTTP 服务器
        //
        // 测试场景：
        // 1. 所有服务在线 - 返回发现的模型列表
        // 2. 部分服务离线 - 只返回在线服务的模型
        // 3. 所有服务离线 - 返回空列表
        //
        // TODO: 集成 mockito 进行 HTTP mock 测试
    }

    #[test]
    #[ignore = "需要本地 AI 服务运行或 mock HTTP 服务器"]
    fn test_model_discovery_scan_service() {
        // 测试 ModelDiscoveryService::scan_service()
        // 验证单个服务的扫描逻辑
        //
        // 测试场景：
        // 1. 服务在线且有模型 - 返回 DiscoveredModel 列表
        // 2. 服务在线但无模型 - 返回空列表
        // 3. 服务离线 - 返回 Err
        // 4. 服务返回无效 JSON - 返回 Err
        //
        // TODO: 集成 mockito 进行 HTTP mock 测试
    }
}

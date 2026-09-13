use thiserror::Error;
use serde_json::Value;
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("provider not configured: {0}")]
    NotConfigured(&'static str),
    #[error("http error: {0}")]
    Http(String),
    #[error("unsupported operation: {0}")]
    Unsupported(&'static str),
}

#[derive(Debug, Clone)]
pub enum Provider {
    OpenRouter { api_key: String, base_url: String },
    DeepInfra { api_key: String, base_url: String },
    OpenAI { api_key: String, base_url: String },
    Anthropic { api_key: String, base_url: String },
    Gemini { api_key: String, base_url: String },
    Ollama { base_url: String },
    ComfyUI { base_url: String },
    LocalPiper,
    LocalAudioCraft,
    Mock,
}

impl Provider {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "openrouter" => Some(Provider::OpenRouter {
                api_key: String::new(),
                base_url: "https://openrouter.ai/api/v1".to_string(),
            }),
            "deepinfra" => Some(Provider::DeepInfra {
                api_key: String::new(),
                base_url: "https://api.deepinfra.com/v1/openai".to_string(),
            }),
            "openai" => Some(Provider::OpenAI {
                api_key: String::new(),
                base_url: "https://api.openai.com/v1".to_string(),
            }),
            "anthropic" => Some(Provider::Anthropic {
                api_key: String::new(),
                base_url: "https://api.anthropic.com/v1".to_string(),
            }),
            "gemini" => Some(Provider::Gemini {
                api_key: String::new(),
                base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            }),
            "ollama_local" => Some(Provider::Ollama {
                base_url: "http://127.0.0.1:11434".to_string(),
            }),
            "comfyui_local" => Some(Provider::ComfyUI {
                base_url: "http://127.0.0.1:8188".to_string(),
            }),
            "local_piper" => Some(Provider::LocalPiper),
            "local_audiocraft" => Some(Provider::LocalAudioCraft),
            "mock" => Some(Provider::Mock),
            _ => None,
        }
    }

    pub async fn complete(&self, model: &str, prompt: &str) -> Result<String, AiError> {
        match self {
            Provider::OpenRouter { api_key, base_url } => {
                let client = Client::new();
                let url = format!("{}/chat/completions", base_url);
                let body = serde_json::json!({ "model": model, "messages": [{ "role": "user", "content": prompt }] });
                let response = client
                    .post(&url)
                    .bearer_auth(api_key)
                    .json(&body)
                    .timeout(Duration::from_secs(30))
                    .send()
                    .await
                    .map_err(|e| AiError::Http(e.to_string()))?;
                let response_json: Value = response.json().await.map_err(|e| AiError::Http(e.to_string()))?;
                Ok(response_json["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string())
            }
            Provider::DeepInfra { api_key, base_url } => {
                let client = Client::new();
                let url = format!("{}/chat/completions", base_url);
                let body = serde_json::json!({ "model": model, "messages": [{ "role": "user", "content": prompt }] });
                let response = client
                    .post(&url)
                    .bearer_auth(api_key)
                    .json(&body)
                    .timeout(Duration::from_secs(30))
                    .send()
                    .await
                    .map_err(|e| AiError::Http(e.to_string()))?;
                let response_json: Value = response.json().await.map_err(|e| AiError::Http(e.to_string()))?;
                Ok(response_json["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string())
            }
            Provider::OpenAI { api_key, base_url } => {
                let client = Client::new();
                let url = format!("{}/chat/completions", base_url);
                let body = serde_json::json!({ "model": model, "messages": [{ "role": "user", "content": prompt }] });
                let response = client
                    .post(&url)
                    .bearer_auth(api_key)
                    .json(&body)
                    .timeout(Duration::from_secs(30))
                    .send()
                    .await
                    .map_err(|e| AiError::Http(e.to_string()))?;
                let response_json: Value = response.json().await.map_err(|e| AiError::Http(e.to_string()))?;
                Ok(response_json["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string())
            }
            Provider::Anthropic { api_key, base_url } => {
                let client = Client::new();
                let url = format!("{}/messages", base_url);
                let body = serde_json::json!({ "model": model, "max_tokens": 1024, "messages": [{ "role": "user", "content": prompt }] });
                let response = client
                    .post(&url)
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .json(&body)
                    .timeout(Duration::from_secs(30))
                    .send()
                    .await
                    .map_err(|e| AiError::Http(e.to_string()))?;
                let response_json: Value = response.json().await.map_err(|e| AiError::Http(e.to_string()))?;
                Ok(response_json["content"][0]["text"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string())
            }
            Provider::Gemini { api_key, base_url } => {
                let client = Client::new();
                let url = format!("{}/models/{}:generateContent?key={}", base_url, model, api_key);
                let body = serde_json::json!({ "contents": [{ "parts": [{ "text": prompt }] }] });
                let response = client
                    .post(&url)
                    .json(&body)
                    .timeout(Duration::from_secs(30))
                    .send()
                    .await
                    .map_err(|e| AiError::Http(e.to_string()))?;
                let response_json: Value = response.json().await.map_err(|e| AiError::Http(e.to_string()))?;
                Ok(response_json["candidates"][0]["content"]["parts"][0]["text"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string())
            }
            Provider::Ollama { base_url } => {
                let client = Client::new();
                let url = format!("{}/api/generate", base_url);
                let body = serde_json::json!({ "model": model, "prompt": prompt, "stream": false });
                let response = client
                    .post(&url)
                    .json(&body)
                    .timeout(Duration::from_secs(30))
                    .send()
                    .await
                    .map_err(|e| AiError::Http(e.to_string()))?;
                let response_json: Value = response.json().await.map_err(|e| AiError::Http(e.to_string()))?;
                Ok(response_json["response"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string())
            }
            Provider::Mock => Ok(format!("[mock] {}: {}", model, &prompt[..prompt.len().min(200)])),
            _ => Err(AiError::Unsupported("provider does not support text completion")),
        }
    }
}
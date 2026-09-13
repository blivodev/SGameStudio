use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiTask {
    ChatAssistant,
    Vector2D,
    Model3D,
    TexturePbr,
    AutoRig2D,
    AutoRig3D,
    AudioSfx,
    VocalTts,
    Music,
}

impl AiTask {
    pub fn config_key(&self) -> &'static str {
        match self {
            AiTask::ChatAssistant => "chat_assistant",
            AiTask::Vector2D => "vector_2d_generation",
            AiTask::Model3D => "model_3d_generation",
            AiTask::TexturePbr => "texture_pbr_generation",
            AiTask::AutoRig2D => "auto_rigging_2d_heuristic",
            AiTask::AutoRig3D => "auto_rigging_3d_heuristic",
            AiTask::AudioSfx => "audio_sfx_prompt",
            AiTask::VocalTts => "vocal_speech_tts",
            AiTask::Music => "music_composition",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskRoute {
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RoutingConfig {
    pub routes: BTreeMap<String, TaskRoute>,
}

impl RoutingConfig {
    pub fn default_config() -> Self {
        let mut routes = BTreeMap::new();
        routes.insert(
            "chat_assistant".to_string(),
            TaskRoute {
                provider: "deepinfra".to_string(),
                model: Some("meta-llama/Meta-Llama-3.1-70B-Instruct".to_string()),
                fallback_provider: Some("openrouter".to_string()),
                fallback_model: Some("deepseek/deepseek-chat".to_string()),
                ..Default::default()
            },
        );
        routes.insert(
            "vector_2d_generation".to_string(),
            TaskRoute {
                provider: "openrouter".to_string(),
                model: Some("anthropic/claude-3.5-sonnet".to_string()),
                fallback_provider: Some("gemini".to_string()),
                fallback_model: Some("gemini-1.5-flash".to_string()),
                ..Default::default()
            },
        );
        routes.insert(
            "model_3d_generation".to_string(),
            TaskRoute {
                provider: "comfyui".to_string(),
                endpoint: Some("http://127.0.0.1:8188".to_string()),
                workflow: Some("trellis_text_to_3d".to_string()),
                fallback_provider: Some("deepinfra".to_string()),
                fallback_model: Some("stabilityai/triposr".to_string()),
                ..Default::default()
            },
        );
        routes.insert(
            "texture_pbr_generation".to_string(),
            TaskRoute {
                provider: "deepinfra".to_string(),
                model: Some("stabilityai/stable-diffusion-xl-base-1.0".to_string()),
                ..Default::default()
            },
        );
        routes.insert(
            "auto_rigging_2d_heuristic".to_string(),
            TaskRoute {
                provider: "gemini".to_string(),
                model: Some("gemini-1.5-flash".to_string()),
                ..Default::default()
            },
        );
        routes.insert(
            "auto_rigging_3d_heuristic".to_string(),
            TaskRoute {
                provider: "openrouter".to_string(),
                model: Some("openai/gpt-4o-mini".to_string()),
                ..Default::default()
            },
        );
        routes.insert(
            "audio_sfx_prompt".to_string(),
            TaskRoute {
                provider: "openrouter".to_string(),
                model: Some("deepseek/deepseek-chat".to_string()),
                ..Default::default()
            },
        );
        routes.insert(
            "vocal_speech_tts".to_string(),
            TaskRoute {
                provider: "local_piper".to_string(),
                model: Some("en_US-lessac-medium".to_string()),
                ..Default::default()
            },
        );
        routes.insert(
            "music_composition".to_string(),
            TaskRoute {
                provider: "local_audiocraft".to_string(),
                model: Some("musicgen-small".to_string()),
                ..Default::default()
            },
        );
        RoutingConfig { routes }
    }

    pub fn route_for(&self, task: AiTask) -> Option<&TaskRoute> {
        self.routes.get(task.config_key())
    }

    pub fn resolve_with_fallback(&self, task: AiTask) -> Option<(String, Option<String>)> {
        let route = self.route_for(task)?;
        if route.provider.is_empty() {
            return None;
        }
        let mut provider = route.provider.clone();
        let mut model = route.model.clone();
        if model.is_none() {
            if let Some(fallback_provider) = &route.fallback_provider {
                provider = fallback_provider.clone();
                model = route.fallback_model.clone();
            }
        }
        Some((provider, model))
    }
}
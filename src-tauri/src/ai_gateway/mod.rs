pub mod providers;
pub mod router;
pub use providers::{AiError, Provider};
pub use router::{AiTask, RoutingConfig, TaskRoute};

/// Resolves the configured provider for a task and dispatches a completion.
pub async fn run_task(
    config: &RoutingConfig,
    providers: &std::collections::BTreeMap<String, Provider>,
    task: AiTask,
    prompt: &str,
) -> Result<String, AiError> {
    let Some((provider_name, model)) = config.resolve_with_fallback(task) else {
        return Err(AiError::NotConfigured(task.config_key()));
    };
    let Some(provider) = providers.get(&provider_name) else {
        return Err(AiError::NotConfigured(task.config_key()));
    };
    let model = model.as_deref().unwrap_or("default");
    provider.complete(model, prompt).await
}

/// Build the set of providers available right now (from the stored API keys /
/// env vars). Providers without a configured key still resolve to a Provider
/// struct with an empty key; call sites that need a real key should use the
/// `settings` module or rely on the provider returning a HTTP 401/403 upstream.
pub fn configured_providers() -> std::collections::BTreeMap<String, Provider> {
    let mut map = std::collections::BTreeMap::new();
    for name in crate::settings::PROVIDERS {
        if let Some(p) = Provider::from_name(name) {
            map.insert((*name).to_string(), p);
        }
    }
    map
}

/// Whether a provider is usable right now. Keyed providers (OpenRouter,
/// DeepInfra, OpenAI, Anthropic, Gemini) are usable only when an API key is
/// configured via env var / providers.json. Local providers never need a key.
pub fn provider_usable(name: &str) -> bool {
    let keyed = ["openrouter", "deepinfra", "openai", "anthropic", "gemini"];
    if keyed.contains(&name) {
        crate::settings::resolve_api_key(name)
            .map(|k| !k.is_empty())
            .unwrap_or(false)
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_providers_covers_known_providers() {
        let providers = configured_providers();
        for name in crate::settings::PROVIDERS {
            assert!(providers.contains_key(*name), "missing provider {name}");
        }
    }

    #[tokio::test]
    async fn run_task_dispatches_through_mock_provider() {
        // Route chat to the mock provider so the dispatch path is exercised
        // without any network or real API key.
        let mut config = RoutingConfig::default_config();
        config.routes.insert(
            "chat_assistant".to_string(),
            TaskRoute { provider: "mock".to_string(), model: Some("mock-model".to_string()), ..Default::default() },
        );
        let mut providers = std::collections::BTreeMap::new();
        providers.insert("mock".to_string(), Provider::Mock);

        let out = run_task(&config, &providers, AiTask::ChatAssistant, "hello world")
            .await
            .expect("mock dispatch should succeed");
        assert!(out.contains("hello world"));
    }

    #[tokio::test]
    async fn unconfigured_task_returns_not_configured() {
        // An empty provider set / unresolvable route must surface NotConfigured.
        let mut empty = RoutingConfig::default();
        empty.routes.insert("chat_assistant".to_string(), TaskRoute { provider: "".to_string(), ..Default::default() });
        let providers = std::collections::BTreeMap::new();
        let out = run_task(&empty, &providers, AiTask::ChatAssistant, "x").await;
        assert!(matches!(out, Err(AiError::NotConfigured(_))));
    }

    #[test]
    fn local_providers_are_always_usable() {
        assert!(provider_usable("local_piper"));
        assert!(provider_usable("local_audiocraft"));
        assert!(provider_usable("mock"));
        assert!(provider_usable("ollama_local"));
        assert!(provider_usable("comfyui_local"));
    }
}
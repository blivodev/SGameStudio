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
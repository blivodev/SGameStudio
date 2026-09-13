//! Settings + AI provider API-key storage for SGameStudio.
//
//! How does it work? (ELI5)
//!   * Some things you type into the app (like your AI provider keys) should be
//!     remembered between runs. We keep them in a small JSON file on disk and
//!     also let you set them with environment variables.
//!   * Environment variables are the safest (they are never written to disk),
//!     e.g. SGAME_OPENROUTER_KEY=sk-... . A settings file `providers.json`
//!     lets you add keys from inside the app UI.
//!   * When we read settings back to show in the UI, we "mask" real keys so
//!     secrets are never sent to the browser or logged.

use std::collections::BTreeMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Known providers and the environment variable that holds their API key.
pub const PROVIDERS: &[&str] = &[
    "openrouter",
    "deepinfra",
    "openai",
    "anthropic",
    "gemini",
];

/// Where the user's extra keys / routing live on disk.
/// We respect an XDG-like path and fall back next to the repo config.
pub fn settings_file_path() -> PathBuf {
    if let Ok(p) = std::env::var("SGAME_SETTINGS_FILE") {
        return PathBuf::from(p);
    }
    let base = std::env::var("APPDATA")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join("sgamestudio").join("providers.json")
}

/// A saved provider entry: which provider, an optional base URL, and the key.
/// The key is stored with a tiny bitshift-ish obfuscation so it is not plainly
/// readable at rest; it is never returned unmasked through the API.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SavedProvider {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    /// The API key. Kept optional so we can overwrite it without clearing others.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

/// The persisted providers registry (a JSON map of provider name -> entry).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderRegistry {
    pub providers: BTreeMap<String, SavedProvider>,
}

impl ProviderRegistry {
    /// Load the registry from disk, tolerating a missing/corrupt file.
    pub fn load() -> Self {
        let path = settings_file_path();
        match std::fs::read_to_string(&path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save the registry to disk (creating parent dirs as needed).
    pub fn save(&self) -> std::io::Result<()> {
        let path = settings_file_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(self)?;
        std::fs::write(&path, bytes)
    }
}

/// Look up a provider's real key. Env vars win; the on-disk registry is the fallback.
/// Recognised env names: SGAME_<PROVIDER_UPPER>_KEY, e.g. SGAME_OPENROUTER_KEY.
pub fn resolve_api_key(provider: &str) -> Option<String> {
    let env_name = format!("SGAME_{}_KEY", provider.to_uppercase().replace('-', "_"));
    if let Ok(k) = std::env::var(&env_name) {
        if !k.trim().is_empty() {
            return Some(k.trim().to_string());
        }
    }
    let reg = ProviderRegistry::load();
    reg.providers
        .get(provider)
        .and_then(|p| p.api_key.clone())
        .filter(|k| !k.is_empty())
}

/// Basic lossy encoding so at-rest keys are not plaintext (defense in depth).
fn obfuscate(plain: &str) -> String {
    plain
        .bytes()
        .map(|b| (b ^ 0b1010_0110))
        .map(char::from)
        .collect()
}

fn deobfuscate(secret: &str) -> String {
    secret
        .chars()
        .map(|c| ((c as u32) as u8 ^ 0b1010_0110) as char)
        .collect()
}

/// Build the `/api/v1/settings` response body: routing (from the default config),
/// which providers have a key configured (masked), and export defaults.
/// Keys are masked (only first 4 + last 4 chars shown) so no secret leaves the server.
pub fn settings_response() -> serde_json::Value {
    let routing = crate::ai_gateway::RoutingConfig::default_config();
    let mut routes_json = serde_json::Map::new();
    for (task, route) in &routing.routes {
        routes_json.insert(
            task.clone(),
            json!({
                "provider": route.provider,
                "model": route.model,
                "endpoint": route.endpoint,
            }),
        );
    }

    let registry = ProviderRegistry::load();
    let mut providers_json = serde_json::Map::new();
    for p in PROVIDERS {
        let key = resolve_api_key(p);
        providers_json.insert(
            (*p).to_string(),
            json!({
                "configured": key.is_some(),
                "keyMasked": key.as_deref().map(mask_key),
            }),
        );
    }

    json!({
        "aiRouting": routes_json,
        "providers": providers_json,
        "exportDefaults": {
            "defaultEngine": "godot4",
            "audioFormat": "ogg",
            "meshFormat": "glb",
        },
    })
}

/// Keep only a few characters of a key so it is safe to display.
pub fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        return "••••".to_string();
    }
    let head = &key[..4];
    let tail = &key[key.len() - 4..];
    format!("{head}••••{tail}")
}

/// Apply a settings update: set a provider key (and optional base URL), persist,
/// and return the masked confirmation. `clear=true` removes the saved key.
pub fn apply_provider_settings(provider: &str, api_key: Option<&str>, base_url: Option<&str>, clear: bool) -> std::io::Result<serde_json::Value> {
    let mut reg = ProviderRegistry::load();
    let entry = reg.providers.entry(provider.to_string()).or_default();
    if let Some(b) = base_url {
        if !b.is_empty() {
            entry.base_url = Some(b.to_string());
        }
    }
    if clear {
        entry.api_key = None;
    } else if let Some(k) = api_key {
        if !k.is_empty() {
            entry.api_key = Some(obfuscate(k));
        }
    }
    reg.save()?;
    // We never hand the real key (or its obfuscation) back to the caller.
    Ok(json!({ "provider": provider, "configured": !clear, "keyMasked": None::<String> }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_keeps_edges() {
        assert_eq!(mask_key("sk-abcdefghijklmnop"), "sk-a••••mnop");
        assert!(mask_key("short").contains('•'));
    }

    #[test]
    fn obfuscation_roundtrips() {
        let k = "sk-test-123";
        assert_eq!(deobfuscate(&obfuscate(k)), k);
    }

    #[test]
    fn settings_response_is_shaped() {
        let v = settings_response();
        assert!(v["aiRouting"]["chat_assistant"]["provider"].is_string());
        assert!(v["providers"]["openrouter"].is_object());
        assert!(v["exportDefaults"]["meshFormat"].is_string());
    }
}

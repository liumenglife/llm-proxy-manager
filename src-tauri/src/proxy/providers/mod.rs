pub mod openai;
pub mod zai_anthropic;

use crate::models::TokenData;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub model: String,
    pub body: serde_json::Value,
    pub account_id: String,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub status: u16,
    pub body: serde_json::Value,
}

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn name(&self) -> &'static str;
    fn base_url(&self) -> &str;
    async fn send_request(&self, req: ProviderRequest) -> Result<ProviderResponse, String>;
    async fn refresh_token(&self, token: &TokenData) -> Result<TokenData, String>;
    fn default_models(&self) -> Vec<&'static str>;
}

static PROVIDER_REGISTRY: OnceLock<HashMap<String, Box<dyn AiProvider>>> = OnceLock::new();

pub fn init_providers() {
    let mut map: HashMap<String, Box<dyn AiProvider>> = HashMap::new();
    map.insert("codex".to_string(), Box::new(openai::OpenAIProvider));
    let _ = PROVIDER_REGISTRY.set(map);
}

pub fn get_provider(name: &str) -> Option<&'static dyn AiProvider> {
    PROVIDER_REGISTRY.get()?.get(name).map(|b| b.as_ref())
}

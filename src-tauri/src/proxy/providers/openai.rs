use async_trait::async_trait;
use super::{AiProvider, ProviderRequest, ProviderResponse};
use crate::models::TokenData;

pub struct OpenAIProvider;

#[async_trait]
impl AiProvider for OpenAIProvider {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn base_url(&self) -> &str {
        "https://api.openai.com/v1"
    }

    async fn send_request(&self, _req: ProviderRequest) -> Result<ProviderResponse, String> {
        todo!("完成 OpenAI API 请求发送")
    }

    async fn refresh_token(&self, token: &TokenData) -> Result<TokenData, String> {
        crate::modules::oauth_codex::ensure_fresh_codex_token(token).await
    }

    fn default_models(&self) -> Vec<&'static str> {
        vec![
            "PasGPT-5-Codex",
            "GPT-5.1",
            "Codex GPT-5.1",
            "Codex Max GPT-5.1",
            "Codex mini",
            "GPT-5.2",
            "GPT-5.2 Codex",
            "GPT-5.3 Codex",
            "GPT-5.4",
            "GPT-5.4 Fast",
            "GPT-5.4 mini",
            "GPT-5.4 mini Fast",
            "GPT-5.5 Fast",
            "GPT-5.5 Pro",
        ]
    }
}

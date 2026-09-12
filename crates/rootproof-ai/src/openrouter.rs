use crate::{AiConfig, AiError};
use rig::{client::AgentClientExt, completion::Prompt, providers::openrouter};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned,Serialize};

pub struct OpenRouterProvider {
    config: AiConfig,
    api_key: String,
}

impl OpenRouterProvider {
    pub fn new(config: AiConfig) -> Result<Self, AiError> {
        let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| AiError::MissingApiKey)?;

        if api_key.trim().is_empty() {
            return Err(AiError::MissingApiKey);
        }

        if config.model.trim().is_empty() {
            return Err(AiError::MissingModel);
        }

        Ok(Self { config, api_key })
    }

    pub fn model(&self) -> &str {
        &self.config.model
    }

    pub async fn prompt(&self, prompt: &str) -> Result<String, AiError> {
        let client = openrouter::Client::new(self.api_key.clone())
            .map_err(|error| AiError::Provider(error.to_string()))?;

        let agent = client
            .agent(&self.config.model)
            .preamble(
                "You are an AI assistant used by RootProof, \
                        a Rust debugging tool.",
            )
            .build();

        agent
            .prompt(prompt)
            .await
            .map_err(|error| AiError::Provider(error.to_string()))
    }

    pub async fn extract<T>(&self, input: &str, preamble: &str) -> Result<T, AiError>
        where
            T: JsonSchema
                + DeserializeOwned
                + Serialize
                + Send
                + Sync
                + 'static{
        let client =
            openrouter::Client::new(
                self.api_key.clone(),
            )
            .map_err(|error| {
                AiError::Provider(
                    error.to_string(),
                )
            })?;
    
        let extractor =
            client
                .extractor::<T>(
                    &self.config.model,
                )
                .preamble(preamble)
                .retries(2)
                .build();
    
        extractor
            .extract(input)
            .await
            .map_err(|error| {
                AiError::Provider(
                    error.to_string(),
                )
            })
    }
}


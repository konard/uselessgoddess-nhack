//! Ollama HTTP client for local AI inference.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Default Ollama API endpoint.
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

/// Errors that can occur when communicating with Ollama.
#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Failed to parse AI response: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Ollama returned an error: {0}")]
    OllamaError(String),

    #[error("Invalid response format from AI")]
    InvalidResponseFormat,
}

/// Request body for Ollama's generate endpoint.
#[derive(Debug, Serialize)]
pub struct OllamaRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

/// Generation options for Ollama.
#[derive(Debug, Serialize, Default)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<i32>,
}

/// Response from Ollama's generate endpoint.
#[derive(Debug, Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
    #[serde(default)]
    pub total_duration: Option<u64>,
    #[serde(default)]
    pub load_duration: Option<u64>,
    #[serde(default)]
    pub prompt_eval_count: Option<u32>,
    #[serde(default)]
    pub eval_count: Option<u32>,
}

/// Client for communicating with Ollama.
#[derive(Debug, Clone)]
pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    /// Create a new Ollama client with default settings.
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: DEFAULT_OLLAMA_URL.to_string(),
            model: model.into(),
        }
    }

    /// Create a client with a custom base URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Send a prompt to Ollama and get a response.
    pub async fn generate(&self, prompt: &str) -> Result<String, OllamaError> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            format: None,
            options: Some(OllamaOptions {
                temperature: Some(0.8),
                top_p: Some(0.9),
                num_predict: Some(500),
            }),
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OllamaError::OllamaError(error_text));
        }

        let ollama_response: OllamaResponse = response.json().await?;
        Ok(ollama_response.response)
    }

    /// Send a prompt expecting JSON response.
    pub async fn generate_json<T: for<'de> Deserialize<'de>>(
        &self,
        prompt: &str,
    ) -> Result<T, OllamaError> {
        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
            format: Some("json".to_string()),
            options: Some(OllamaOptions {
                temperature: Some(0.7),
                top_p: Some(0.9),
                num_predict: Some(800),
            }),
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OllamaError::OllamaError(error_text));
        }

        let ollama_response: OllamaResponse = response.json().await?;

        // Parse the JSON from the response
        let parsed: T = serde_json::from_str(&ollama_response.response)?;
        Ok(parsed)
    }

    /// Check if Ollama is running and accessible.
    pub async fn health_check(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Get the current model name.
    pub fn model(&self) -> &str {
        &self.model
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        // Default to a capable model for narrative generation
        Self::new("llama3.2")
    }
}

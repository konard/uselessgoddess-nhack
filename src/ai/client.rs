//! Ollama HTTP client for local AI inference.
//!
//! This module provides a client for communicating with a local Ollama instance,
//! supporting both the `/api/generate` endpoint for basic generation and
//! `/api/chat` endpoint for structured outputs with JSON schema validation.

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

/// Request body for Ollama's chat endpoint with structured outputs.
#[derive(Debug, Serialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    /// JSON schema for structured output validation.
    /// See: https://ollama.com/blog/structured-outputs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

/// A message in the chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
}

/// Generation options for Ollama.
#[derive(Debug, Serialize, Default, Clone)]
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

/// Response from Ollama's chat endpoint.
#[derive(Debug, Deserialize)]
pub struct OllamaChatResponse {
    pub model: String,
    pub message: ChatMessage,
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

    /// Send a prompt expecting JSON response (legacy mode with format: "json").
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

    /// Generate structured output with JSON schema validation.
    ///
    /// This uses Ollama's structured outputs feature via the `/api/chat` endpoint.
    /// The model's response will be constrained to match the provided JSON schema,
    /// ensuring more reliable and consistent output than basic JSON mode.
    ///
    /// See: <https://ollama.com/blog/structured-outputs>
    ///
    /// # Arguments
    ///
    /// * `prompt` - The user prompt describing what to generate
    /// * `schema` - JSON schema that defines the expected output structure
    ///
    /// # Example
    ///
    /// ```ignore
    /// let schema = serde_json::json!({
    ///     "type": "object",
    ///     "properties": {
    ///         "name": {"type": "string"},
    ///         "description": {"type": "string"}
    ///     },
    ///     "required": ["name", "description"]
    /// });
    ///
    /// let response: MyStruct = client.generate_structured(prompt, schema).await?;
    /// ```
    pub async fn generate_structured<T: for<'de> Deserialize<'de>>(
        &self,
        prompt: &str,
        schema: serde_json::Value,
    ) -> Result<T, OllamaError> {
        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage::user(prompt)],
            stream: false,
            format: Some(schema),
            options: Some(OllamaOptions {
                // Use temperature 0 for deterministic structured outputs
                temperature: Some(0.0),
                top_p: Some(0.9),
                num_predict: Some(800),
            }),
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OllamaError::OllamaError(error_text));
        }

        let chat_response: OllamaChatResponse = response.json().await?;

        // Parse the JSON from the assistant's message content
        let parsed: T = serde_json::from_str(&chat_response.message.content)?;
        Ok(parsed)
    }

    /// Generate structured output with system message and JSON schema.
    ///
    /// This variant allows setting a system message to provide context or
    /// personality instructions to the model before the user prompt.
    pub async fn generate_structured_with_system<T: for<'de> Deserialize<'de>>(
        &self,
        system_msg: &str,
        prompt: &str,
        schema: serde_json::Value,
    ) -> Result<T, OllamaError> {
        let request = OllamaChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage::system(system_msg),
                ChatMessage::user(prompt),
            ],
            stream: false,
            format: Some(schema),
            options: Some(OllamaOptions {
                temperature: Some(0.0),
                top_p: Some(0.9),
                num_predict: Some(800),
            }),
        };

        let response = self
            .client
            .post(format!("{}/api/chat", self.base_url))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(OllamaError::OllamaError(error_text));
        }

        let chat_response: OllamaChatResponse = response.json().await?;
        let parsed: T = serde_json::from_str(&chat_response.message.content)?;
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

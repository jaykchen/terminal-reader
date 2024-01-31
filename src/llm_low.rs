use reqwest::{ header::{ HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE }, Client };
use serde::{ Deserialize, Serialize };
use std::env;

pub async fn chat_inner_async(
    system_prompt: &str,
    user_input: &str,
    max_token: u16,
    model: &str
) -> anyhow::Result<String> {
    // Changed Error type to anyhow::Error for more general error handling.
    let api_key = env::var("LLM_API_KEY").expect("LLM_API_KEY-must-be-set");

    let client = Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap());

    let messages =
        serde_json::json!([
        {
            "role": "system",
            "content": system_prompt
        },
        {
            "role": "user",
            "content": user_input
        }
    ]);

    let body =
        serde_json::json!({
        "model": model.to_string(),
        "messages": messages,
        "max_tokens": max_token,
        "temperature": 0.5
    });

    use anyhow::Context;

    let response = client
        .post("https://api.deepinfra.com/v1/openai/chat/completions")
        .headers(headers)
        .json(&body)
        .send().await
        .context("Failed to send request to API")?; // Adds context to the error

    let status_code = response.status();

    if status_code.is_success() {
        let body_text = response.text().await.context("Failed to read response body")?;

        let chat_response: CreateCompletionResponse = serde_json
            ::from_str(&body_text)
            .context("Failed to parse successful response from API")?;
        if let Some(choice) = chat_response.choices.get(0) {
            Ok(choice.message.content.to_string())
        } else {
            Err(anyhow::anyhow!("No response choices found"))
        }
    } else {
        let error_body = response
            .text().await
            .unwrap_or_else(|_| "Failed to read error response body".to_string());
        Err(anyhow::anyhow!("API request failed with status code {}: {}", status_code, error_body))
    }
}

use serde_json::Value; // Make sure serde_json is in your Cargo.toml

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64, // Assuming a Unix timestamp can fit into u64
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: CompletionUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletionUsage {
    pub prompt_tokens: u32,
    pub total_tokens: u32,
    pub completion_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: MessageContent,
    pub finish_reason: String, // Assuming it's always present and a String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageContent {
    pub role: String,
    pub content: String,
    pub name: Option<String>, // Nullable in JSON, so we use Option
    pub tool_calls: Option<Value>, // Assuming dynamic content; replace Value with a specific struct if known
}

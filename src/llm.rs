// use http_req::{ request::Method, request::Request, response, uri::Uri };
// use log;
use reqwest::header::{ HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT };
// use reqwest::{ header, Client };
use secrecy::{
    // ExposeSecret,
    Secret,
};
use std::collections::HashMap;

use async_openai::{
    config::Config,
    types::{
        // ChatCompletionFunctionsArgs, ChatCompletionRequestMessage,
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        // ChatCompletionTool, ChatCompletionToolArgs, ChatCompletionToolType,
        CreateChatCompletionRequestArgs,
    },
    Client as OpenAIClient,
};
use std::env;

pub async fn chain_of_chat(
    sys_prompt_1: &str,
    usr_prompt_1: &str,
    chat_id: &str,
    gen_len_1: u16,
    usr_prompt_2: &str,
    gen_len_2: u16,
    error_tag: &str
) -> anyhow::Result<String> {
    let token = env::var("LLM_API_KEY").unwrap_or(String::from("LLM_API_KEY-must-be-set"));

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("MyClient/1.0.0"));
    let config = LocalServiceProviderConfig {
        api_base: String::from("https://api.deepinfra.com/v1/openai"),
        headers: headers,
        api_key: Secret::new(token),
        query: HashMap::new(),
    };

    let model = "Phind/Phind-CodeLlama-34B-v2";
    let client = OpenAIClient::with_config(config);

    let mut messages = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content(sys_prompt_1)
            .build()
            .expect("Failed to build system message")
            .into(),
        ChatCompletionRequestUserMessageArgs::default().content(usr_prompt_1).build()?.into()
    ];
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(gen_len_1)
        .model(model)
        .messages(messages.clone())
        .build()?;

    // dbg!("{:?}", request.clone());

    let chat = client.chat().create(request).await?;

    match chat.choices[0].message.clone().content {
        Some(res) => {
            println!("step 1 Points: {:?}", res);
        }
        None => {
            return Err(anyhow::anyhow!(error_tag.to_string()));
        }
    }

    messages.push(
        ChatCompletionRequestUserMessageArgs::default().content(usr_prompt_2).build()?.into()
    );

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(gen_len_2)
        .model(model)
        .messages(messages)
        .build()?;

    let chat = client.chat().create(request).await?;

    match chat.choices[0].message.clone().content {
        Some(res) => {
            println!("step 2 Raw: {:?}", res);
            Ok(res)
        }
        None => {
            return Err(anyhow::anyhow!(error_tag.to_string()));
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocalServiceProviderConfig {
    pub api_base: String,
    pub headers: HeaderMap,
    pub api_key: Secret<String>,
    pub query: HashMap<String, String>,
}

impl Config for LocalServiceProviderConfig {
    fn headers(&self) -> HeaderMap {
        self.headers.clone()
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.api_base, path)
    }

    fn query(&self) -> Vec<(&str, &str)> {
        self.query
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    fn api_base(&self) -> &str {
        &self.api_base
    }

    fn api_key(&self) -> &Secret<String> {
        &self.api_key
    }
}

pub async fn chat_inner_async(
    system_prompt: &str,
    user_input: &str,
    max_token: u16,
    model: &str
) -> anyhow::Result<String> {
    let token = env::var("LLM_API_KEY").unwrap_or(String::from("LLM_API_KEY-must-be-set"));

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(USER_AGENT, HeaderValue::from_static("MyClient/1.0.0"));
    let config = LocalServiceProviderConfig {
        api_base: String::from("https://api.deepinfra.com/v1/openai"),
        headers: headers,
        api_key: Secret::new(token),
        query: HashMap::new(),
    };

    let model = "Phind/Phind-CodeLlama-34B-v2";
    let client = OpenAIClient::with_config(config);

    let messages = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt)
            .build()
            .expect("Failed to build system message")
            .into(),
        ChatCompletionRequestUserMessageArgs::default().content(user_input).build()?.into()
    ];
    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(max_token)
        .model(model)
        .messages(messages)
        .build()?;

    match client.chat().create(request).await {
        Ok(chat) =>
            match chat.choices[0].message.clone().content {
                Some(res) => {
                    // println!("{:?}", chat.choices[0].message.clone());
                    Ok(res)
                }
                None => Err(anyhow::anyhow!("Failed to get reply from OpenAI")),
            }
        Err(_e) => {
            println!("Error getting response from hosted LLM: {:?}", _e);
            Err(anyhow::anyhow!(_e))
        }
    }
}

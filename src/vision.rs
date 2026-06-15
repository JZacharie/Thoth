use anyhow::Result;
use base64::Engine;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::VisionConfig;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: Vec<ContentPart>,
}

#[derive(Serialize)]
struct ContentPart {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_url: Option<ImageUrl>,
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

pub struct VisionAnalyzer {
    client: Client,
    endpoint: String,
    secret: String,
    config: VisionConfig,
}

impl VisionAnalyzer {
    pub fn new(client: Client, endpoint: String, secret: String, config: VisionConfig) -> Self {
        Self {
            client,
            endpoint,
            secret,
            config,
        }
    }

    pub async fn analyze_screenshot(&self, png_data: &[u8]) -> Result<String> {
        let encoded = base64::engine::general_purpose::STANDARD.encode(png_data);
        let data_url = format!("data:image/png;base64,{}", encoded);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![Message {
                role: "user".into(),
                content: vec![
                    ContentPart {
                        content_type: "text".into(),
                        text: Some(self.config.system_prompt.clone()),
                        image_url: None,
                    },
                    ContentPart {
                        content_type: "image_url".into(),
                        text: None,
                        image_url: Some(ImageUrl { url: data_url }),
                    },
                ],
            }],
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .header("X-Thoth-Secret", &self.secret)
            .header("Authorization", format!("Bearer {}", self.secret))
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let body: ChatResponse = response.json().await?;
        let content = body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        tracing::info!("vision analysis result (len: {})", content.len());

        Ok(content)
    }
}

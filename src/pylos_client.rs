use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::PylosConfig;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
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

fn language_name(code: &str) -> &str {
    match code {
        "fr" => "français",
        "en" => "anglais",
        "es" => "espagnol",
        "de" => "allemand",
        "it" => "italien",
        "pt" => "portugais",
        "nl" => "néerlandais",
        "ja" => "japonais",
        "zh" => "chinois",
        "ru" => "russe",
        _ => "français",
    }
}

fn contains_sensitive_data(text: &str) -> bool {
    let patterns = [
        regex::Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(),
        regex::Regex::new(r"pk-[a-zA-Z0-9]{20,}").unwrap(),
        regex::Regex::new(r"eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap(),
        regex::Regex::new(r"-----BEGIN.*PRIVATE KEY-----").unwrap(),
        regex::Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").unwrap(),
    ];
    patterns.iter().any(|re| re.is_match(text))
}

pub fn is_sensitive(text: &str) -> bool {
    contains_sensitive_data(text)
}

pub struct PylosClient {
    client: Client,
    config: PylosConfig,
    target_language: String,
}

impl PylosClient {
    pub fn new(mut config: PylosConfig, target_language: String) -> Self {
        while config.endpoint.ends_with('/') {
            config.endpoint.pop();
        }
        if config.endpoint.ends_with("/v1") {
            config.endpoint.truncate(config.endpoint.len() - 3);
        }
        while config.endpoint.ends_with('/') {
            config.endpoint.pop();
        }

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("reqwest client");
        Self {
            client,
            config,
            target_language,
        }
    }

    fn build_prompt(&self) -> String {
        let lang = language_name(&self.target_language);
        format!(
            "Tu es un traducteur et correcteur de texte ultra-précis.\n\
             Ta tâche est de traduire, corriger l'orthographe/grammaire et rendre le texte fourni clair et concis.\n\
             Traduis le texte suivant en {}.\n\
             Tu dois UNIQUEMENT retourner le texte corrigé et traduit.\n\
             Ne commence JAMAIS ta réponse par des formules de politesse, des introductions ou des explications.\n\
             Ne mets pas de guillemets ou de blocs de code markdown autour de ta réponse.",
            lang
        )
    }

    async fn translate_with_model(&self, text: &str, model: &str) -> Result<String> {
        let request = ChatRequest {
            model: model.into(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: self.build_prompt(),
                },
                Message {
                    role: "user".into(),
                    content: text.into(),
                },
            ],
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.config.endpoint))
            .header("X-Thoth-Secret", &self.config.secret)
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

        Ok(content)
    }

    pub fn model_name(&self) -> String {
        self.config.model.clone()
    }

    pub async fn translate(&self, text: &str) -> Result<String> {
        let result = self.translate_with_model(text, &self.config.model).await;

        match result {
            Ok(content) => Ok(content),
            Err(e) => {
                tracing::warn!("primary model failed: {e}, trying fallback");
                match &self.config.fallback_model {
                    Some(fallback) => {
                        self.translate_with_model(text, fallback)
                            .await
                            .map_err(|e2| {
                                tracing::error!("fallback model also failed: {e2}");
                                anyhow::anyhow!("both models failed — primary: {e}, fallback: {e2}")
                            })
                    }
                    None => Err(e),
                }
            }
        }
    }
}

/// Returns a map of language codes to language names for tray menu.
#[allow(dead_code)]
pub fn supported_languages() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("fr", "Français");
    m.insert("en", "English");
    m.insert("es", "Español");
    m.insert("de", "Deutsch");
    m.insert("it", "Italiano");
    m.insert("pt", "Português");
    m.insert("nl", "Nederlands");
    m.insert("ja", "日本語");
    m.insert("zh", "中文");
    m.insert("ru", "Русский");
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_sanitization() {
        let cfg = PylosConfig {
            endpoint: "https://pylos.p.zacharie.org/v1/".into(),
            ..Default::default()
        };
        let client = PylosClient::new(cfg, "fr".into());
        assert_eq!(client.config.endpoint, "https://pylos.p.zacharie.org");

        let cfg = PylosConfig {
            endpoint: "https://pylos.p.zacharie.org/v1".into(),
            ..Default::default()
        };
        let client = PylosClient::new(cfg, "fr".into());
        assert_eq!(client.config.endpoint, "https://pylos.p.zacharie.org");

        let cfg = PylosConfig {
            endpoint: "https://pylos.p.zacharie.org/".into(),
            ..Default::default()
        };
        let client = PylosClient::new(cfg, "fr".into());
        assert_eq!(client.config.endpoint, "https://pylos.p.zacharie.org");
    }

    #[test]
    fn test_language_name() {
        assert_eq!(language_name("fr"), "français");
        assert_eq!(language_name("en"), "anglais");
        assert_eq!(language_name("de"), "allemand");
        assert_eq!(language_name("zz"), "français");
    }

    #[test]
    fn test_detect_api_key() {
        assert!(contains_sensitive_data("sk-abcd1234efgh5678ijkl9012mnop"));
        assert!(contains_sensitive_data("pk-abcd1234efgh5678ijkl9012mnop"));
    }

    #[test]
    fn test_detect_jwt() {
        assert!(contains_sensitive_data(
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dB2KdP9fBq3Jm5gW4q6c7Q"
        ));
    }

    #[test]
    fn test_detect_ssh_key() {
        assert!(contains_sensitive_data(
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpQIBAAKCAQEA..."
        ));
    }

    #[test]
    fn test_detect_credit_card() {
        assert!(contains_sensitive_data("4111 1111 1111 1111"));
        assert!(contains_sensitive_data("4111-1111-1111-1111"));
    }

    #[test]
    fn test_clean_text_not_detected() {
        assert!(!contains_sensitive_data("Bonjour, comment allez-vous ?"));
        assert!(!contains_sensitive_data("Le chat est sur le tapis."));
    }

    #[test]
    fn test_empty_text_not_detected() {
        assert!(!contains_sensitive_data(""));
    }

    #[test]
    fn test_is_sensitive_public_fn() {
        assert!(is_sensitive("sk-abcdefghijklmnopqrstuvwxyz1234"));
        assert!(!is_sensitive("Ceci est un texte normal pour traduction"));
    }

    #[test]
    fn test_supported_languages() {
        let langs = supported_languages();
        assert_eq!(langs.len(), 10);
        assert_eq!(*langs.get("fr").unwrap(), "Français");
        assert_eq!(*langs.get("en").unwrap(), "English");
    }
}

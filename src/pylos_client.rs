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
        // AWS Access Key ID
        regex::Regex::new(r"(?i)AKIA[A-Z0-9]{16}").unwrap(),
        // GitHub Tokens
        regex::Regex::new(r"gh[pousr]_[a-zA-Z0-9]{36,255}").unwrap(),
        // Slack Tokens
        regex::Regex::new(r"xox[bp]-[a-zA-Z0-9-]{10,}").unwrap(),
        regex::Regex::new(r"(?i)slack").unwrap(),
        // Database URIs
        regex::Regex::new(r"(?i)mongodb://").unwrap(),
        regex::Regex::new(r"(?i)postgres(ql)?://").unwrap(),
        regex::Regex::new(r"(?i)mysql://").unwrap(),
    ];
    patterns.iter().any(|re| re.is_match(text))
}

pub fn is_sensitive(text: &str) -> bool {
    contains_sensitive_data(text)
}

pub fn anonymize(text: &str) -> (String, std::collections::HashMap<String, String>) {
    let mut placeholders = std::collections::HashMap::new();
    let mut modified_text = text.to_string();

    let patterns = [
        regex::Regex::new(r"sk-[a-zA-Z0-9]{20,}").unwrap(),
        regex::Regex::new(r"pk-[a-zA-Z0-9]{20,}").unwrap(),
        regex::Regex::new(r"eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap(),
        regex::Regex::new(r"-----BEGIN.*PRIVATE KEY-----").unwrap(),
        regex::Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").unwrap(),
        regex::Regex::new(r"(?i)AKIA[A-Z0-9]{16}").unwrap(),
        regex::Regex::new(r"gh[pousr]_[a-zA-Z0-9]{36,255}").unwrap(),
        regex::Regex::new(r"xox[bp]-[a-zA-Z0-9-]{10,}").unwrap(),
        regex::Regex::new(r"(?i)slack").unwrap(),
        regex::Regex::new(r#"(?i)mongodb://[^\s&'"<>]+"#).unwrap(),
        regex::Regex::new(r#"(?i)postgres(ql)?://[^\s&'"<>]+"#).unwrap(),
        regex::Regex::new(r#"(?i)mysql://[^\s&'"<>]+"#).unwrap(),
    ];

    let mut counter = 0;
    for re in &patterns {
        while let Some(m) = re.find(&modified_text) {
            let matched_str = m.as_str().to_string();
            let placeholder = format!("__THOTH_PII_{}__", counter);
            modified_text = modified_text.replace(&matched_str, &placeholder);
            placeholders.insert(placeholder, matched_str);
            counter += 1;
        }
    }

    (modified_text, placeholders)
}

pub fn deanonymize(text: &str, placeholders: &std::collections::HashMap<String, String>) -> String {
    let mut restored = text.to_string();
    for (placeholder, original) in placeholders {
        let index_str = placeholder
            .trim_start_matches("__THOTH_PII_")
            .trim_end_matches("__");
        if let Ok(index) = index_str.parse::<u32>() {
            let pattern = format!(
                r"(?i)(?:__\s*|\[\s*|\{{\s*)?THOTH\s*_?\s*PII\s*_?\s*{}\s*(?:\s*\]|\s*\}}|\s*__)?",
                index
            );
            if let Ok(re) = regex::Regex::new(&pattern) {
                restored = re.replace_all(&restored, original).to_string();
                continue;
            }
        }
        let escaped = regex::escape(placeholder);
        if let Ok(re) = regex::RegexBuilder::new(&escaped)
            .case_insensitive(true)
            .build()
        {
            restored = re.replace_all(&restored, original).to_string();
        }
    }
    restored
}

pub fn clean_response(text: &str) -> String {
    let mut cleaned = text.trim().to_string();

    let prefixes = [
        regex::Regex::new(r"(?i)^voici le texte corrigé et traduit\s*:\s*\n*").unwrap(),
        regex::Regex::new(r"(?i)^voici la traduction\s*:\s*\n*").unwrap(),
        regex::Regex::new(r"(?i)^voici le texte traduit\s*:\s*\n*").unwrap(),
        regex::Regex::new(r"(?i)^here is the translation\s*:\s*\n*").unwrap(),
        regex::Regex::new(r"(?i)^here is the corrected and translated text\s*:\s*\n*").unwrap(),
        regex::Regex::new(r"(?i)^here is the corrected text\s*:\s*\n*").unwrap(),
        regex::Regex::new(r"(?i)^voici le texte corrigé\s*:\s*\n*").unwrap(),
    ];

    for re in &prefixes {
        cleaned = re.replace(&cleaned, "").to_string();
    }

    let mut cleaned = cleaned.trim().to_string();
    if let Some(stripped) = cleaned.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        cleaned = stripped.trim().to_string();
    } else if let Some(stripped) = cleaned.strip_prefix('«').and_then(|s| s.strip_suffix('»')) {
        cleaned = stripped.trim().to_string();
    }

    cleaned
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
        if config.endpoint.ends_with("/v1/chat/completions") {
            config.endpoint.truncate(config.endpoint.len() - 20);
        } else if config.endpoint.ends_with("/chat/completions") {
            config.endpoint.truncate(config.endpoint.len() - 17);
        }
        if config.endpoint.ends_with("/v1") {
            config.endpoint.truncate(config.endpoint.len() - 3);
        }
        while config.endpoint.ends_with('/') {
            config.endpoint.pop();
        }

        let insecure = crate::is_insecure();
        let is_local =
            config.endpoint.contains("localhost") || config.endpoint.contains("127.0.0.1");

        if !insecure && !is_local && !config.endpoint.starts_with("https://") {
            config.endpoint = config.endpoint.replace("http://", "https://");
            if !config.endpoint.starts_with("https://") {
                config.endpoint = format!("https://{}", config.endpoint);
            }
        }

        let mut builder = Client::builder().timeout(Duration::from_secs(config.timeout_secs));

        if insecure {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build().expect("reqwest client");
        Self {
            client,
            config,
            target_language,
        }
    }

    fn build_prompt(&self, target_lang: &str) -> String {
        let lang = language_name(target_lang);
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

    async fn translate_with_model(
        &self,
        text: &str,
        model: &str,
        target_lang: &str,
    ) -> Result<String> {
        let (anon_text, mapping) = anonymize(text);
        let request = ChatRequest {
            model: model.into(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: self.build_prompt(target_lang),
                },
                Message {
                    role: "user".into(),
                    content: anon_text,
                },
            ],
        };

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.config.endpoint))
            .header("X-Thoth-Secret", &self.config.secret)
            .header("Authorization", format!("Bearer {}", self.config.secret))
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

        let cleaned = clean_response(&content);
        let restored = deanonymize(&cleaned, &mapping);
        Ok(restored)
    }

    pub fn model_name(&self) -> String {
        self.config.model.clone()
    }

    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }

    pub async fn test_connection(&self) -> Result<()> {
        let url = format!("{}/v1/models", self.config.endpoint);
        let _ = self
            .client
            .get(&url)
            .header("X-Thoth-Secret", &self.config.secret)
            .header("Authorization", format!("Bearer {}", self.config.secret))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    fn build_instruction_prompt(&self) -> String {
        "Tu es un assistant personnel intelligent et ultra-précis.\n\
         Analyse le texte fourni, identifie la consigne ou l'action demandée (par exemple : résumer, expliquer, répondre à un e-mail, reformuler, etc.) et exécute-la directement sur le reste du texte.\n\
         Génère UNIQUEMENT la réponse ou le résultat final attendu.\n\
         Ne commence JAMAIS ta réponse par des formules de politesse, des introductions ou des explications sur ce que tu fais.\n\
         Ne mets pas de guillemets ou de blocs de code markdown autour de ta réponse, sauf si la consigne demande explicitement un format spécifique.".to_string()
    }

    pub async fn execute_instruction(&self, text: &str) -> Result<String> {
        let (anon_text, mapping) = anonymize(text);
        let system_prompt = self.build_instruction_prompt();
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: system_prompt.clone(),
                },
                Message {
                    role: "user".into(),
                    content: anon_text.clone(),
                },
            ],
        };

        let result = self
            .client
            .post(format!("{}/v1/chat/completions", self.config.endpoint))
            .header("X-Thoth-Secret", &self.config.secret)
            .header("Authorization", format!("Bearer {}", self.config.secret))
            .json(&request)
            .send()
            .await?
            .error_for_status();

        let response = match result {
            Ok(res) => res,
            Err(e) => {
                tracing::warn!("primary model failed on instruction: {e}, trying fallback");
                match &self.config.fallback_model {
                    Some(fallback) => {
                        let request_fallback = ChatRequest {
                            model: fallback.clone(),
                            messages: vec![
                                Message {
                                    role: "system".into(),
                                    content: system_prompt,
                                },
                                Message {
                                    role: "user".into(),
                                    content: anon_text.clone(),
                                },
                            ],
                        };
                        self.client
                            .post(format!("{}/v1/chat/completions", self.config.endpoint))
                            .header("X-Thoth-Secret", &self.config.secret)
                            .header("Authorization", format!("Bearer {}", self.config.secret))
                            .json(&request_fallback)
                            .send()
                            .await?
                            .error_for_status()?
                    }
                    None => return Err(e.into()),
                }
            }
        };

        let body: ChatResponse = response.json().await?;
        let content = body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        let cleaned = clean_response(&content);
        let restored = deanonymize(&cleaned, &mapping);
        Ok(restored)
    }

    pub async fn translate_to(&self, text: &str, target_lang: &str) -> Result<String> {
        let result = self
            .translate_with_model(text, &self.config.model, target_lang)
            .await;

        match result {
            Ok(content) => Ok(content),
            Err(e) => {
                tracing::warn!("primary model failed: {e}, trying fallback");
                match &self.config.fallback_model {
                    Some(fallback) => self
                        .translate_with_model(text, fallback, target_lang)
                        .await
                        .map_err(|e2| {
                            tracing::error!("fallback model also failed: {e2}");
                            anyhow::anyhow!("both models failed — primary: {e}, fallback: {e2}")
                        }),
                    None => Err(e),
                }
            }
        }
    }

    pub async fn execute_with_custom_prompt(
        &self,
        user_prompt: &str,
        text: &str,
    ) -> Result<String> {
        let full_message = format!("{}\n\n{}", user_prompt, text);
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![Message {
                role: "user".into(),
                content: full_message,
            }],
        };

        let result = self
            .client
            .post(format!("{}/v1/chat/completions", self.config.endpoint))
            .header("X-Thoth-Secret", &self.config.secret)
            .header("Authorization", format!("Bearer {}", self.config.secret))
            .json(&request)
            .send()
            .await?
            .error_for_status();

        let response = match result {
            Ok(res) => res,
            Err(e) => {
                tracing::warn!("primary model failed on custom prompt: {e}, trying fallback");
                match &self.config.fallback_model {
                    Some(fallback) => {
                        let request_fallback = ChatRequest {
                            model: fallback.clone(),
                            messages: vec![Message {
                                role: "user".into(),
                                content: format!("{}\n\n{}", user_prompt, text),
                            }],
                        };
                        self.client
                            .post(format!("{}/v1/chat/completions", self.config.endpoint))
                            .header("X-Thoth-Secret", &self.config.secret)
                            .header("Authorization", format!("Bearer {}", self.config.secret))
                            .json(&request_fallback)
                            .send()
                            .await?
                            .error_for_status()?
                    }
                    None => return Err(e.into()),
                }
            }
        };

        let body: ChatResponse = response.json().await?;
        let content = body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(content)
    }

    fn build_reformulate_prompt(&self) -> String {
        "Tu es un rédacteur et communicant d'exception spécialisé dans la clarification textuelle.\n\
         Ton objectif est de reformuler, clarifier les idées et restructurer le texte fourni pour le rendre extrêmement fluide, compréhensible et percutant, tout en préservant fidèlement son sens d'origine.\n\
         Règles strictes :\n\
         - Améliore le style, élimine les redondances et structure les arguments logiquement.\n\
         - Conserve le même niveau de langue (ou rends-le professionnel si familier).\n\
         - Retourne UNIQUEMENT le texte reformulé final.\n\
         - Ne commence JAMAIS par des formules de politesse, des introductions, ou des commentaires sur les changements apportés.\n\
         - Ne mets aucun guillemet ni bloc de code markdown autour de ton texte.".to_string()
    }

    pub async fn reformulate(&self, text: &str) -> Result<String> {
        let system_prompt = self.build_reformulate_prompt();
        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: system_prompt.clone(),
                },
                Message {
                    role: "user".into(),
                    content: text.into(),
                },
            ],
        };

        let result = self
            .client
            .post(format!("{}/v1/chat/completions", self.config.endpoint))
            .header("X-Thoth-Secret", &self.config.secret)
            .header("Authorization", format!("Bearer {}", self.config.secret))
            .json(&request)
            .send()
            .await?
            .error_for_status();

        let response = match result {
            Ok(res) => res,
            Err(e) => {
                tracing::warn!("primary model failed on reformulate: {e}, trying fallback");
                match &self.config.fallback_model {
                    Some(fallback) => {
                        let request_fallback = ChatRequest {
                            model: fallback.clone(),
                            messages: vec![
                                Message {
                                    role: "system".into(),
                                    content: system_prompt,
                                },
                                Message {
                                    role: "user".into(),
                                    content: text.into(),
                                },
                            ],
                        };
                        self.client
                            .post(format!("{}/v1/chat/completions", self.config.endpoint))
                            .header("X-Thoth-Secret", &self.config.secret)
                            .header("Authorization", format!("Bearer {}", self.config.secret))
                            .json(&request_fallback)
                            .send()
                            .await?
                            .error_for_status()?
                    }
                    None => return Err(e.into()),
                }
            }
        };

        let body: ChatResponse = response.json().await?;
        let content = body
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        Ok(content)
    }

    pub async fn translate(&self, text: &str) -> Result<String> {
        self.translate_to(text, &self.target_language).await
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

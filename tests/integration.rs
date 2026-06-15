use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn test_pylos_translate_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{
                "message": {
                    "content": "Bonjour le monde"
                }
            }]
        })))
        .mount(&mock_server)
        .await;

    let config = thoth::PylosConfig {
        endpoint: mock_server.uri(),
        model: "gemma4:12b".into(),
        fallback_model: Some("gemini4:12b".into()),
        timeout_secs: 5,
        secret: "test-secret".into(),
    };

    let client = thoth::PylosClient::new(config, "fr".into());
    let result = client.translate("Hello world").await.unwrap();
    assert_eq!(result, "Bonjour le monde");
}

#[tokio::test]
async fn test_pylos_secret_header() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(wiremock::matchers::header("X-Thoth-Secret", "mon-secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": "ok"}}]
        })))
        .mount(&mock_server)
        .await;

    let config = thoth::PylosConfig {
        endpoint: mock_server.uri(),
        model: "gemma4:12b".into(),
        fallback_model: None,
        timeout_secs: 5,
        secret: "mon-secret".into(),
    };

    let client = thoth::PylosClient::new(config, "fr".into());
    let result = client.translate("Hello").await.unwrap();
    assert_eq!(result, "ok");
}

#[tokio::test]
async fn test_pylos_fallback_on_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(wiremock::matchers::body_partial_json(
            serde_json::json!({"model": "gemma4:12b"}),
        ))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(wiremock::matchers::body_partial_json(
            serde_json::json!({"model": "gemini4:12b"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": "fallback response"}}]
        })))
        .mount(&mock_server)
        .await;

    let config = thoth::PylosConfig {
        endpoint: mock_server.uri(),
        model: "gemma4:12b".into(),
        fallback_model: Some("gemini4:12b".into()),
        timeout_secs: 5,
        secret: "test".into(),
    };

    let client = thoth::PylosClient::new(config, "fr".into());
    let result = client.translate("Hello").await.unwrap();
    assert_eq!(result, "fallback response");
}

#[test]
fn test_sensitive_data_detection() {
    assert!(thoth::is_sensitive("sk-abcdefghijklmnopqrstuvwxyz1234"));
    assert!(!thoth::is_sensitive("Ceci est un texte normal"));
}

#[test]
fn test_validate_language() {
    assert!(thoth::validate_language("fr"));
    assert!(thoth::validate_language("en"));
    assert!(!thoth::validate_language("zz"));
}

#[test]
fn test_hotkey_pattern_parse() {
    let h = thoth::HotkeyPattern::parse("Ctrl+Shift+T").unwrap();
    assert_eq!(
        h.modifiers,
        vec![thoth::Modifier::Ctrl, thoth::Modifier::Shift]
    );
    assert_eq!(h.key, thoth::HotkeyKey::Letter('t'));
}

#[tokio::test]
async fn test_pylos_reformulate_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{
                "message": {
                    "content": "Texte clarifié"
                }
            }]
        })))
        .mount(&mock_server)
        .await;

    let config = thoth::PylosConfig {
        endpoint: mock_server.uri(),
        model: "gemma4:12b".into(),
        fallback_model: None,
        timeout_secs: 5,
        secret: "test-secret".into(),
    };

    let client = thoth::PylosClient::new(config, "fr".into());
    let result = client.reformulate("Quelque texte").await.unwrap();
    assert_eq!(result, "Texte clarifié");
}

// ── Nouveaux tests : Vision, MQTT, S3 ────────────────────────

#[test]
fn test_vision_config_defaults() {
    let cfg = thoth::config::VisionConfig::default();
    assert_eq!(cfg.model, "gemini-3.5-flash");
    assert_eq!(cfg.hotkey, "Ctrl+Shift+Win+P");
    assert!(cfg.system_prompt.contains("Analyse cette image"));
}

#[tokio::test]
async fn test_vision_analysis_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": "B"}}]
        })))
        .mount(&mock_server)
        .await;

    let _config = thoth::PylosConfig {
        endpoint: mock_server.uri(),
        model: "gemma4:12b".into(),
        fallback_model: None,
        timeout_secs: 5,
        secret: "test-vision".into(),
    };

    let vision_config = thoth::config::VisionConfig::default();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let endpoint = mock_server.uri().trim_end_matches('/').to_string();
    let analyzer = thoth::vision::VisionAnalyzer::new(
        http_client,
        endpoint,
        "test-vision".into(),
        vision_config,
    );

    let dummy_png = create_dummy_png();
    let result = analyzer.analyze_screenshot(&dummy_png).await.unwrap();
    assert_eq!(result, "B");
}

#[tokio::test]
async fn test_vision_analysis_sends_multimodal_request() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(wiremock::matchers::header_exists("X-Thoth-Secret"))
        .and(wiremock::matchers::header(
            "Authorization",
            "Bearer test-vision-multi",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": "A"}}]
        })))
        .mount(&mock_server)
        .await;

    let _config = thoth::PylosConfig {
        endpoint: mock_server.uri(),
        model: "gemini-3.5-flash".into(),
        fallback_model: None,
        timeout_secs: 5,
        secret: "test-vision-multi".into(),
    };

    let vision_config = thoth::config::VisionConfig::default();
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();

    let endpoint = mock_server.uri().trim_end_matches('/').to_string();
    let analyzer = thoth::vision::VisionAnalyzer::new(
        http_client,
        endpoint,
        "test-vision-multi".into(),
        vision_config,
    );

    let dummy_png = create_dummy_png();
    let result = analyzer.analyze_screenshot(&dummy_png).await.unwrap();
    assert_eq!(result, "A");
}

#[test]
fn test_mqtt_config_defaults() {
    let cfg = thoth::config::MqttConfig::default();
    assert_eq!(cfg.broker, "mqtt-emqx.p.zacharie.org");
    assert_eq!(cfg.topic, "thoth/answers");
    assert_eq!(cfg.port, 8883);
    assert!(cfg.use_tls);
}

#[test]
fn test_mqtt_payload_serialization() {
    let payload = serde_json::json!({
        "timestamp": "2025-06-15T12:00:00Z",
        "window_title": "Test Window",
        "answer_proposed": "42",
        "latency_ms": 1234,
    });

    let json_str = serde_json::to_string(&payload).unwrap();
    assert!(json_str.contains("Test Window"));
    assert!(json_str.contains("42"));
    assert!(json_str.contains("1234"));

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(parsed["answer_proposed"], "42");
}

#[test]
fn test_s3_config_defaults() {
    let cfg = thoth::config::S3Config::default();
    assert_eq!(cfg.endpoint, "https://minio-170-api.zacharie.org");
    assert_eq!(cfg.bucket, "thoth-screenshots");
    assert_eq!(cfg.region, "auto");
}

#[test]
fn test_s3_storage_empty_secret_returns_none() {
    let cfg = thoth::config::S3Config {
        secret_key: String::new(),
        ..Default::default()
    };
    let result = thoth::s3_storage::S3Storage::new(&cfg);
    assert!(result.unwrap().is_none());
}

#[test]
fn test_chrono_or_fallback_format() {
    let ts = thoth::orchestrator::chrono_or_fallback();
    assert!(!ts.is_empty(), "timestamp should not be empty");
    assert!(ts.contains('d'), "timestamp should contain 'd' for days");
    assert!(ts.contains(':'), "timestamp should contain ':' for time");
}

#[test]
fn test_config_new_sections_roundtrip() {
    let toml_str = r#"
[pylos]
endpoint = "https://test.example.com"
model = "test-model"
timeout_secs = 15
secret = "test-secret"

[behavior]
target_language = "fr"
restore_clipboard = true
show_notifications = true
debounce_ms = 500
hotkey = "Ctrl+Shift+Win+N"

[mqtt]
broker = "test-broker.example.com"
username = "test-user"
password = "test-pass"
topic = "test/topic"
port = 8883
use_tls = true

[s3]
endpoint = "https://test-s3.example.com"
bucket = "test-bucket"
access_key = "test-access"
secret_key = "test-secret"
region = "us-east-1"

[vision]
model = "test-vision-model"
hotkey = "Ctrl+Shift+Win+V"
system_prompt = "Test prompt"
"#;

    let config: thoth::Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.pylos.endpoint, "https://test.example.com");
    assert_eq!(config.mqtt.broker, "test-broker.example.com");
    assert_eq!(config.s3.bucket, "test-bucket");
    assert_eq!(config.vision.model, "test-vision-model");
    assert_eq!(config.behavior.hotkey, "Ctrl+Shift+Win+N");
}

#[test]
fn test_config_new_sections_defaults() {
    let config = thoth::Config::default();
    assert_eq!(config.mqtt.broker, "mqtt-emqx.p.zacharie.org");
    assert_eq!(config.s3.bucket, "thoth-screenshots");
    assert_eq!(config.vision.model, "gemini-3.5-flash");

    let serialized = toml::to_string(&config).unwrap();
    assert!(serialized.contains("[mqtt]"));
    assert!(serialized.contains("[s3]"));
    assert!(serialized.contains("[vision]"));
}

#[test]
fn test_config_serialization_roundtrip() {
    let config = thoth::Config::default();
    let serialized = toml::to_string(&config).unwrap();
    let deserialized: thoth::Config = toml::from_str(&serialized).unwrap();
    assert_eq!(deserialized.mqtt.broker, config.mqtt.broker);
    assert_eq!(deserialized.s3.endpoint, config.s3.endpoint);
    assert_eq!(deserialized.vision.model, config.vision.model);
}

fn create_dummy_png() -> Vec<u8> {
    let mut png_bytes = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        let img = image::DynamicImage::new_rgba8(1, 1);
        img.write_to(&mut cursor, image::ImageFormat::Png)
            .expect("failed to write dummy PNG");
    }
    png_bytes
}

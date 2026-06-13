use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

/// Teste que le client Pylos envoie correctement une requête
/// et parse la réponse JSON attendue.
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

/// Teste que le header X-Thoth-Secret est envoyé.
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

/// Teste que le fallback fonctionne quand le modèle principal échoue.
#[tokio::test]
async fn test_pylos_fallback_on_error() {
    let mock_server = MockServer::start().await;

    // Le modèle principal (gemma4:12b) retourne 500
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(wiremock::matchers::body_partial_json(
            serde_json::json!({"model": "gemma4:12b"}),
        ))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    // Le fallback (gemini4:12b) retourne une réponse valide
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

/// Teste la détection des données sensibles.
#[test]
fn test_sensitive_data_detection() {
    assert!(thoth::is_sensitive("sk-abcdefghijklmnopqrstuvwxyz1234"));
    assert!(!thoth::is_sensitive("Ceci est un texte normal"));
}

/// Teste la validation des langues supportées.
#[test]
fn test_validate_language() {
    assert!(thoth::validate_language("fr"));
    assert!(thoth::validate_language("en"));
    assert!(!thoth::validate_language("zz"));
}

/// Teste le parsing du pattern de hotkey.
#[test]
fn test_hotkey_pattern_parse() {
    let h = thoth::HotkeyPattern::parse("Ctrl+Shift+T").unwrap();
    assert_eq!(
        h.modifiers,
        vec![thoth::Modifier::Ctrl, thoth::Modifier::Shift]
    );
    assert_eq!(h.key, thoth::HotkeyKey::Letter('t'));
}

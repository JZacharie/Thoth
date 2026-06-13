## Story: Client HTTP Pylos

**ID**: S1-THOTH-04
**Épic**: EPIC-THOTH-01
**Points**: 3
**Statut**: DONE

---

### User Story

**As a** Developer
**I want** que Thoth envoie le texte à la passerelle locale Pylos via l'API OpenAI-compatible
**So that** le texte soit traité par le modèle LLM

---

### Acceptance Criteria

- [ ] Given un texte à traduire, when Thoth envoie `POST /v1/chat/completions` à Pylos, alors la requête est correctement formatée
- [ ] Given la réponse contient le texte traduit, when Thoth parse la réponse JSON, alors le texte est extrait du champ `choices[0].message.content`
- [ ] Given Pylos est injoignable, when la requête échoue, alors Thoth logge l'erreur et ne plante pas
- [ ] Given la réponse est lente, when le timeout est dépassé (10s), alors Thoth annule la requête
- [ ] Given le prompt système strict, when la requête est envoyée, alors le LLM ne renvoie QUE le texte traduit (pas de preamble)
- [ ] **SECURITE** : Given la requête est envoyée à Pylos, when le header `X-Thoth-Secret` est présent, alors Pylos peut authentifier Thoth sur le loopback
- [ ] **SECURITE** : Given le header secret est généré, when aucun secret n'est configuré, alors Thoth génère un UUID aléatoire et le persiste dans config.toml

---

### Technical Notes

**Fichier**: `src/pylos_client.rs`

**Configuration**:
```rust
pub struct PylosConfig {
    pub endpoint: String,        // http://localhost:3000
    pub model: String,           // gemma4:12b
    pub timeout_secs: u64,       // 10
}
```

**Requête**:
```rust
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,   // "system" | "user"
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
```

**Prompt système** (extraire dans `config.rs`):
```
Tu es un traducteur et correcteur de texte ultra-précis.
Ta tâche est de traduire, corriger l'orthographe/grammaire et rendre le texte fourni clair et concis.
Tu dois UNIQUEMENT retourner le texte corrigé et traduit.
Ne commence JAMAIS ta réponse par des formules de politesse, des introductions ou des explications.
Ne mets pas de guillemets ou de blocs de code markdown autour de ta réponse.
```

**Header secret** (ADR-004) :
```rust
client.post(&config.endpoint)
    .header("X-Thoth-Secret", &config.secret)
    .json(&request)
    .send()
```

---

### Definition of Done

- [ ] Requête POST correctement formatée vers Pylos
- [ ] Header `X-Thoth-Secret` inclus dans chaque requête
- [ ] Parsing de la réponse JSON
- [ ] Timeout configurable
- [ ] Gestion des erreurs (connexion refusée, timeout, réponse invalide)
- [ ] Tests unitaires avec mock HTTP
- [ ] `cargo clippy` et `cargo test` passent

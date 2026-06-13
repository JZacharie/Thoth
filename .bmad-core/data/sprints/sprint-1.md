# Sprint 1 — Plan

**Période**: 2026-05-10 → 2026-05-24
**Objectif**: Rendre Pylos production-ready sur les fondations critiques

---

## Sprint Goal

Ajouter la persistance SQLite des logs, le provider Azure OpenAI, et les embeddings —
les trois gaps les plus bloquants pour un déploiement production.

---

## Stories

| ID | Titre | Points | Statut | Épic |
|---|---|---|---|---|
| S1-01 | SQLite log persistence | 5 | DONE | EPIC-01 |
| S1-02 | Log retention + cleanup job | 2 | DONE | EPIC-01 |
| S1-03 | Log API enrichi (filtres avancés) | 3 | DONE | EPIC-01 |
| S1-04 | Azure OpenAI provider | 5 | DONE | EPIC-02b |
| S1-05 | Embeddings API (POST /v1/embeddings) | 5 | DONE | EPIC-03 |
| S1-06 | Weighted key selection | 2 | DONE | EPIC-04e |

**Total**: 22 points — **SPRINT COMPLETED** ✓

---

## Story S1-01: SQLite Log Persistence

**As a** Platform Engineer
**I want** Pylos to persist request logs to SQLite
**So that** logs survive restarts and are queryable over time

### Acceptance Criteria

- [ ] Given a chat completion request, when the response is sent, then a log entry is written to SQLite
- [ ] Given Pylos restarts, when I query `/api/logs`, then historical logs are returned
- [ ] Given >10k log entries, when I add more, then old entries are NOT dropped (unlike current ring buffer)
- [ ] Given a log query with `?provider=openai&model=gpt-4`, then only matching logs are returned
- [ ] Given a log query with `?start_time=X&end_time=Y`, then only logs in the time window are returned

### Technical Notes

**Bifrost source**: `framework/logstore/logstore.go`

**Affected crates**:
- `pylos-application` — new `SqliteLogStore` implementing existing `LogStore` trait
- `pylos-server` — inject `SqliteLogStore` instead of `InMemoryLogStore`

**Dependencies** (`Cargo.toml`):
```toml
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio", "chrono"] }
```

**Schema** (`migrations/001_create_logs.sql`):
```sql
CREATE TABLE IF NOT EXISTS requests (
    id TEXT PRIMARY KEY,
    timestamp INTEGER NOT NULL,  -- Unix ms
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    status TEXT NOT NULL,        -- 'success' | 'error'
    latency_ms INTEGER NOT NULL,
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    cost_usd REAL,
    virtual_key_id TEXT,
    error_message TEXT
);
CREATE INDEX idx_requests_timestamp ON requests(timestamp);
CREATE INDEX idx_requests_provider ON requests(provider);
CREATE INDEX idx_requests_model ON requests(model);
CREATE INDEX idx_requests_vk ON requests(virtual_key_id);
```

**Config** (`pylos.json`):
```json
{
  "server": {
    "log_storage": "sqlite",
    "log_db_path": "./pylos-logs.db",
    "log_retention_days": 365
  }
}
```

**New/modified endpoints**:
- `GET /api/logs?provider=X&model=Y&status=Z&start_time=T1&end_time=T2&limit=N&offset=M`

### DoD
- [ ] Unit tests for SqliteLogStore CRUD
- [ ] Migration runs on first start
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Config documented

---

## Story S1-02: Log Retention + Cleanup Job

**As a** Platform Engineer
**I want** old logs to be automatically purged after a configurable number of days
**So that** the SQLite database doesn't grow unbounded

### Acceptance Criteria

- [ ] Given `log_retention_days: 30` in config, when a background job runs, then logs older than 30 days are deleted
- [ ] Given the cleanup job runs, when I check logs, then recent logs are not affected
- [ ] The cleanup job runs once per day at startup and then every 24h

### Technical Notes

- Tokio background task spawned in `pylos-server/src/main.rs`
- `DELETE FROM requests WHERE timestamp < ?` with `UNIX_TIMESTAMP() - retention_days * 86400`
- Points: 2

---

## Story S1-03: Log API Filtres Avancés

**As a** ML Engineer
**I want** to filter logs by virtual key ID and see aggregated stats per model
**So that** I can debug usage for a specific VK or model

### Acceptance Criteria

- [ ] `GET /api/logs?virtual_key_id=sk-pylos-xxx` returns only logs for that VK
- [ ] `GET /api/logs/stats` returns `{total_requests, success_rate, avg_latency_ms, total_tokens, total_cost_usd}` — already exists but now queries SQLite
- [ ] `GET /api/logs/stats?provider=anthropic&model=claude-3-5-sonnet` returns stats filtered by provider+model

### Technical Notes

- Extend `LogQuery` struct with `virtual_key_id: Option<String>` filter
- Reuse existing `/api/logs/stats` handler, swap data source to SQLite
- Points: 3

---

## Story S1-04: Azure OpenAI Provider

**As a** Platform Engineer
**I want** to route requests to Azure OpenAI
**So that** enterprise deployments using Azure can use Pylos as their gateway

### Acceptance Criteria

- [ ] Given an Azure provider configured in `pylos.json`, when I send `POST /v1/chat/completions`, then the request is forwarded to Azure OpenAI
- [ ] Given a streaming request to Azure, when the response streams, then SSE chunks are forwarded correctly
- [ ] Given an invalid Azure API key, when I send a request, then `401 Unauthorized` is returned
- [ ] Given model affinity, when `gpt-4` is in the request and an Azure provider is configured for `gpt-4`, then Azure is preferred

### Technical Notes

**Bifrost source**: `core/providers/azure/`

**Bifrost Azure specifics**:
- URL: `https://{resource_name}.openai.azure.com/openai/deployments/{deployment_name}/chat/completions?api-version=2024-02-01`
- Auth header: `api-key: {key}` (not `Authorization: Bearer`)
- Model param ignored — deployment name is in URL
- Otherwise OpenAI-compatible wire format

**Affected crates**:
- `pylos-core` — add `ProviderKind::Azure`, add `AzureConfig { resource_name, deployment_name, api_version }` to `ProviderConfig`
- `pylos-infrastructure` — new `AzureProvider` in `providers/azure/`
- `pylos-application` — add Azure model affinity

**Config**:
```json
{
  "providers": [{
    "name": "my-azure",
    "kind": "azure",
    "api_key": "env.AZURE_API_KEY",
    "azure_config": {
      "resource_name": "my-resource",
      "deployment_name": "gpt-4-deployment",
      "api_version": "2024-02-01"
    }
  }]
}
```

**New error variants**: None (reuse existing)

### DoD
- [ ] Unit tests for URL construction
- [ ] Unit tests for request/response conversion
- [ ] Integration test with mock HTTP server
- [ ] Points: 5

---

## Story S1-05: Embeddings API

**As a** ML Engineer
**I want** to call `POST /v1/embeddings` through Pylos
**So that** I can use Pylos as a unified gateway for embeddings as well as chat

### Acceptance Criteria

- [ ] Given `POST /v1/embeddings` with `{"model": "text-embedding-3-small", "input": "hello"}`, then Pylos forwards to OpenAI and returns embeddings
- [ ] Given an unsupported provider for embeddings, then `501 Not Implemented` is returned (not a 500)
- [ ] Given streaming=false (embeddings don't stream), then response is returned as JSON directly
- [ ] Given multiple inputs in array form, then all embeddings are returned

### Technical Notes

**Bifrost source**: `core/providers/openai/embedding.go`

**New domain types** (`pylos-core`):
```rust
pub struct EmbeddingRequest {
    pub model: String,
    pub input: EmbeddingInput,  // String | Vec<String>
    pub encoding_format: Option<String>,  // "float" | "base64"
    pub dimensions: Option<u32>,
    pub user: Option<String>,
}

pub struct EmbeddingResponse {
    pub object: String,  // "list"
    pub data: Vec<Embedding>,
    pub model: String,
    pub usage: EmbeddingUsage,
}

pub struct Embedding {
    pub index: usize,
    pub object: String,  // "embedding"
    pub embedding: Vec<f32>,
}
```

**Provider trait extension** (`pylos-core/src/domain/traits.rs`):
```rust
async fn embed(&self, request: &EmbeddingRequest) -> Result<EmbeddingResponse, PylosError> {
    Err(PylosError::Unsupported("embeddings not supported by this provider".into()))
}
```

**Affected crates**:
- `pylos-core` — new types + default `embed()` in `Provider` trait
- `pylos-infrastructure/providers/openai` — implement `embed()`
- `pylos-infrastructure/providers/anthropic` — default (not supported)
- `pylos-server` — new `POST /v1/embeddings` route

### DoD
- [ ] OpenAI embeddings work end-to-end
- [ ] Unsupported providers return 501
- [ ] Points: 5

---

## Story S1-06: Weighted Key Selection

**As a** Platform Engineer
**I want** to assign weights to API keys for a provider
**So that** I can distribute load between keys proportionally (e.g., 70% key A, 30% key B)

### Acceptance Criteria

- [ ] Given two keys with weights 7.0 and 3.0, when 1000 requests are made, then ~70% go to key A and ~30% to key B
- [ ] Given a key with weight 0, when requests are made, then that key is never selected
- [ ] Given all keys have equal weight (or no weight specified), then selection is uniform random

### Technical Notes

**Bifrost source**: `core/keyselectors/weighted_random.go`

**Current Pylos**: round-robin or first key. Change to weighted random.

**Config**:
```json
{
  "providers": [{
    "name": "openai",
    "keys": [
      {"value": "env.KEY_A", "weight": 7.0},
      {"value": "env.KEY_B", "weight": 3.0}
    ]
  }]
}
```

**Affected crates**:
- `pylos-core` — add `weight: f64` to `ApiKey` / key config struct
- `pylos-infrastructure` — new `select_key_weighted()` function
- `pylos-application` — use weighted selection in `InferenceOrchestrator`

### DoD
- [ ] Unit test validates distribution (statistical, 10000 iterations)
- [ ] Default weight=1.0 when not specified
- [ ] Points: 2

---

## Blockers

- Aucun connu au début du sprint

## Notes

- Utiliser `sqlx` avec migrations inline (`sqlx::migrate!()`)
- Tester l'Azure provider avec un mock HTTP server (pas besoin de vraies clés Azure)
- L'API embeddings doit être testable via Ollama (modèle `nomic-embed-text`)

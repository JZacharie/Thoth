# Template: New Provider

Use this template when adding a new LLM provider to Pylos.

---

## Provider: [Provider Name]

**API Type**: [OpenAI-compatible / Custom / Anthropic-compatible]
**Streaming**: [Yes / No]
**Auth method**: [API Key / IAM / OAuth / None]

---

## Implementation Checklist

### 1. Domain Layer (`pylos-core`)

- [ ] Add variant to `ProviderKind` enum in `crates/pylos-core/src/domain/provider.rs`
  ```rust
  pub enum ProviderKind {
      // ... existing
      MyProvider,
  }
  ```

- [ ] Add any new config fields to `ProviderConfig` in `crates/pylos-core/src/domain/config.rs`
  ```rust
  // Document new fields here
  ```

- [ ] Add new `PylosError` variants if needed in `crates/pylos-core/src/error.rs`

### 2. Infrastructure Layer (`pylos-infrastructure`)

- [ ] Create module: `crates/pylos-infrastructure/src/providers/<name>/mod.rs`
- [ ] Create client: `crates/pylos-infrastructure/src/providers/<name>/client.rs`
- [ ] Implement `Provider` trait:
  ```rust
  #[async_trait]
  impl Provider for MyProviderClient {
      async fn complete(&self, request: &PylosRequest) -> Result<PylosResponse, PylosError> { ... }
      async fn stream(&self, request: &PylosRequest) -> Result<BoxStream<StreamChunk>, PylosError> { ... }
      async fn health_check(&self) -> Result<(), PylosError> { ... }
  }
  ```
- [ ] Handle request format conversion (from OpenAI format to provider format)
- [ ] Handle response format conversion (from provider format to OpenAI format)
- [ ] Handle SSE streaming with `async-stream`
- [ ] Export module in `crates/pylos-infrastructure/src/providers/mod.rs`

### 3. Application Layer (`pylos-application`)

- [ ] Add model affinity rules in `crates/pylos-application/src/use_cases/inference.rs`
  ```rust
  // Example: route "myprovider/" prefixed models to MyProvider
  fn affinity_score(provider_kind: &ProviderKind, model: &str) -> u8 { ... }
  ```

### 4. Server Layer (`pylos-server`)

- [ ] Register provider in provider factory
- [ ] Add any new metrics if needed

### 5. Configuration

- [ ] Document new provider config in `pylos.json` example:
  ```json
  {
    "providers": [
      {
        "name": "my-provider",
        "kind": "my_provider",
        "api_key": "env.MY_PROVIDER_API_KEY",
        "base_url": "https://api.myprovider.com/v1",
        "models": ["model-a", "model-b"]
      }
    ]
  }
  ```

### 6. Tests

- [ ] Unit tests for request/response format conversion
- [ ] Unit tests for error mapping
- [ ] Mock-based tests for the `Provider` trait implementation

---

## Request/Response Format

### Input (PylosRequest → Provider Format)

Describe mapping here.

### Output (Provider Format → PylosResponse)

Describe mapping here.

### Streaming (Provider SSE → StreamChunk)

Describe SSE format here.

---

## Error Mapping

| Provider Error | PylosError Variant |
|---|---|
| 401 Unauthorized | `PylosError::Unauthorized` |
| 429 Rate Limited | `PylosError::RateLimitExceeded` |
| 408/504 Timeout | `PylosError::Timeout` |
| Other 4xx | `PylosError::InvalidRequest` |
| 5xx | `PylosError::ProviderError` |

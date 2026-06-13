# Bifrost → Pylos: Analyse Architecturale BMAD

**Date**: 2026-05-10
**Auteur**: Architect (Larry) via BMAD
**Statut**: Référence de migration

---

## Résumé Exécutif

Bifrost est un AI Gateway Go de production (24 providers, 20 000+ lignes de code).
Pylos est son port Rust en cours, avec une architecture hexagonale solide mais couvrant
seulement une fraction des fonctionnalités de Bifrost.

Ce document identifie les gaps, priorise les extractions, et guide l'implémentation Rust.

---

## 1. Inventaire Bifrost vs Pylos

### Providers

| Provider | Bifrost | Pylos | Priorité |
|---|---|---|---|
| OpenAI | Complet | Complet | - |
| Anthropic | Complet | Complet | - |
| AWS Bedrock | Complet + STS | Complet + STS | - |
| Ollama | Complet | Complet (via OpenAI) | - |
| OpenRouter | Complet | Complet | - |
| Azure OpenAI | Complet | MANQUANT | P1 |
| Gemini (Google) | Complet | MANQUANT | P1 |
| Groq | Complet | MANQUANT | P2 |
| Cohere | Complet | MANQUANT | P2 |
| Mistral | Complet | MANQUANT | P2 |
| Vertex AI | Complet | MANQUANT | P3 |
| vLLM | Complet | MANQUANT | P3 |
| Perplexity | Complet | MANQUANT | P3 |
| Cerebras | Complet | MANQUANT | P3 |
| Fireworks | Complet | MANQUANT | P3 |
| HuggingFace | Complet | MANQUANT | P3 |
| xAI (Grok) | Complet | MANQUANT | P3 |
| Replicate | Complet | MANQUANT | P4 |
| ElevenLabs | Complet | MANQUANT | P4 |
| Nebius | Complet | MANQUANT | P4 |
| SGL | Complet | MANQUANT | P4 |
| Runway | Complet | MANQUANT | P4 |
| Parasail | Complet | MANQUANT | P4 |

### Features Core

| Feature | Bifrost | Pylos | Gap |
|---|---|---|---|
| Chat Completion (stream+unary) | Oui | Oui | Aucun |
| Text Completion | Oui | Non | Ajouter route POST /v1/completions |
| Embeddings | Oui | Non | **Majeur** |
| Reranking | Oui | Non | Majeur |
| TTS (Speech) | Oui | Non | Majeur |
| Transcription | Oui | Non | Majeur |
| Image Generation | Oui | Non | Majeur |
| Image Edit/Variation | Oui | Non | Majeur |
| OCR | Oui | Non | Moyen |
| Video Generation | Oui | Non | Bas |
| Batch API | Oui | Non | Moyen |
| Files API | Oui | Non | Moyen |
| Responses API (OpenAI) | Oui | Non | Moyen |
| Fallbacks multi-provider | Oui | Oui | Aucun |
| Retry + backoff | Oui | Oui | Aucun |
| Streaming SSE | Oui | Oui | Aucun |
| WebSocket Realtime | Oui | Non | Bas |
| Model affinity routing | Simple | Simple | Améliorer |

### Governance

| Feature | Bifrost | Pylos | Gap |
|---|---|---|---|
| Virtual Keys (sk-pylos-*) | sk-bf-* | Oui | Format différent, OK |
| Rate limiting (tokens/req) | Oui | Partiel | Compléter |
| Budget USD (daily/monthly) | Oui | Non | **Majeur** |
| Model ACLs (allowlist/denylist) | Oui | Partiel | Compléter |
| Teams hierarchy | Oui | Non | P2 |
| Customers hierarchy | Oui | Non | P2 |
| CEL routing rules | Oui | Non | P2 |
| Key weight (load balancing) | Oui | Non | P2 |
| Key aliases (model → profile) | Oui | Non | P3 |
| Per-VK model config | Oui | Non | P2 |
| Mandatory VK enforcement | Oui | Partiel | Compléter |

### Observabilité

| Feature | Bifrost | Pylos | Gap |
|---|---|---|---|
| Prometheus metrics | Oui | Oui | Aucun |
| Request logs (SQLite/PG) | SQLite+PG | In-memory | **Majeur** — persistance |
| Log filtering/search | Oui | Partiel | Compléter |
| Cost estimation | Oui | Partiel | Compléter |
| Tracing (OTel) | Oui | Partiel | Compléter |
| Grafana provisioning | Oui | Oui | Aucun |

### Plugin System

| Feature | Bifrost | Pylos | Gap |
|---|---|---|---|
| LLMPlugin (pre/post hooks) | Oui | Oui | Aucun (trait existe) |
| HTTPTransportPlugin | Oui | Non | P2 |
| MCPPlugin | Oui | Non | P3 |
| ObservabilityPlugin (async) | Oui | Non | P2 |
| Short-circuit (cache hit) | Oui | Non | P2 |
| Plugin placement/ordering | Oui | Non | P2 |
| Dynamic plugin loading (.so) | Oui | Non | P4 |
| Semantic cache plugin | Oui | Non | P3 |
| Governance plugin | Oui | Non | P1 (intégré) |
| Telemetry plugin | Oui | Partiel | Compléter |

### MCP (Model Context Protocol)

| Feature | Bifrost | Pylos | Gap |
|---|---|---|---|
| MCP client (stdio/SSE/HTTP) | Oui | Non | P3 |
| Multi-turn tool loop | Oui | Non | P3 |
| MCP server endpoint | Oui | Non | P3 |
| MCP tool filtering | Oui | Non | P3 |
| Per-VK MCP access | Oui | Non | P3 |

### Config & Runtime

| Feature | Bifrost | Pylos | Gap |
|---|---|---|---|
| Config hot-reload | Oui | Oui | Aucun |
| Config depuis SQLite/PG | Oui | Non | P2 |
| Config API CRUD | 80+ endpoints | Partiel | Compléter |
| Encryption secrets (AES-256) | Oui | Non | P2 |
| env.VAR_NAME interpolation | Oui | Oui | Aucun |
| OAuth2 SSO | Oui | Non | P4 |
| JWT auth | Oui | Non | P2 |
| CORS configuré | Oui | Oui | Aucun |

### UI Dashboard

| Feature | Bifrost | Pylos | Gap |
|---|---|---|---|
| Dashboard KPIs | Oui | Oui | Aucun |
| Request logs UI | Oui | Oui | Aucun |
| Provider management UI | Oui | Oui | Aucun |
| Virtual keys UI | Oui | Oui | Aucun |
| Routing rules UI | Oui | Non | P2 |
| Model catalog UI | Oui | Non | P2 |
| MCP registry UI | Oui | Non | P3 |
| Prompt repository UI | Oui | Non | P3 |
| Governance hierarchy UI | Oui | Non | P2 |
| Plugin management UI | Oui | Non | P2 |

---

## 2. Patterns clés à porter en Rust

### 2.1 Channel-based async dispatch (Bifrost Go → Pylos Rust)

**Bifrost (Go):**
```go
type ProviderQueue struct {
    queue chan *ChannelMessage  // buffered channel, never closed
    done  chan struct{}          // close signal
}
// Workers: select { case msg := <-queue: ... case <-done: return }
```

**Pylos (Rust équivalent):**
```rust
// tokio::sync::mpsc channels (DÉJÀ PRÉSENT partiellement)
// Améliorer avec:
use tokio::sync::{mpsc, broadcast};

struct ProviderQueue {
    tx: mpsc::Sender<ChannelMessage>,
    shutdown: broadcast::Sender<()>,
}
// Workers: tokio::select! { msg = rx.recv() => ..., _ = shutdown.recv() => break }
```

### 2.2 Atomic provider list hot-swap (Bifrost → Pylos)

**Bifrost (Go):**
```go
providers atomic.Pointer[[]Provider]
// Reload: newList = ...; providers.Store(&newList)
```

**Pylos (Rust équivalent):**
```rust
// DÉJÀ PRÉSENT: Arc<RwLock<Vec<Box<dyn Provider>>>>
// Améliorer avec arc-swap crate pour lock-free reads:
use arc_swap::ArcSwap;
providers: ArcSwap<Vec<Box<dyn Provider + Send + Sync>>>
```

### 2.3 Plugin short-circuit

**Bifrost:**
```go
type LLMPluginShortCircuit struct {
    Response *BifrostResponse
    Stream   chan *BifrostStreamChunk
    Error    *BifrostError
}
```

**Pylos Rust:**
```rust
pub enum PluginOutcome {
    Continue,
    ShortCircuit(PylosResponse),
    ShortCircuitStream(BoxStream<'static, StreamChunk>),
    Error(PylosError),
}
```

### 2.4 Streaming accumulator

**Bifrost:** chunk accumulator runs in parallel, merges delta updates for post-hooks.

**Pylos:** ajouter un `StreamAccumulator` dans `pylos-application`:
```rust
pub struct StreamAccumulator {
    content: String,
    tool_calls: Vec<ToolCall>,
    usage: Option<Usage>,
    finish_reason: Option<FinishReason>,
}
```

### 2.5 Multi-type request union

**Bifrost** a un `BifrostRequest` union avec 30+ variantes.

**Pylos** ne supporte que `ChatCompletionRequest`. Étendre:
```rust
pub enum PylosRequest {
    ChatCompletion(ChatCompletionRequest),
    TextCompletion(TextCompletionRequest),  // à ajouter
    Embedding(EmbeddingRequest),             // à ajouter
    Speech(SpeechRequest),                   // à ajouter
    // etc.
}
```

### 2.6 Fallback mechanism

**Bifrost:**
```go
type Fallback struct {
    Provider ModelProvider
    Model    string
}
```

**Pylos:** le fallback est déjà implémenté en itérant sur les providers. Formaliser:
```rust
pub struct Fallback {
    pub provider: String,
    pub model: String,
}
// Dans ChatCompletionRequest:
pub fallbacks: Option<Vec<Fallback>>,
```

### 2.7 Weighted Key Selection

**Bifrost:** `WeightedRandom` key selector (~10ns).

**Pylos:** ajouter dans `pylos-infrastructure`:
```rust
pub fn select_key_weighted(keys: &[ApiKey]) -> Option<&ApiKey> {
    let total: f64 = keys.iter().map(|k| k.weight).sum();
    let mut rng = fastrand::f64() * total;
    for key in keys {
        rng -= key.weight;
        if rng <= 0.0 { return Some(key); }
    }
    keys.last()
}
```

### 2.8 Model Catalog

**Bifrost:** `framework/modelcatalog` — model metadata + pricing sync.

**Pylos:** créer `crates/pylos-application/src/model_catalog.rs`:
```rust
pub struct ModelInfo {
    pub provider: String,
    pub model_id: String,
    pub context_window: u32,
    pub input_price_per_1k: f64,   // USD
    pub output_price_per_1k: f64,  // USD
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub supports_streaming: bool,
}
```

---

## 3. Écarts architecturaux critiques

### 3.1 Persistance des logs

**Bifrost:** SQLite + PostgreSQL via GORM, 365 jours de rétention, queryable.
**Pylos:** ring buffer in-memory (10k entries max), perdu au restart.

**Impact:** blocant pour production. Ajouter SQLite via `sqlx` ou `rusqlite`.

### 3.2 Model Catalog et pricing

**Bifrost:** catalog complet avec pricing sync depuis URL, heuristiques par tier.
**Pylos:** heuristiques de coût inline dans `log_store.rs`, pas de catalog.

**Impact:** coût estimé incorrect pour nouveaux modèles.

### 3.3 Config persistence

**Bifrost:** config complète stockée en DB, CRUD API, hot-reload atomique.
**Pylos:** config depuis `pylos.json` uniquement, hot-reload partiel.

**Impact:** pas d'admin UI pour éditer les providers/VKs sans restart.

### 3.4 Governance hiérarchique

**Bifrost:** VK → Team → Customer → Business Unit, avec budgets et rate limits à chaque niveau.
**Pylos:** VK uniquement, rate limits basiques.

**Impact:** pas de gouvernance multi-tenant.

### 3.5 Types de requêtes

**Bifrost:** 15+ types (chat, text, embedding, speech, image, video, batch, files, OCR...).
**Pylos:** chat uniquement.

**Impact:** ne couvre qu'un subset des cas d'usage LLM.

---

## 4. Ce qui est bien dans Pylos et à conserver

1. **Architecture hexagonale 4-crates** — meilleure que la structure Go de Bifrost
2. **Traits `Provider` et `LlmPlugin`** — bien définis, à étendre plutôt que reécrire
3. **InferenceOrchestrator** — bon point d'entrée, à enrichir
4. **Prometheus metrics** — déjà bien intégrées
5. **LogStore avec ring buffer** — bon pour dev, remplacer par SQLite en prod
6. **UI React** — fonctionnelle, à étendre
7. **Docker Compose + Grafana** — infrastructure monitoring complète
8. **CI/CD avec clippy + fmt + audit** — qualité code maintenue

---

## 5. Recommandations de migration

### Ne PAS porter (complexité vs valeur)
- WebRTC Realtime (niche, complexe)
- Video generation (Runway — très spécifique)
- Starlark code execution sandbox (complexité élevée)
- OAuth2 SSO pour MCP (enterprise, P4)
- Dynamic plugin loading (.so) — Rust a un meilleur modèle statique

### Porter en priorité P1 (sprint 1-2)
1. **Persistance SQLite** pour les logs (remplacer ring buffer)
2. **Azure OpenAI provider** (très utilisé en enterprise)
3. **Gemini provider** (Google)
4. **Budget governance** (cost caps per VK)
5. **Embeddings** (`POST /v1/embeddings`)

### Porter en priorité P2 (sprint 3-4)
6. **Config CRUD API** (persistance des providers/VKs en DB)
7. **Governance hiérarchique** (Teams, Customers)
8. **CEL routing rules**
9. **Plugin short-circuit** (pour semantic cache)
10. **Groq, Cohere, Mistral providers**

### Porter en priorité P3 (sprint 5+)
11. **MCP client** (stdio + SSE)
12. **Semantic cache** (vector store)
13. **TTS et Transcription**
14. **Batch API et Files API**
15. **Model Catalog avec pricing sync**

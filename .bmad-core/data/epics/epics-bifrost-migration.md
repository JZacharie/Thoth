# Épics — Migration Bifrost → Pylos

**Source**: Analyse BMAD `.bmad-core/data/bifrost-analysis.md`
**Date**: 2026-05-10

---

## EPIC-01: Persistance des Logs (SQLite)

**Priorité**: P1 — Critique pour production
**Bifrost source**: `framework/logstore/`

**Problème**: Pylos perd tous les logs au restart (ring buffer in-memory).
**Solution**: Remplacer le `LogStore` in-memory par SQLite via `sqlx` + `rusqlite`.

**Scope**:
- Ajouter crate `pylos-persistence` (ou module dans `pylos-application`)
- Schema SQLite: table `requests` avec colonnes (id, timestamp, provider, model, status, latency_ms, prompt_tokens, completion_tokens, cost_usd, virtual_key_id, request_body?, response_body?)
- Migrer l'API `/api/logs*` pour requêter SQLite
- Conserver l'interface `LogStore` existante (swap implementation)
- Support optionnel PostgreSQL (feature flag)
- Rétention configurable (default 365 jours)

**Stories**: S1-01 à S1-03

---

## EPIC-02: Nouveaux Providers

**Priorité**: P1-P3 selon provider
**Bifrost source**: `core/providers/`

**Sous-épics par catégorie**:

### EPIC-02a: Providers OpenAI-compatibles (P2)
Ces providers délèguent au client OpenAI existant avec une base_url différente.
Implémentation : ajouter `ProviderKind` + affinity rules uniquement.

- Groq (`https://api.groq.com/openai/v1`)
- Cerebras (`https://api.cerebras.ai/v1`)
- Perplexity (`https://api.perplexity.ai`)
- Fireworks (`https://api.fireworks.ai/inference/v1`)
- Nebius (`https://api.studio.nebius.ai/v1`)
- Mistral (`https://api.mistral.ai/v1`)
- xAI/Grok (`https://api.x.ai/v1`)
- vLLM (configurable base URL)

### EPIC-02b: Azure OpenAI (P1)
Spécificités: URL format `https://{resource}.openai.azure.com/openai/deployments/{deployment}`, header `api-key`, version param.

### EPIC-02c: Gemini / Google GenAI (P1)
Format API distinct: `generateContent`, parts[], contenu multimodal différent de OpenAI.

### EPIC-02d: Cohere (P2)
Format propre: `/v1/chat`, `/v1/rerank`, `/v1/embed`.

### EPIC-02e: Vertex AI (P3)
Google Cloud Vertex AI, auth GCP ADC / service account.

**Stories**: S2-01 à S2-15

---

## EPIC-03: Types de Requêtes LLM

**Priorité**: P1-P2
**Bifrost source**: `core/schemas/`, `transports/bifrost-http/handlers/`

**Scope**:
- `POST /v1/embeddings` — Embeddings (P1)
- `POST /v1/completions` — Text completion (P2)
- `POST /v1/audio/speech` — TTS (P3)
- `POST /v1/audio/transcriptions` — Transcription (P3)
- `POST /v1/images/generations` — Image generation (P3)

**Architecture**:
- Étendre `PylosRequest` enum dans `pylos-core`
- Ajouter trait methods à `Provider`: `embed()`, `text_complete()`, etc.
- Ajouter routes dans `pylos-server`
- Chaque type optionnel → providers qui ne le supportent pas retournent `PylosError::Unsupported`

**Stories**: S3-01 à S3-10

---

## EPIC-04: Governance Avancée

**Priorité**: P1-P2
**Bifrost source**: `plugins/governance/`

### EPIC-04a: Budget USD (P1)
- Tracking coût par VK en temps réel
- Budget daily/monthly/total configurable
- Rejection avec `PylosError::BudgetExceeded` quand dépassé
- Reset automatique (daily/monthly)
- API `/api/virtual-keys/{id}/budget`

### EPIC-04b: Rate Limiting enrichi (P2)
- Limits par (tokens/requests) × (window: second/minute/hour/day)
- Actuellement: simple in-memory. Migrer vers SQLite pour persistance cross-restart.

### EPIC-04c: Hiérarchie Teams/Customers (P2)
- `VirtualKey → Team → Customer`
- Budgets et rate limits héritables/cumulatifs
- CRUD API pour teams et customers

### EPIC-04d: CEL Routing Rules (P2)
**Bifrost source**: `framework/routing/`
- Intégrer `cel-interpreter` crate Rust (ou évaluation custom)
- Expressions: `request.model == "gpt-4"`, `request.virtual_key == "sk-..."`, etc.
- Rules can override: provider, model, keys, fallbacks

### EPIC-04e: Weighted Key Load Balancing (P2)
- Chaque provider key a un `weight: f64`
- `WeightedRandom` selection dans `pylos-infrastructure`

**Stories**: S4-01 à S4-12

---

## EPIC-05: Config Persistence & CRUD API

**Priorité**: P2
**Bifrost source**: `framework/configstore/`

**Problème**: Pylos nécessite un restart pour changer providers/VKs.
**Solution**: Stocker la config en SQLite, exposer une CRUD API complète.

**Scope**:
- Schema SQLite pour providers, virtual_keys, rate_limits, budgets
- API endpoints manquants:
  - `POST /providers` — créer provider
  - `DELETE /providers/:name` — supprimer provider
  - `POST /virtual-keys` — créer VK
  - `PUT /virtual-keys/:id` — modifier VK
  - `DELETE /virtual-keys/:id` — supprimer VK
- Garder `pylos.json` comme source initiale (import au premier démarrage)
- Hot-reload automatique après modification API

**Stories**: S5-01 à S5-08

---

## EPIC-06: Plugin System Étendu

**Priorité**: P2-P3
**Bifrost source**: `core/schemas/plugin.go`, `plugins/`

### EPIC-06a: Plugin Short-Circuit (P2)
- `PreLLMHook` peut retourner `PluginOutcome::ShortCircuit(response)` pour sauter le provider
- Nécessaire pour semantic cache

### EPIC-06b: ObservabilityPlugin async (P2)
- Post-hook asynchrone appelé APRÈS que la réponse est envoyée au client
- Pour: logging asynchrone, métriques, tracing OTel

### EPIC-06c: Plugin Ordering & Placement (P2)
- `placement: PreBuiltin | Builtin | PostBuiltin`
- `order: u8` dans chaque groupe

### EPIC-06d: Semantic Cache Plugin (P3)
**Bifrost source**: `plugins/semanticcache/`
- Embedder le prompt, chercher dans vector store
- Si similarité > threshold: retourner réponse cached (short-circuit)
- Backends: Qdrant ou SQLite avec pgvector-lite

**Stories**: S6-01 à S6-08

---

## EPIC-07: MCP (Model Context Protocol)

**Priorité**: P3
**Bifrost source**: `core/mcp/`

**Scope**:
- Client MCP (stdio + SSE transports)
- Injection automatique des tools dans les requêtes
- Agent loop multi-turn (LLM → tool call → résultat → LLM)
- Endpoint MCP server (`/mcp`)
- Config dans `pylos.json`: `mcp.client_configs[]`
- Filtrage des tools par VK

**Stories**: S7-01 à S7-10

---

## EPIC-08: Model Catalog

**Priorité**: P2
**Bifrost source**: `framework/modelcatalog/`

**Scope**:
- Table `model_catalog` en SQLite
- Champs: provider, model_id, context_window, input_price_per_1k_tokens, output_price_per_1k_tokens, supports_vision, supports_tools, supports_streaming
- Sync depuis URL configurable (JSON externe)
- API `GET /v1/models` enrichie avec metadata
- Pricing exact (remplace les heuristiques inline du LogStore)

**Stories**: S8-01 à S8-05

---

## EPIC-09: OpenTelemetry

**Priorité**: P2
**Bifrost source**: `plugins/otel/`, `core/schemas/tracer.go`

**Scope**:
- Intégrer `opentelemetry` crate Rust (déjà déclaré dans workspace!)
- Spans: `llm.call`, `plugin`, `retry`, `fallback`
- Attributs gen_ai.*: `provider`, `model`, `input_tokens`, `output_tokens`, `latency`
- Export OTLP gRPC/HTTP
- Config dans `pylos.json`: `plugins[{name: "otel", config: {endpoint: "..."}}]`

**Stories**: S9-01 à S9-04

---

## EPIC-10: UI Enrichissement

**Priorité**: P2-P3
**Bifrost source**: `ui/app/workspace/`

**Features manquantes dans l'UI Pylos**:
- Routing rules UI (builder CEL)
- Model catalog UI
- Governance hierarchy (Teams/Customers)
- Plugin management UI
- MCP registry (P3)
- Budget tracking dashboard

**Stories**: S10-01 à S10-08

---

## Priorisation Globale

```
Sprint 1 (2 semaines): EPIC-01 (SQLite logs) + EPIC-03a (Embeddings) + EPIC-02b (Azure)
Sprint 2 (2 semaines): EPIC-02c (Gemini) + EPIC-04a (Budget USD) + EPIC-04e (Weighted keys)
Sprint 3 (2 semaines): EPIC-02a (OpenAI-compat providers) + EPIC-04b (Rate limits SQLite) + EPIC-05a (Config CRUD base)
Sprint 4 (2 semaines): EPIC-04c (Teams/Customers) + EPIC-04d (CEL Routing) + EPIC-06a-b (Plugin short-circuit)
Sprint 5+:             EPIC-07 (MCP) + EPIC-08 (Model catalog) + EPIC-09 (OTel) + EPIC-06d (Semantic cache)
```

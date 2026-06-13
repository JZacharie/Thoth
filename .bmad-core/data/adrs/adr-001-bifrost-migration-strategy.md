# ADR-001: Stratégie de Migration Bifrost → Pylos

**Date**: 2026-05-10
**Status**: Accepted
**Deciders**: Architect (Larry), Dev

---

## Context

Bifrost est un AI Gateway Go mature avec 24 providers, governance complète, MCP, et plus.
Pylos est son port Rust en cours, avec une architecture hexagonale solide mais couvrant
seulement ~20% des fonctionnalités de Bifrost.

La question est: comment migrer les features de Bifrost vers Pylos de manière systématique
et prioritaire, sans casser ce qui existe déjà.

---

## Decision Drivers

- Pylos doit être production-ready le plus tôt possible
- L'architecture hexagonale Rust de Pylos est meilleure que la structure Go de Bifrost
- Certaines features Bifrost sont complexes (WebRTC, Starlark, .so plugins) et peu prioritaires
- Les traits Rust `Provider` et `LlmPlugin` sont bien définis et doivent être préservés
- La compatibilité OpenAI doit rester complète à tout moment

---

## Considered Options

1. **Réécriture complète** — tout porter d'un coup, merge final
2. **Migration feature-by-feature** — ajouter features une par une, toujours deployable
3. **Fork Bifrost en Rust** — traduire mécaniquement le Go en Rust
4. **Abandon Pylos** — utiliser Bifrost Go directement

---

## Decision

**Option 2: Migration feature-by-feature** par ordre de priorité business.

---

## Consequences

### Positive

- Pylos reste deployable et testable à chaque sprint
- On ne porte que ce qui a de la valeur (pas de video generation si pas nécessaire)
- L'architecture hexagonale Rust est préservée et améliorée
- Les patterns Go de Bifrost sont adaptés idiomatiquement en Rust (pas traduits)

### Negative / Trade-offs

- Delta fonctionnel entre Bifrost et Pylos pendant plusieurs sprints
- Les features Bifrost avancées (CEL routing, semantic cache) prennent du temps

---

## Règles d'Architecture pour la Migration

### Règle 1: Adapter, ne pas traduire
Ne pas traduire mécaniquement le Go en Rust. Adapter les patterns:
- `chan *ChannelMessage` (Go) → `tokio::sync::mpsc` (Rust)
- `atomic.Pointer[[]Provider]` (Go) → `arc_swap::ArcSwap` (Rust)
- `sync.Pool` (Go) → pool custom ou `typed-arena` (Rust, si nécessaire)
- `interface{}` (Go) → `enum` ou `Box<dyn Trait>` (Rust)

### Règle 2: Étendre les traits, ne pas les casser
Le trait `Provider` dans `pylos-core/src/domain/traits.rs` est la fondation.
Pour chaque nouveau type de requête (embeddings, speech...):
- Ajouter une méthode avec implémentation **default** retournant `Err(PylosError::Unsupported)`
- Les providers existants ne sont PAS forcés à implémenter la nouvelle méthode
- Les providers qui supportent la feature l'implémentent explicitement

```rust
// Pattern: default = unsupported
async fn embed(&self, _req: &EmbeddingRequest) -> Result<EmbeddingResponse, PylosError> {
    Err(PylosError::Unsupported("embeddings".into()))
}
```

### Règle 3: Persistance SQLite first, PostgreSQL second
- Implémenter les stores avec `sqlx` + feature `sqlite`
- Ajouter support PostgreSQL via feature flag `postgres` dans un second temps
- Interface `LogStore` / `ConfigStore` abstraite → swap d'implémentation sans changement de code métier

### Règle 4: Config backward-compatible
Chaque nouveau champ `pylos.json` doit avoir une valeur par défaut via `serde(default)`.
Aucune config existante ne doit casser lors d'une mise à jour.

### Règle 5: Ne pas porter ces features (rapport coût/valeur défavorable)
- WebRTC Realtime (remplacer par WebSocket direct si besoin)
- Starlark code execution sandbox
- OAuth2 SSO pour MCP clients
- Dynamic plugin loading (.so) — Rust favorise les plugins statiques
- Video generation (Runway) — trop spécifique

### Règle 6: Plugin short-circuit avant semantic cache
Le semantic cache nécessite le short-circuit dans le pipeline de plugins.
Implémenter EPIC-06a avant EPIC-06d.

---

## Mapping des Concepts Bifrost → Pylos

| Concept Bifrost | Équivalent Pylos |
|---|---|
| `BifrostConfig` | `PylosConfig` dans `pylos-core` |
| `Account` interface | Config provider dans `ProviderConfig` |
| `Provider` interface | `Provider` trait dans `pylos-core/domain/traits.rs` |
| `LLMPlugin` interface | `LlmPlugin` trait dans `pylos-core/domain/traits.rs` |
| `BifrostContext` | `RequestContext` dans `pylos-core` |
| `BifrostError` | `PylosError` enum dans `pylos-core/error.rs` |
| `ChannelMessage` | `PylosRequest` (direct, pas de channel pour l'instant) |
| `ProviderQueue` chan | `InferenceOrchestrator` avec concurrency limitée |
| `Fallback[]` | Providers list avec retry dans `InferenceOrchestrator` |
| `EnvVar` type | Résolution `env.X` dans config parser |
| `WhiteList/BlackList` | `VirtualKeyConfig.allowed_models` |
| `GovernancePlugin` | Middleware virtuel key dans `pylos-server` |
| `LogStore` interface | `LogStore` trait dans `pylos-application` |
| `ConfigStore` interface | Nouveau `ConfigStore` trait à créer |
| `ModelCatalog` | Nouveau dans `pylos-application` |

---

## References

- Bifrost architecture: `/home/joseph/git/bifrost/core/bifrost.go`
- Bifrost schemas: `/home/joseph/git/bifrost/core/schemas/`
- Pylos provider trait: `crates/pylos-core/src/domain/traits.rs`
- Pylos error types: `crates/pylos-core/src/error.rs`
- Analyse complète: `.bmad-core/data/bifrost-analysis.md`
- Épics migration: `.bmad-core/data/epics/epics-bifrost-migration.md`

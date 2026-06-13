## Story: Métriques d'Utilisation Locales

**ID**: S3-THOTH-03
**Épic**: EPIC-THOTH-03
**Points**: 2
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** voir des statistiques d'utilisation (nombre de traductions, temps moyen)
**So that** je puisse mesurer l'utilité de Thoth au quotidien

---

### Acceptance Criteria

- [ ] Given Thoth traduit un texte, when l'opération termine, alors un compteur est incrémenté
- [ ] Given l'utilisateur clique sur "Statistiques" dans le menu tray, when la fenêtre s'affiche, alors elle montre : nombre total de traductions, aujourd'hui, cette semaine
- [ ] Given les stats sont stockées localement, when Thoth redémarre, alors les compteurs persistent
- [ ] Given le menu "Statistiques", when l'utilisateur choisit "Réinitialiser", alors tous les compteurs sont remis à zéro

---

### Technical Notes

**Fichiers**: `src/metrics.rs`

**Stockage**: Fichier JSON dans `%APPDATA%/thoth/metrics.json`

```rust
#[derive(Serialize, Deserialize, Default)]
struct UsageMetrics {
    total_translations: u64,
    total_errors: u64,
    total_bytes_processed: u64,
    total_latency_ms: u64,           // Somme pour calculer la moyenne
    today_translations: u64,
    today_date: NaiveDate,
    week_translations: u64,
    week_start: NaiveDate,
    last_used: Option<NaiveDateTime>,
    model_usage: HashMap<String, u64>,  // gemma4:12b → 42, gemini4:12b → 7
}
```

**Affichage**: Fenêtre simple (ou notification avec les stats). Version MVP : notification toast avec les chiffres du jour.

**Latence moyenne**: `total_latency_ms / total_translations`

---

### Definition of Done

- [ ] Métriques persistées dans `%APPDATA%/thoth/metrics.json`
- [ ] Compteurs : total, aujourd'hui, cette semaine
- [ ] Latence moyenne
- [ ] Option "Statistiques" dans le menu tray
- [ ] `cargo clippy` et `cargo test` passent

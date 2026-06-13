## Story: CI/CD Pipeline GitHub Actions

**ID**: S1-THOTH-08
**Épic**: EPIC-THOTH-01
**Points**: 3
**Statut**: DONE

---

### User Story

**As a** Platform Engineer
**I want** une pipeline CI/CD complète qui analyse, teste et livre le binaire Thoth
**So that** chaque merge sur `main` produise un artifact testé et les tags `v*` créent une release GitHub prête à télécharger

---

### Acceptance Criteria

- [ ] Given un push sur `main`, when la pipeline s'exécute, alors :
  - `cargo fmt --all --check` vérifie le formatage
  - `cargo clippy -- -D warnings` analyse le code
  - `cargo test --workspace` exécute tous les tests
  - Le binaire release est compilé et uploadé comme artifact
- [ ] Given un tag `v*` est poussé, when la pipeline s'exécute, alors une GitHub Release est créée avec le `.exe` et son SHA256
- [ ] Given des dépendances obsolètes, when `cargo outdated` est exécuté, alors un warning est émis (sans bloquer)
- [ ] Given des vulnérabilités de sécurité, when `cargo audit` est exécuté, alors un rapport est produit (sans bloquer)
- [ ] Given des dépendances inutilisées, when `cargo udeps` est exécuté, alors un rapport est produit (sans bloquer)

---

### Technical Notes

**Fichier**: `.github/workflows/ci.yml`

**Jobs**:

| Job | Runner | Déclencheur | Outil |
|---|---|---|---|
| `quality` | `windows-latest` | PR + push | `cargo fmt`, `cargo clippy`, `cargo outdated`, `cargo audit`, `cargo udeps` |
| `test` | `windows-latest` | PR + push | `cargo test --workspace` |
| `build` | `windows-latest` | push main + tags | `cargo build --release` avec LTO, strip |
| `release` | `windows-latest` | tags `v*` | `softprops/action-gh-release` |

**Outils d'analyse**:
- `cargo-outdated` — détecte les crates obsolètes
- `cargo-audit` — vérifie les vulnérabilités (base de données RustSec)
- `cargo-udeps` — trouve les dépendances inutilisées
- `dtolnay/rust-toolchain` — installation de Rust
- `Swatinem/rust-cache` — cache des dépendances Cargo

**Build optimisé**:
```toml
[profile.release]
opt-level = "z"     # Taille minimale
lto = true
codegen-units = 1
strip = "symbols"
```

**Release assets**:
- `thoth-windows-x86_64.zip` — archive complète
- `thoth.exe` — binaire standalone
- `thoth.exe.sha256` — checksum de vérification

---

### Definition of Done

- [ ] La pipeline complète s'exécute sans erreur sur un runner Windows
- [ ] Les checks qualité (fmt, clippy) sont bloquants sur PR
- [ ] Les checks obsolescence/sécurité sont informatifs (non bloquants)
- [ ] Le build release produit un artifact téléchargeable
- [ ] La release GitHub est automatique sur tag `v*`
- [ ] `cargo clippy` et `cargo test` passent

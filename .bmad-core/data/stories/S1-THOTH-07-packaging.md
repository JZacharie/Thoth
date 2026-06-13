## Story: Packaging Windows

**ID**: S1-THOTH-07
**Épic**: EPIC-THOTH-01
**Points**: 2
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** installer Thoth facilement sur Windows
**So that** je n'aie pas besoin de compiler le projet moi-même

---

### Acceptance Criteria

- [ ] Given un build release, when `cargo build --release` est exécuté, alors un `.exe` est produit sans dépendances externes
- [ ] Given le `.exe` est copié sur un autre Windows 10/11, when il est exécuté, alors Thoth fonctionne sans installation supplémentaire
- [ ] Given un package MSI ou un script, when l'installateur est exécuté, alors Thoth est ajouté au démarrage automatique (optionnel)

---

### Technical Notes

**Build**:
```bash
cargo build --release --target x86_64-pc-windows-msvc
```

**Target binary**: `target/release/thoth.exe`

**Démarrage automatique** (optionnel):
- Ajouter une entrée dans `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- Créer un script PowerShell ou un petit installateur

**CI/CD**:
- GitHub Actions avec runner Windows
- `actions-rs/toolchain` pour installer Rust
- Uploader l'artefact `.exe` sur les releases GitHub

**Dépendances statiques**:
- `cargo build --release` produit un `.exe` standalone avec Tokio, reqwest, etc. liés statiquement
- Aucun runtime VC++ requis si target `x86_64-pc-windows-msvc` (lié à la CRT universelle Windows)

---

### Definition of Done

- [ ] `cargo build --release` produit `thoth.exe`
- [ ] Le binaire fonctionne sur Windows 10/11 sans dépendances
- [ ] La taille du binaire est optimisée (< 10MB)
- [ ] `cargo clippy` et `cargo test` passent

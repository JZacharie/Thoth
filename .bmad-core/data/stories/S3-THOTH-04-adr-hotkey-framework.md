## Story: ADR - Choix du Framework de Hotkey

**ID**: S3-THOTH-04
**Épic**: EPIC-THOTH-03
**Points**: 1
**Statut**: DONE

---

### User Story

**As a** Architecte
**I want** une décision architecturale documentée sur le choix du framework de hotkey
**So that** l'équipe et les contributeurs comprennent pourquoi `rdev` a été choisi plutôt que `inputbot`

---

### Acceptance Criteria

- [ ] Given l'ADR est rédigé, when un nouveau développeur lit le dossier `adrs/`, alors il comprend le contexte et la décision
- [ ] Given l'ADR contient les alternatives évaluées, when une nouvelle option apparaît (ex: `windows-rs` hook), alors l'ADR facilite la réévaluation
- [ ] Given l'ADR est approuvé, when l'implémentation commence, alors le choix est verrouillé

---

### Technical Notes

**Fichier**: `.bmad-core/data/adrs/adr-003-hotkey-framework.md`

**Structure** (suivant le template ADR):
- Titre : "Choix du framework de global hotkey"
- Contexte : besoin d'intercepter Win+N sous Windows
- Options : `rdev` vs `inputbot` vs `windows-rs` (SetWindowsHookEx direct)
- Décision : `rdev`
- Conséquences : ... 
- Statut : Accepté

---

### Definition of Done

- [ ] ADR rédigé et approuvé
- [ ] Liens vers les issues/discussions pertinentes
- [ ] `cargo clippy` et `cargo test` passent (S/O)

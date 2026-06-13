## Story: Configuration de la Langue Cible

**ID**: S2-THOTH-02
**Épic**: EPIC-THOTH-02
**Points**: 2
**Statut**: DONE

---

### User Story

**As a** Utilisateur
**I want** choisir la langue dans laquelle le texte doit être traduit
**So that** je puisse traduire vers l'anglais, l'espagnol, l'allemand, etc.

---

### Acceptance Criteria

- [ ] Given la config contient `target_language = "en"`, when le texte est envoyé, alors le prompt système inclut "Traduis en anglais"
- [ ] Given la config contient `target_language = "es"`, when le texte est envoyé, alors le prompt système inclut "Traduis en espagnol"
- [ ] Given la langue cible est "fr" (défaut), when le texte est envoyé, alors le comportement est identique à avant (traduction/correction vers le français)
- [ ] Given une langue non supportée est configurée, when Thoth charge la config, alors il logge un avertissement et utilise "fr" par défaut

---

### Technical Notes

**Fichier**: `src/config.rs` (champ `target_language`)

**Prompt dynamique** : Le prompt système n'est plus statique mais inclut la langue cible.

```
Tu es un traducteur et correcteur de texte ultra-précis.
Ta tâche est de traduire, corriger l'orthographe/grammaire et rendre le texte fourni clair et concis.
Traduis le texte suivant en {target_language}.
Tu dois UNIQUEMENT retourner le texte corrigé et traduit...
```

**Langues supportées** (ISO 639-1) :
| Code | Langue |
|---|---|
| `fr` | Français |
| `en` | Anglais |
| `es` | Espagnol |
| `de` | Allemand |
| `it` | Italien |
| `pt` | Portugais |
| `nl` | Néerlandais |
| `ja` | Japonais |
| `zh` | Chinois |
| `ru` | Russe |

**Menu tray**: Sous-menu "Langue cible" avec les options disponibles.

---

### Definition of Done

- [ ] Champ `target_language` dans la config
- [ ] Prompt système dynamique selon la langue
- [ ] Validation des codes langue supportés
- [ ] Option dans le menu contextuel de la system tray
- [ ] `cargo clippy` et `cargo test` passent

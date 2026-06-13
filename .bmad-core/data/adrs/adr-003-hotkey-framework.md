# ADR-003: Choix du Framework de Global Hotkey

**Statut**: Accepté
**Date**: 2026-06-13
**Décideur**: Larry (Architect)

---

## Contexte

Thoth doit intercepter un raccourci clavier global (`Win + N`) sous Windows.
Ce raccourci doit fonctionner depuis n'importe quelle application, sans conflit
avec les hotkeys existantes. Le framework choisi doit permettre :

- L'écoute des événements clavier globaux
- Le filtrage de combinaisons spécifiques (modifieurs + touche)
- La simulation de touches (Ctrl+C, Ctrl+V) pour les opérations presse-papier
- Le désenregistrement/ré-enregistrement dynamique (hotkey configurable)

## Options

### Option A : `rdev` (choisi)

| Critère | Évaluation |
|---|---|
| API | `listen(callback)` — reçoit tous les événements clavier |
| Simulation | `simulate(EventType)` pour Ctrl+C/V |
| Multi-OS | Windows, macOS, Linux (stubs Linux dans Thoth) |
| Hotkey dynamique | Possible via mise à jour du pattern dans le callback |
| Maintenance | Crate maintenue, 2k+ étoiles GitHub |
| Sécurité | Aucun log des frappes — le callback est controlé par notre code |
| Dépendances | Minimes : `x11` sur Linux, `winapi` sur Windows |

**Avantages** :
- API simple et éprouvée
- Une seule crate pour l'écoute ET la simulation (pas besoin de `enigo` en plus)
- Hotkey configurable sans redémarrer (le callback lit un `Mutex<HotkeyPattern>`)

**Inconvénients** :
- Sur Linux, nécessite `libx11-dev` (résolu par `cfg(not(windows))` stubs)
- Reçoit TOUS les événements — nécessite un filtrage strict dans notre code

### Option B : `inputbot`

| Critère | Évaluation |
|---|---|
| API | `bind(hotkey, callback)` — plus haut niveau |
| Simulation | Non inclus — nécessite `enigo` ou `rdev` en plus |
| Multi-OS | Windows, macOS, Linux |
| Hotkey dynamique | `unbind()` disponible |
| Maintenance | Crate maintenue |

**Avantages** :
- API plus simple pour binder un hotkey spécifique
- Pas besoin de filtrer manuellement les événements

**Inconvénients** :
- Nécessite une seconde crate (`enigo`) pour la simulation clavier
- `unbind` n'est pas aussi fiable que notre approche `Mutex<Pattern>`
- Communauté plus petite

### Option C : `windows-rs` (Win32 SetWindowsHookEx)

| Critère | Évaluation |
|---|---|
| API | Appels Win32 directs (`SetWindowsHookEx`, `SendInput`) |
| Simulation | `SendInput` pour Ctrl+C/V |
| Multi-OS | Windows uniquement |
| Hotkey dynamique | `UnhookWindowsHookEx` / `RegisterHotKey` |
| Maintenance | SDK Microsoft |

**Avantages** :
- Contrôle total, pas de dépendance tierce
- `RegisterHotKey` est le mécanisme officiel Windows pour les hotkeys globaux

**Inconvénients** :
- Beaucoup de boilerplate (message pump, window procedure)
- Ne fonctionne que sur Windows — pas de test possible sur Linux
- ~300 lignes de code Win32 complexe vs ~50 lignes avec `rdev`

## Décision

**Nous choisissons `rdev`** pour les raisons suivantes :

1. **Une seule dépendance** pour l'écoute ET la simulation — `inputbot` nécessite `enigo` en plus
2. **API simple** — `listen(callback)` vs `SetWindowsHookEx` + message pump
3. **Notre pattern `Mutex<HotkeyPattern>`** permet un hotkey dynamique sans redémarrer
4. **Stubs Linux** — le code compile sur toutes les plateformes via `cfg(not(windows))`, même si le hotkey ne fonctionne que sur Windows
5. **Sécurité** — nous contrôlons le callback, aucun événement n'est loggé ou persistant

## Conséquences

**Positives** :
- Pas de seconde crate pour la simulation
- Code simple et maintenable
- Hotkey dynamique via config hot-reload
- Tests unitaires possibles sur la couche de parsing (`HotkeyPattern`)

**Négatives** :
- `rdev` n'est pas disponible sur tous les systèmes de CI non-Windows
- Impossible de tester le hotkey réel sur Linux
- Filtrage manuel des événements — doit être audité pour la sécurité

## Références

- [rdev](https://crates.io/crates/rdev) — crates.io
- [inputbot](https://crates.io/crates/inputbot) — crates.io
- [windows-rs](https://crates.io/crates/windows) — crates.io
- ADR-002: Architecture de Thoth
- ADR-004: Sécurité (section hotkey)

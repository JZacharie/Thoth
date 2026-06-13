# ADR-004: Sécurité et Analyse des Menaces pour Thoth

**Statut**: Accepté
**Date**: 2026-06-13
**Décideur**: Larry (Architect)

---

## Contexte

Thoth intercepte les frappes clavier, lit/écrit le presse-papier et envoie du texte
utilisateur à un LLM via Pylos. Ces opérations présentent des risques de sécurité
spécifiques (keylogging, fuite de données, injection).

## Décisions

### 1. Aucun log de frappes clavier

L'écoute globale (`rdev`) ne doit **jamais** logger le contenu des touches.
Seul l'événement "hotkey déclenché" est loggé.

```rust
// ✅ CORRECT
tracing::debug!("hotkey triggered");

// ❌ INTERDIT
tracing::debug!("key pressed: {:?}", event);
```

### 2. Header secret X-Thoth-Secret

Toute requête vers Pylos doit inclure un header secret partagé pour éviter
qu'un processus malveillant sur le loopback ne puisse usurper Pylos.

```rust
client.post(endpoint)
    .header("X-Thoth-Secret", config.secret)
    .json(&request)
    .send()
```

**Raison**: Un processus malveillant pourrait écouter sur `localhost:3000` et
recevoir le texte utilisateur. Le header secret ajoute une couche d'authentification
minimale sur le loopback.

**Stockage**: Le secret est configurable dans `config.toml`. Si non défini,
Thoth génère un UUID aléatoire au premier lancement et le persiste.

### 3. Debounce anti-DoS

Un délai de **500ms** minimum est imposé entre deux déclenchements du hotkey
pour éviter :
- Les boucles accidentelles (touche collée)
- La saturation de Pylos
- La consommation involontaire de tokens LLM

```rust
let last_trigger = Instant::now();
const DEBOUNCE_MS: u64 = 500;

// Dans la boucle d'écoute
if last_trigger.elapsed() < Duration::from_millis(DEBOUNCE_MS) {
    continue; // ignorer
}
last_trigger = Instant::now();
```

### 4. Filtre heuristique de données sensibles

Avant d'envoyer le texte à Pylos, Thoth vérifie si le contenu ressemble
à des secrets connus et ignore silencieusement si c'est le cas.

Patterns exclus :
- Clés API : `sk-[a-zA-Z0-9]{20,}`, `pk-[a-zA-Z0-9]{20,}`
- Tokens JWT : `eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+`
- Clés SSH : `-----BEGIN (RSA|EC|OPENSSH) PRIVATE KEY-----`
- Cartes bancaires : `\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}`
- Mots de passe dans des champs type `password=`

```rust
fn contains_secrets(text: &str) -> bool {
    let patterns = [
        r"sk-[a-zA-Z0-9]{20,}",
        r"eyJ[a-zA-Z0-9_-]+\.eyJ",
        r"-----BEGIN.*PRIVATE KEY-----",
        r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b",
    ];
    patterns.iter().any(|p| regex::Regex::new(p).unwrap().is_match(text))
}
```

**Note**: Ce filtre est une protection de base uniquement. L'utilisateur reste
responsable de ne pas utiliser Thoth sur des champs sensibles.

### 5. TLS pour Pylos

Bien que Pylos tourne sur loopback, le support TLS dès que Pylos le permettra.
À court terme, le header secret compense l'absence de chiffrement.

### 6. Signature Authenticode

Le binaire doit être signé pour :
- Éviter les alertes Windows SmartScreen
- Garantir l'intégrité du binaire (protection tampering)
- Permettre aux politiques d'entreprise de whitelister Thoth

## Conséquences

**Positives**:
- Le risque keylogging est éliminé par conception (pas de log de touches)
- Le header secret protège contre les écoutes loopback
- Le debounce empêche les boucles accidentelles coûteuses
- Le filtre protège des fuites les plus courantes
- La signature Authenticode établit la confiance

**Négatives**:
- Le filtre heuristique n'est pas infaillible (faux négatifs possibles)
- Le header secret est partagé en clair dans config.toml (mais loopback uniquement)
- La signature nécessite un certificat payant (~200-300€/an)

## Références

- OWASP Top 10 — A02:2021 Cryptographic Failures
- OWASP Top 10 — A05:2021 Security Misconfiguration
- Microsoft Security Development Lifecycle (SDL)
- STRIDE threat model
- `.bmad-core/data/epics/epics-thoth-v2.md` (S2-THOTH-05 signing)

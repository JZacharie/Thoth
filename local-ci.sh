#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# local-ci.sh
# Exécute les mêmes checks que la pipeline GitHub Actions,
# mais en local pour un feedback plus rapide.
#
# Usage :
#   chmod +x local-ci.sh
#   ./local-ci.sh
#
# Étapes :
#   1. cargo fmt --check
#   2. cargo clippy --workspace --all-targets
#   3. cargo test --workspace --all-features
#   4. cargo build --release (vérifie que la compilation release passe)
#   5. cargo outdated (si installé)
#   6. cargo audit    (si installé)
#   7. cargo udeps    (si installé)
# ============================================================

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'
PASS=0
FAIL=0

check() {
    local name="$1"
    shift
    echo -e "${YELLOW}━━━ [$name] ━━━${NC}"
    if "$@" 2>&1; then
        echo -e "${GREEN}✓ $name passed${NC}"
        PASS=$((PASS + 1))
    else
        echo -e "${RED}✗ $name failed${NC}"
        FAIL=$((FAIL + 1))
    fi
    echo
}

echo -e "${YELLOW}══════════════════════════════════════${NC}"
echo -e "${YELLOW}  Local CI — Thoth${NC}"
echo -e "${YELLOW}  $(date)${NC}"
echo -e "${YELLOW}══════════════════════════════════════${NC}"
echo

# --- Obligatoires (bloquants dans la pipeline) ---

check "cargo fmt" cargo fmt --all --check
check "cargo clippy" cargo clippy --workspace --all-targets -- -D warnings

if command -v cargo-nextest &>/dev/null; then
    check "cargo nextest" cargo nextest run --workspace --no-fail-fast
else
    check "cargo test" cargo test --workspace --all-features
fi

check "cargo build --release" cargo build --release

# --- Outils de la CI (exécutés si installés en local) ---

if command -v actionlint &>/dev/null; then
    check "actionlint" actionlint
else
    echo -e "${YELLOW}⚠ actionlint non installé — saute (go install github.com/rhysd/actionlint/cmd/actionlint@latest)${NC}"
fi

if command -v typos &>/dev/null; then
    check "typos" typos
else
    echo -e "${YELLOW}⚠ typos non installé — saute (cargo install typos-cli)${NC}"
fi

if command -v cargo-deny &>/dev/null; then
    check "cargo deny" cargo deny check
else
    echo -e "${YELLOW}⚠ cargo-deny non installé — saute (cargo install --locked cargo-deny)${NC}"
fi

# --- Autres outils locaux ---

if command -v cargo-outdated &>/dev/null; then
    check "cargo outdated" cargo outdated --exit-code 1
else
    echo -e "${YELLOW}⚠ cargo-outdated non installé — saute (cargo install cargo-outdated)${NC}"
fi

if command -v cargo-audit &>/dev/null; then
    check "cargo audit" cargo audit
else
    echo -e "${YELLOW}⚠ cargo-audit non installé — saute (cargo install cargo-audit)${NC}"
fi

if command -v cargo-udeps &>/dev/null; then
    check "cargo udeps" cargo +nightly udeps --all-targets
else
    echo -e "${YELLOW}⚠ cargo-udeps non installé — saute (cargo install cargo-udeps)${NC}"
fi

# --- Résumé ---

echo -e "${YELLOW}══════════════════════════════════════${NC}"
echo -e "  Résumé : ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}"
echo -e "${YELLOW}══════════════════════════════════════${NC}"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi

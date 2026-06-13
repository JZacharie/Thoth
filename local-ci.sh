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
check "cargo clippy" cargo clippy --workspace --all-targets
check "cargo test" cargo test --workspace --all-features
check "cargo build --release" cargo build --release

# --- Facultatifs (non bloquants dans la pipeline) ---

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

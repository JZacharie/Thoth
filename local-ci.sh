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

# Vérifie les bibliothèques système requises sur Linux (xcb, xi, gtk, xkbcommon…)
LINUX_DEPS_OK=true
if [[ "$(uname -s)" == "Linux" ]]; then
    MISSING_PKGS=()
    for lib in xcb xi gtk+-3.0 xkbcommon xtst; do
        if ! pkg-config --exists "$lib" 2>/dev/null; then
            MISSING_PKGS+=("$lib")
            LINUX_DEPS_OK=false
        fi
    done
    # libxdo n'a pas de fichier .pc — vérification via ldconfig
    if ! ldconfig -p 2>/dev/null | grep -q "libxdo\.so"; then
        MISSING_PKGS+=("libxdo")
        LINUX_DEPS_OK=false
    fi
    if [ "$LINUX_DEPS_OK" = false ]; then
        echo -e "${RED}✗ Dépendances système Linux manquantes : ${MISSING_PKGS[*]}${NC}"
        echo -e "${YELLOW}  → Installe-les avec :${NC}"
        echo -e "${YELLOW}    sudo apt-get install -y \\${NC}"
        echo -e "${YELLOW}      libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \\${NC}"
        echo -e "${YELLOW}      libxcb1-dev libx11-dev libxi-dev libxtst-dev libxdo-dev \\${NC}"
        echo -e "${YELLOW}      libxkbcommon-dev \\${NC}"
        echo -e "${YELLOW}      libgtk-3-dev libatk1.0-dev libcairo2-dev libglib2.0-dev libpango1.0-dev \\${NC}"
        echo -e "${YELLOW}      libssl-dev pkg-config${NC}"
        echo -e "${YELLOW}  → Tests+build sautés. Exécute cargo check comme substitut.${NC}"
        check "cargo check (lib, no-link)" cargo check --lib
        check "cargo check (release equivalent)" cargo check --release
    fi
fi

if [ "$LINUX_DEPS_OK" = true ]; then
    if command -v cargo-nextest &>/dev/null; then
        check "cargo nextest" cargo nextest run --workspace --no-fail-fast
    else
        check "cargo test" cargo test --workspace --all-features
    fi
    check "cargo build --release" cargo build --release
fi


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
    check "cargo outdated" cargo outdated
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

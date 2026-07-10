#!/usr/bin/env bash
set -euo pipefail

# ============================================================
# local-ci.sh
# Miroir de la CI GitHub, adapté à la plateforme hôte.
# Usage : ./local-ci.sh
#
# Phases (calquées sur .github/workflows/ci.yml) :
#   1. precheck  → actionlint + fmt
#   2. check     → clippy + tests (+ deny si installé)
#   3. build     → cargo build --release
#   4. extras    → outdated / audit / udeps (si installés)
# ============================================================

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'
PASS=0
FAIL=0

OS="$(uname -s)"

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
echo -e "${YELLOW}  Local CI — Thoth (${OS})${NC}"
echo -e "${YELLOW}  $(date)${NC}"
echo -e "${YELLOW}══════════════════════════════════════${NC}"
echo

# ── Phase 1 : Precheck ────────────────────────────────────

if command -v actionlint &>/dev/null; then
    check "actionlint" actionlint
else
    echo -e "${YELLOW}⚠ actionlint non installé — saute (go install github.com/rhysd/actionlint/cmd/actionlint@latest)${NC}"
fi

if ! cargo fmt --all --check &>/dev/null; then
    echo -e "${YELLOW}⚠ Formatage incorrect détecté. Correction par cargo fmt...${NC}"
    cargo fmt --all
fi
check "cargo fmt" cargo fmt --all --check

# ── Phase 2 : Check (clippy + tests) ──────────────────────

check "cargo clippy" cargo clippy --workspace --all-targets -- -D warnings

# Vérifie les bibliothèques système requises sur Linux
LINUX_DEPS_OK=true
if [[ "$OS" == "Linux" ]]; then
    MISSING_PKGS=()
    for lib in xcb xi gtk+-3.0 xkbcommon xtst; do
        if ! pkg-config --exists "$lib" 2>/dev/null; then
            MISSING_PKGS+=("$lib")
            LINUX_DEPS_OK=false
        fi
    done
    if ! grep -q "libxdo\.so" <<< "$(ldconfig -p 2>/dev/null)"; then
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
fi

# ── Phase 3 : Build release (fast) ─────────────────────────
# Miroir du job build-windows-fast / la phase release de la CI

if [ "$LINUX_DEPS_OK" = true ]; then
    check "cargo build --release" cargo build --release
fi

# deny : miroir de la CI (exécuté sur push)
if command -v cargo-deny &>/dev/null; then
    check "cargo deny" cargo deny check
fi

# ── Phase 4 : Extras (hors CI, optionnels) ─────────────────

if command -v typos &>/dev/null; then
    check "typos" typos
fi

if command -v cargo-outdated &>/dev/null; then
    check "cargo outdated" cargo outdated
fi

if command -v cargo-audit &>/dev/null; then
    check "cargo audit" cargo audit
fi

if command -v cargo-udeps &>/dev/null; then
    check "cargo udeps" cargo +nightly udeps --all-targets
fi

# ── Résumé ─────────────────────────────────────────────────

echo -e "${YELLOW}══════════════════════════════════════${NC}"
echo -e "  Résumé : ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}"
echo -e "${YELLOW}══════════════════════════════════════${NC}"

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi

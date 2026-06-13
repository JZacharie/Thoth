# ============================================================
# local-ci.ps1
# Exececute les memes checks que la pipeline GitHub Actions,
# mais en local pour un feedback plus rapide (sous Windows/PowerShell).
#
# Usage :
#   .\local-ci.ps1
#   .\local-ci.ps1 -Release
# ============================================================

Param(
    [switch]$Release
)

# Configuration de la console
$Host.UI.RawUI.WindowTitle = "Local CI - Thoth"

$PASS = 0
$FAIL = 0

function Write-Border {
    Write-Host "=======================================" -ForegroundColor Yellow
}

function Write-Header {
    param($title)
    Write-Host "--- [$title] ---" -ForegroundColor Yellow
}

function Check-Command {
    param($Name, $ScriptBlock)
    Write-Header $Name
    
    # Execute le script block et laisse la sortie s'afficher en temps reel
    & $ScriptBlock
    
    # Si la commande est externe, on verifie $LASTEXITCODE. Sinon on verifie $?
    $status = $false
    if ($LASTEXITCODE -ne $null) {
        if ($LASTEXITCODE -eq 0) { $status = $true }
    } else {
        if ($?) { $status = $true }
    }

    if ($status) {
        Write-Host "[OK] $Name passed`n" -ForegroundColor Green
        $global:PASS++
    } else {
        Write-Host "[FAIL] $Name failed`n" -ForegroundColor Red
        $global:FAIL++
    }
    # Reset exit code pour la prochaine commande
    $global:LASTEXITCODE = $null
}

Write-Border
Write-Host "  Local CI - Thoth (Windows)" -ForegroundColor Yellow
Write-Host "  $(Get-Date)" -ForegroundColor Yellow
Write-Border
Write-Host ""

# Arrete toutes les instances en cours de thoth.exe pour liberer le verrou sur le binaire
Write-Host "Arrêt de toutes les instances de thoth.exe..." -ForegroundColor Yellow
taskkill /F /IM thoth.exe 2>$null
Start-Sleep -Seconds 1

# --- Obligatoires (bloquants dans la pipeline) ---

Check-Command "cargo fmt" { cargo fmt --all --check }
Check-Command "cargo clippy" { cargo clippy --workspace --all-targets -- -D warnings }

if (Get-Command cargo-nextest -ErrorAction SilentlyContinue) {
    Check-Command "cargo nextest" { cargo nextest run --workspace --no-fail-fast }
} else {
    Check-Command "cargo test" { cargo test --workspace --all-features }
}

# Compilation du binaire (Debug par défaut pour tester plus rapidement, ou Release)
if ($Release) {
    Check-Command "cargo build --release" { cargo build --release }
    $binaryPath = "target\release\thoth.exe"
} else {
    Check-Command "cargo build" { cargo build }
    $binaryPath = "target\debug\thoth.exe"
}

# --- Outils de la CI (executes si installes en local) ---

if (Get-Command actionlint -ErrorAction SilentlyContinue) {
    Check-Command "actionlint" { actionlint }
} else {
    Write-Host "[WARN] actionlint non installe - saute (go install github.com/rhysd/actionlint/cmd/actionlint@latest)`n" -ForegroundColor Yellow
}

if (Get-Command typos -ErrorAction SilentlyContinue) {
    Check-Command "typos" { typos }
} else {
    Write-Host "[WARN] typos non installe - saute (cargo install typos-cli)`n" -ForegroundColor Yellow
}

if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
    Check-Command "cargo deny" { cargo deny check }
} else {
    Write-Host "[WARN] cargo-deny non installe - saute (cargo install --locked cargo-deny)`n" -ForegroundColor Yellow
}

# --- Autres outils locaux ---

if (Get-Command cargo-outdated -ErrorAction SilentlyContinue) {
    Check-Command "cargo outdated" { cargo outdated }
} else {
    Write-Host "[WARN] cargo-outdated non installe - saute (cargo install cargo-outdated)`n" -ForegroundColor Yellow
}

if (Get-Command cargo-audit -ErrorAction SilentlyContinue) {
    Check-Command "cargo audit" { cargo audit }
} else {
    Write-Host "[WARN] cargo-audit non installe - saute (cargo install cargo-audit)`n" -ForegroundColor Yellow
}

if (Get-Command cargo-udeps -ErrorAction SilentlyContinue) {
    Check-Command "cargo udeps" { cargo +nightly udeps --all-targets }
} else {
    Write-Host "[WARN] cargo-udeps non installe - saute (cargo install cargo-udeps)`n" -ForegroundColor Yellow
}

# --- Resume ---

Write-Border
Write-Host "  Resume : " -NoNewline -ForegroundColor Yellow
Write-Host "$PASS passed" -NoNewline -ForegroundColor Green
Write-Host ", " -NoNewline
Write-Host "$FAIL failed" -ForegroundColor Red
if (Test-Path $binaryPath) {
    $resolvedPath = Resolve-Path $binaryPath
    Write-Host "  Binaire disponible ici : " -NoNewline -ForegroundColor Yellow
    Write-Host $resolvedPath.Path -ForegroundColor Cyan
}
Write-Border

if ($FAIL -gt 0) {
    exit 1
}

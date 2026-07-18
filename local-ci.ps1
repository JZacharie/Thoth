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
        $script:PASS++
    } else {
        Write-Host "[FAIL] $Name failed`n" -ForegroundColor Red
        $script:FAIL++
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

# Signature du binaire avec un certificat auto-signé pour passer la validation de signature au démarrage
if (Test-Path $binaryPath) {
    Write-Host "Signature du binaire avec un certificat auto-signé..." -ForegroundColor Yellow
    $cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq "CN=ThothCodeSign" } | Select-Object -First 1
    if (-not $cert) {
        $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=ThothCodeSign" -CertStoreLocation Cert:\CurrentUser\My
        $certBytes = $cert.Export([System.Security.Cryptography.X509Certificates.X509ContentType]::Cert)
        $tempCertPath = [System.IO.Path]::GetTempFileName()
        [System.IO.File]::WriteAllBytes($tempCertPath, $certBytes)
        # Import dans Trusted Root pour valider la confiance de la signature
        Import-Certificate -FilePath $tempCertPath -CertStoreLocation Cert:\CurrentUser\Root *>$null
        Remove-Item $tempCertPath
    }
    Set-AuthenticodeSignature -FilePath $binaryPath -Certificate $cert | Out-Null
    Write-Host "[OK] Signature de thoth.exe effectuée`n" -ForegroundColor Green
}

# --- Outils de la CI (executes et installes si manquants) ---

if (-not (Get-Command actionlint -ErrorAction SilentlyContinue)) {
    Write-Host "actionlint non installé. Installation via go install..." -ForegroundColor Yellow
    go install github.com/rhysd/actionlint/cmd/actionlint@latest
}
if (Get-Command actionlint -ErrorAction SilentlyContinue) {
    Check-Command "actionlint" { actionlint }
} else {
    Write-Host "[FAIL] Impossible d'installer ou de lancer actionlint`n" -ForegroundColor Red
}

if (-not (Get-Command cargo-deny -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-deny non installé. Installation via cargo install..." -ForegroundColor Yellow
    cargo install --locked cargo-deny
}
if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
    Check-Command "cargo deny" { cargo deny check }
} else {
    Write-Host "[FAIL] Impossible d'installer ou de lancer cargo-deny`n" -ForegroundColor Red
}

# --- Autres outils locaux ---

if (-not (Get-Command cargo-outdated -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-outdated non installé. Installation via cargo install..." -ForegroundColor Yellow
    cargo install --locked cargo-outdated
}
if (Get-Command cargo-outdated -ErrorAction SilentlyContinue) {
    Check-Command "cargo outdated" { cargo outdated }
} else {
    Write-Host "[FAIL] Impossible d'installer ou de lancer cargo-outdated`n" -ForegroundColor Red
}

if (-not (Get-Command cargo-audit -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-audit non installé. Installation via cargo install..." -ForegroundColor Yellow
    cargo install --locked cargo-audit
}
if (Get-Command cargo-audit -ErrorAction SilentlyContinue) {
    Check-Command "cargo audit" { cargo audit }
} else {
    Write-Host "[FAIL] Impossible d'installer ou de lancer cargo-audit`n" -ForegroundColor Red
}

if (-not (Get-Command cargo-udeps -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-udeps non installé. Installation via cargo install..." -ForegroundColor Yellow
    cargo install --locked cargo-udeps
}
if (Get-Command cargo-udeps -ErrorAction SilentlyContinue) {
    if (-not (rustup toolchain list | Select-String "nightly")) {
        Write-Host "Toolchain nightly non installée. Installation via rustup..." -ForegroundColor Yellow
        rustup toolchain install nightly --profile minimal
    }
    Check-Command "cargo udeps" { cargo +nightly udeps --all-targets }
} else {
    Write-Host "[FAIL] Impossible d'installer ou de lancer cargo-udeps`n" -ForegroundColor Red
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

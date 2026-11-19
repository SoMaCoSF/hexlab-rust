# HexLab — Windows install & run (PowerShell)
# The one-shot Grok never delivered: clone, build, smoke-test, run.
# Usage:  irm https://raw.githubusercontent.com/SoMaCoSF/hexlab-rust/main/install.ps1 | iex
#    or:  .\install.ps1 [-Dir C:\src\hexlab-rust]
param(
    [string]$Dir = "$HOME\hexlab-rust"
)
$ErrorActionPreference = "Stop"

function Need($cmd, $hint) {
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
        Write-Host "MISSING: $cmd — $hint" -ForegroundColor Red
        return $false
    }
    return $true
}

$ok = $true
$ok = (Need "git" "Install Git for Windows: https://git-scm.com/download/win") -and $ok
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust not found — installing rustup…" -ForegroundColor Yellow
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install --id Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements
        $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
    } else {
        Write-Host "Install Rust from https://rustup.rs then re-run this script." -ForegroundColor Red
        exit 1
    }
}
if (-not $ok) { exit 1 }

if (Test-Path "$Dir\.git") {
    Write-Host "Updating $Dir"
    git -C $Dir pull --ff-only
} else {
    Write-Host "Cloning SoMaCoSF/hexlab-rust → $Dir"
    git clone https://github.com/SoMaCoSF/hexlab-rust.git $Dir
}
Set-Location $Dir

Write-Host "Building hexlab + hexlab-tui (release)…"
cargo build --release -p hexlab-cli -p hexlab-tui
$bin = "$Dir\target\release"
$env:Path = "$bin;$env:Path"

Write-Host "`n— Smoke test —" -ForegroundColor Cyan
& "$bin\hexlab.exe" bom woodshop | Select-Object -First 4
& "$bin\hexlab.exe" city preset recon -o "$env:TEMP\hexlab-city.json"
& "$bin\hexlab.exe" city telemetry "$env:TEMP\hexlab-city.json"
& "$bin\hexlab.exe" spoken "create a bom for a woodshop and stock it" --city "$env:TEMP\hexlab-city.json"

Write-Host "`nHexLab is built. Run it:" -ForegroundColor Green
Write-Host "  $bin\hexlab.exe bom woodshop"
Write-Host "  $bin\hexlab.exe city preset carrier -o city.json"
Write-Host "  $bin\hexlab-tui.exe --city city.json      # full-screen TUI"
Write-Host "Tauri desktop shell (needs WebView2, built into Windows 11):"
Write-Host "  cargo install tauri-cli --version '^2'; cd apps\hexlab-tauri; cargo tauri build"

#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

Write-Host "`n[1/4] Checking code formatting (rustfmt)..." -ForegroundColor Cyan
cargo fmt --all -- --check

Write-Host "`n[2/4] Running cargo check on all targets..." -ForegroundColor Cyan
cargo check --workspace --all-targets

Write-Host "`n[3/4] Running clippy (clean lints)..." -ForegroundColor Cyan
cargo clippy --workspace --all-targets -- -A clippy::pedantic -A clippy::type_complexity -D warnings

Write-Host "`n[4/4] Running cargo audit..." -ForegroundColor Cyan
if (Get-Command cargo-audit -ErrorAction SilentlyContinue) {
    cargo audit
} else {
    Write-Host "cargo-audit is not installed. To run security audit locally, install it via:" -ForegroundColor Yellow
    Write-Host "cargo install cargo-audit" -ForegroundColor White
}

Write-Host "`nAll checks completed successfully!" -ForegroundColor Green

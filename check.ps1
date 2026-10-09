#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"
Write-Host "Running cargo check..." -ForegroundColor Cyan
cargo check --workspace --all-targets
Write-Host "Running clippy..." -ForegroundColor Cyan
cargo clippy --workspace --all-targets -- -D warnings

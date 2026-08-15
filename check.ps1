# PowerShell Workspace Verification Script
Write-Host "Running cargo fmt check..."
cargo fmt --all -- --check

Write-Host "Running cargo clippy check..."
cargo clippy --workspace --all-targets -- -D warnings

Write-Host "Running cargo test..."
cargo test --workspace

Write-Host "Verification complete!"

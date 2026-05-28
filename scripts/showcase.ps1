# Run the NeoTUI MVP showcase.
# Usage: .\scripts\showcase.ps1

Write-Host "Inspecting NeoTUI runtime readiness..." -ForegroundColor Green
cargo run -p neotui-cli -- doctor

if ($LASTEXITCODE -ne 0) {
    Write-Host "Doctor failed; fix runtime readiness before recording the showcase." -ForegroundColor Red
    exit 1
}

Write-Host "Validating showcase DSL..." -ForegroundColor Green
cargo run -p neotui-cli -- check examples/visual-system-showcase.toml

if ($LASTEXITCODE -ne 0) {
    Write-Host "Showcase validation failed." -ForegroundColor Red
    exit 1
}

Write-Host "Starting showcase. Press Ctrl+Q to exit." -ForegroundColor Green
cargo run -p neotui-cli -- run examples/visual-system-showcase.toml

if ($LASTEXITCODE -ne 0) {
    Write-Host "Showcase run failed." -ForegroundColor Red
    exit 1
}

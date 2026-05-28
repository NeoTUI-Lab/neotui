# Benchmark script for NeoTUI core baselines
# Usage: .\scripts\bench.ps1

Write-Host "Running NeoTUI core baseline benchmarks..." -ForegroundColor Green

cargo test -p neotui-core --test benchmarks -- --ignored --nocapture

if ($LASTEXITCODE -eq 0) {
    Write-Host "Benchmarks completed!" -ForegroundColor Green
} else {
    Write-Host "Benchmarks failed!" -ForegroundColor Red
    exit 1
}

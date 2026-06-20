# Test script for the NeoTUI Python package
# Usage: .\scripts\test-python.ps1 [-Native]

param(
    [switch]$Native
)

$ErrorActionPreference = "Stop"

Write-Host "Running NeoTUI Python package tests..." -ForegroundColor Green

$uv = Get-Command uv -ErrorAction SilentlyContinue
if ($uv) {
    uv run --no-project --with pytest --with tomli python -m pytest -p no:cacheprovider python\neotui-py\tests
} else {
    $python = Get-Command python -ErrorAction SilentlyContinue
    if (-not $python) {
        Write-Host "Python test runner requires uv or python on PATH." -ForegroundColor Red
        exit 1
    }
    $env:PYTHONPATH = "python\neotui-py\src"
    python -m pytest -p no:cacheprovider python\neotui-py\tests
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "Python tests failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Checking Python form intent example JSON serialization..." -ForegroundColor Green
$previousPythonPath = $env:PYTHONPATH
$env:PYTHONPATH = "python\neotui-py\src"
if ($uv) {
    $generatedJson = uv run --no-project --with tomli python examples\python\form_intent.py --json
} else {
    $generatedJson = python examples\python\form_intent.py --json
}
$env:PYTHONPATH = $previousPythonPath

if ($LASTEXITCODE -ne 0) {
    Write-Host "Python form intent example serialization failed!" -ForegroundColor Red
    exit 1
}

$contractJson = "examples\python\form-intent.json"
$generatedCanonical = (($generatedJson | Out-String) | ConvertFrom-Json) | ConvertTo-Json -Depth 100 -Compress
$contractCanonical = (Get-Content -LiteralPath $contractJson -Raw | ConvertFrom-Json) | ConvertTo-Json -Depth 100 -Compress
if ($generatedCanonical -ne $contractCanonical) {
    Write-Host "Python form intent JSON does not match $contractJson." -ForegroundColor Red
    exit 1
}

if ($Native) {
    $rustVersion = rustc -vV
    $rustHost = ($rustVersion | Where-Object { $_ -like "host:*" })
    if ($rustHost -like "*msvc*" -and -not (Get-Command link -ErrorAction SilentlyContinue)) {
        Write-Host "Native Python extension build requires MSVC link.exe for the active Rust host." -ForegroundColor Red
        Write-Host "Install Visual Studio Build Tools with the Visual C++ workload, or run this gate in WSL/Linux." -ForegroundColor Red
        exit 1
    }
    if ($rustHost -like "*windows-gnu*" -and -not (Get-Command dlltool -ErrorAction SilentlyContinue)) {
        Write-Host "Native Python extension build requires dlltool.exe for the active GNU Rust host." -ForegroundColor Red
        Write-Host "Install MinGW binutils that provide dlltool.exe, switch back to the MSVC toolchain with link.exe, or run this gate in WSL/Linux." -ForegroundColor Red
        exit 1
    }

    Write-Host "Building NeoTUI Python native extension with maturin..." -ForegroundColor Green
    if ($uv) {
        uv run --project python\neotui-py maturin develop
    } else {
        Push-Location python\neotui-py
        try {
            python -m maturin develop
        } finally {
            Pop-Location
        }
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Native Python extension build failed!" -ForegroundColor Red
        exit 1
    }

    Write-Host "Validating Python form intent app through neotui check..." -ForegroundColor Green
    cargo build -p neotui-cli
    if ($LASTEXITCODE -ne 0) {
        Write-Host "neotui-cli build failed!" -ForegroundColor Red
        exit 1
    }

    $neotuiBin = Join-Path $PWD "target\debug\neotui"
    $neotuiExe = "$neotuiBin.exe"
    if (Test-Path $neotuiExe) {
        $neotuiBin = $neotuiExe
    }

    $previousPythonPath = $env:PYTHONPATH
    $env:PYTHONPATH = "python\neotui-py\src"
    if ($uv) {
        uv run --no-project --with tomli python examples\python\form_intent.py --check-only --neotui-bin $neotuiBin
    } else {
        python examples\python\form_intent.py --check-only --neotui-bin $neotuiBin
    }
    $env:PYTHONPATH = $previousPythonPath

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Python form intent check failed!" -ForegroundColor Red
        exit 1
    }
}

Write-Host "Python package checks passed!" -ForegroundColor Green

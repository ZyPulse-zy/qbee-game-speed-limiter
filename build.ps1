$ErrorActionPreference = "Stop"

$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargo) {
    $cargoExe = $cargo.Source
} else {
    $cargoPath = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (Test-Path $cargoPath) {
        $cargoExe = $cargoPath
    }
}

if (-not $cargoExe) {
    throw "Cannot find cargo. Install Rust from https://rustup.rs/ first."
}

$target = "x86_64-pc-windows-gnu"
& $cargoExe build --release --target $target --bin download-limiter-monitor --bin download-limiter-config

if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Copy-Item "target\$target\release\download-limiter-monitor.exe" "download_limiter_monitor.exe" -Force
Copy-Item "target\$target\release\download-limiter-config.exe" "download_limiter_config.exe" -Force

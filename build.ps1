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
& $cargoExe build --release --target $target --bin qbee-limiter-monitor --bin qbee-limiter-config

if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Copy-Item "target\$target\release\qbee-limiter-monitor.exe" "qbee_limiter_monitor.exe" -Force
Copy-Item "target\$target\release\qbee-limiter-config.exe" "qbee_limiter_config.exe" -Force

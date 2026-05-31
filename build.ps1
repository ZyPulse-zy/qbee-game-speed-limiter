$ErrorActionPreference = "Stop"

$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    $cargoPath = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
    if (Test-Path $cargoPath) {
        $cargo = Get-Item $cargoPath
    }
}

if (-not $cargo) {
    throw "Cannot find cargo. Install Rust from https://rustup.rs/ first."
}

$target = "x86_64-pc-windows-gnu"
& $cargo.Source build --release --target $target

if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Copy-Item "target\$target\release\qbee-game-speed-limiter.exe" "qbee_game_speed_limiter.exe" -Force

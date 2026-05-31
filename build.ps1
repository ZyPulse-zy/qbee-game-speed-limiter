$ErrorActionPreference = "Stop"

$compiler = Get-Command g++ -ErrorAction SilentlyContinue
if (-not $compiler) {
    throw "Cannot find g++. Install MinGW-w64 or run the GitHub Actions workflow, which installs MSYS2/MinGW."
}

& $compiler.Source `
    -std=c++17 `
    -O2 `
    -municode `
    -mwindows `
    -static-libgcc `
    -static-libstdc++ `
    -o qbee_game_speed_limiter.exe `
    native\QbeeGameSpeedLimiter.cpp `
    -lwinhttp `
    -lcomctl32 `
    -lole32 `
    -lshell32

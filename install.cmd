@echo off
setlocal
title qbee Game Speed Limiter Installer
cd /d "%~dp0"

echo Installing qbee Game Speed Limiter shortcuts...
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1" -StartConfig
if errorlevel 1 (
  echo.
  echo Installation failed. Please make sure the zip was fully extracted first.
  pause
  exit /b 1
)

echo.
echo Done. The configuration window should open automatically.
pause

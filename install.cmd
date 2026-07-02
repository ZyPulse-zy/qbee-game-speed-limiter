@echo off
setlocal
title Download Client Game Speed Limiter Installer
cd /d "%~dp0"

echo Installing Download Client Game Speed Limiter shortcuts...
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

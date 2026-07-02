@echo off
setlocal
title Download Client Game Speed Limiter Uninstaller
cd /d "%~dp0"

echo Removing Download Client Game Speed Limiter shortcuts and startup entry...
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0uninstall.ps1"
if errorlevel 1 (
  echo.
  echo Uninstall cleanup failed. You can still delete this folder manually after closing the app.
  pause
  exit /b 1
)

echo.
echo Done. You can delete this folder if you no longer need the app.
pause

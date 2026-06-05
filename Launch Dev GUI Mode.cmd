@echo off
setlocal
cd /d "%~dp0"
set "KNIGHT_SHIFT_PRESENTATION_MODE=gui"

call pnpm tauri dev
if errorlevel 1 (
  echo.
  echo Dev launch failed.
  pause
  exit /b 1
)

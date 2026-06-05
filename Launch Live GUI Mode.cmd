@echo off
setlocal
cd /d "%~dp0"
set "KNIGHT_SHIFT_PRESENTATION_MODE=gui"
set "KNIGHT_SHIFT_EXE=%~dp0src-tauri\target\release\knight-shift.exe"

if not exist "%KNIGHT_SHIFT_EXE%" (
  echo Building Knight Shift release executable...
  call pnpm tauri build
  if errorlevel 1 (
    echo.
    echo Build failed.
    pause
    exit /b 1
  )
)

start "" "%KNIGHT_SHIFT_EXE%"

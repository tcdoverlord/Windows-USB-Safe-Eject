@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>nul
if errorlevel 1 (
  echo Rust toolchain not found.
  echo Install Rust from https://rustup.rs/ and reopen this terminal.
  pause
  exit /b 1
)
echo Building Safe USB Eject in release mode...
cargo build --release
if errorlevel 1 (
  echo.
  echo Build failed. Review the compiler output above.
  pause
  exit /b 1
)
if not exist "target\release\safe-usb-eject.exe" (
  echo Build completed but EXE was not found.
  pause
  exit /b 1
)
if not exist "dist" mkdir dist
copy /y "target\release\safe-usb-eject.exe" "dist\SafeUSB-Eject.exe" >nul
if not exist "dist\scripts" mkdir "dist\scripts"
copy /y "scripts\Safe-USB-Eject.ps1" "dist\scripts\Safe-USB-Eject.ps1" >nul
if not exist "dist\logs" mkdir "dist\logs"
echo.
echo Build complete: dist\SafeUSB-Eject.exe
echo Keep the scripts folder beside the EXE.
pause
endlocal

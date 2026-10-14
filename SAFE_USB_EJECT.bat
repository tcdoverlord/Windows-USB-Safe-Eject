@echo off
setlocal
cd /d "%~dp0"
if not exist "target\release\safe-usb-eject.exe" (
  echo Safe USB Eject has not been built yet.
  echo Run BUILD-EXE.bat first.
  pause
  exit /b 1
)
start "" "%~dp0target\release\safe-usb-eject.exe"
endlocal

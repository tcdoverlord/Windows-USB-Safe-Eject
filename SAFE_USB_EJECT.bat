@echo off
setlocal
cd /d "%~dp0"

:: Safe USB Eject launcher
:: Requires Windows PowerShell 5.1 or newer.

powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\Safe-USB-Eject.ps1"
set "EXITCODE=%ERRORLEVEL%"

echo.
if "%EXITCODE%"=="0" (
    echo [SUCCESS] The USB device was safely ejected.
) else (
    echo [NOTICE] The USB device was not ejected. Review the message above.
)

echo.
pause
exit /b %EXITCODE%

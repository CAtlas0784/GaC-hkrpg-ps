@echo off
cd /d "%~dp0"
echo ==============================================
echo   Building GaC hkrpg ps (Release Mode)
echo ==============================================

cargo build --release
if %errorlevel% neq 0 (
    echo.
    echo [ERROR] Build failed! Please check the error messages above.
    pause
    exit /b %errorlevel%
)

echo.
echo ==============================================
echo [SUCCESS] Build completed!
echo Executables built at:
echo   - target\release\gameserver.exe
echo   - target\release\sdkserver.exe
echo.
echo You can launch them anytime with 'run.bat'.
echo ==============================================
pause

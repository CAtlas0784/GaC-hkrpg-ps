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

copy /y "target\release\gameserver.exe" "gameserver.exe" >nul
copy /y "target\release\sdkserver.exe" "sdkserver.exe" >nul

echo.
echo ==============================================
echo [SUCCESS] Build completed!
echo Executables deployed to root:
echo   - gameserver.exe
echo   - sdkserver.exe
echo.
echo You can launch them anytime with 'run.bat'.
echo ==============================================
pause

@echo off
setlocal enabledelayedexpansion
chcp 65001 > nul
cd /d "%~dp0"

title GaC hkrpg ps - Server and Proxy Launcher

echo ==============================================================================
echo   [ GaC hkrpg ps ] Game Server and Proxy Launcher
echo ==============================================================================
echo.
echo   [1] เปิด Server + Proxy ของมัน (Internal Hook / Launcher)
echo   [2] เปิด Server + Gay Proxy   (Windows System MITM Proxy)
echo   [3] เปิด Server อย่างเดียว    (SDK Server + Game Server Only)
echo.
echo ==============================================================================

set "CHOICE=%~1"
if not defined CHOICE (
    set /p "CHOICE=   เลือกตัวเลือก [1-3] (Default: 1): "
)
if not defined CHOICE set "CHOICE=1"

:: 1. Ensure Binaries Exist
if not exist "sdkserver.exe" if exist "target\release\sdkserver.exe" copy /y "target\release\sdkserver.exe" "sdkserver.exe" >nul
if not exist "gameserver.exe" if exist "target\release\gameserver.exe" copy /y "target\release\gameserver.exe" "gameserver.exe" >nul

if not exist "sdkserver.exe" (
    echo [X] sdkserver.exe not found! Please compile or run build.bat first.
    pause
    exit /b 1
)
if not exist "gameserver.exe" (
    echo [X] gameserver.exe not found! Please compile or run build.bat first.
    pause
    exit /b 1
)

:: 2. Terminate Old Server Instances
echo [*] Cleaning old server processes...
taskkill /f /im sdkserver.exe >nul 2>&1
taskkill /f /im gameserver.exe >nul 2>&1

:: If user chose Option 3 (Server Only), ensure any leftover Gay Proxy is also closed and cleaned
if "%CHOICE%"=="3" (
    taskkill /f /im GayProxy.exe >nul 2>&1
    taskkill /f /im GayProxy.Guardian.exe >nul 2>&1
    call :find_gay_proxy
    if defined GAY_PROXY_EXE "!GAY_PROXY_EXE!" --clean >nul 2>&1
)

ping 127.0.0.1 -n 2 >nul

:: 3. Start Core Servers
echo [*] Launching SDK Server [Port 21000]...
start "GaC hkrpg ps - SDK Server [Port 21000]" cmd /k "@echo off & title GaC hkrpg ps - SDK Server [Port 21000] & echo ============================================================================== & echo   [ GaC hkrpg ps ] SDK / Dispatch Gateway (HTTP :21000) & echo ============================================================================== & echo. & sdkserver.exe"

ping 127.0.0.1 -n 2 >nul

echo [*] Launching Game Server [Port 23301]...
start "GaC hkrpg ps - Game Server [Port 23301]" cmd /k "@echo off & title GaC hkrpg ps - Game Server [Port 23301] & echo ============================================================================== & echo   [ GaC hkrpg ps ] Game World Engine (UDP/KCP :23301) & echo ============================================================================== & echo. & gameserver.exe"

ping 127.0.0.1 -n 2 >nul

:: 4. Handle Proxy Options
if "%CHOICE%"=="1" goto opt_internal_proxy
if "%CHOICE%"=="2" goto opt_gay_proxy
if "%CHOICE%"=="3" goto opt_server_only

echo [!] Unknown option '%CHOICE%', defaulting to Server Only.
goto opt_server_only

:: ========================================================================
:: [1] Server + Proxy ของมัน
:: ========================================================================
:opt_internal_proxy
echo.
echo ==============================================================================
echo   [*] โหมด: Server + Proxy ของมัน
echo ==============================================================================
call :find_game_dir
if defined GAME_DIR (
    echo [*] ตรวจพบโฟลเดอร์เกม: !GAME_DIR!
    if exist "!GAME_DIR!\launcher.exe" (
        echo [*] กำลังเปิด Proxy Launcher: !GAME_DIR!\launcher.exe
        start "Proxy Launcher" /d "!GAME_DIR!" "!GAME_DIR!\launcher.exe"
    ) else if exist "!GAME_DIR!\StarRail.exe" (
        if not exist "!GAME_DIR!\version.dll" if not exist "!GAME_DIR!\hkrpg.dll" (
            call :deploy_hook_dll "!GAME_DIR!"
        )
        echo [*] กำลังเปิด StarRail.exe พร้อม Proxy Hook...
        start "Star Rail Client" /d "!GAME_DIR!" "!GAME_DIR!\StarRail.exe"
    ) else (
        echo [!] ไม่พบ StarRail.exe หรือ launcher.exe ในโฟลเดอร์เกม
    )
) else (
    echo [!] ไม่พบโฟลเดอร์เกมอัตโนมัติ กรุณาเปิดตัวเกมที่มี Proxy Hook ด้วยตนเอง
)
goto finish

:: ========================================================================
:: [2] Server + Gay Proxy
:: ========================================================================
:opt_gay_proxy
echo.
echo ==============================================================================
echo   [*] โหมด: Server + Gay Proxy
echo ==============================================================================
call :find_gay_proxy
if defined GAY_PROXY_EXE (
    echo [*] กำลังเปิด Gay Proxy: !GAY_PROXY_EXE!
    start "Hoyo PS - Gay Proxy [System MITM]" /d "!GAY_PROXY_DIR!" "!GAY_PROXY_EXE!" --port 21000
) else if defined GAY_PROXY_BAT (
    echo [*] กำลังเปิด Gay Proxy: !GAY_PROXY_BAT!
    start "Hoyo PS - Gay Proxy [System MITM]" /d "!GAY_PROXY_DIR!" cmd /k "!GAY_PROXY_BAT!"
) else (
    echo [X] ไม่พบโปรแกรม Gay Proxy อัตโนมัติ!
)
goto finish

:: ========================================================================
:: [3] Server อย่างเดียว
:: ========================================================================
:opt_server_only
echo.
echo ==============================================================================
echo   [*] โหมด: Server อย่างเดียว
echo ==============================================================================
echo [OK] เซิร์ฟเวอร์ทำงานที่พอร์ต 21000 และ 23301 เรียบร้อยแล้ว
goto finish

:finish
echo.
echo ==============================================================================
echo [OK] ดำเนินการเปิดระบบเรียบร้อยแล้ว!
echo - ปิดหน้าต่างเซิร์ฟเวอร์แต่ละตัวเพื่อหยุดการทำงาน
echo - หน้าต่างนี้จะปิดตัวลงอัตโนมัติ
echo ==============================================================================
ping 127.0.0.1 -n 3 >nul
exit /b 0

:: ========================================================================
:: Helper: ค้นหาโฟลเดอร์เกมอัตโนมัติ (Zero Hardcoding)
:: ========================================================================
:find_game_dir
set "GAME_DIR="
for /f "tokens=2*" %%A in ('reg query "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Star Rail" /v "InstallPath" 2^>nul') do (
    if exist "%%~B\Games\StarRail.exe" set "GAME_DIR=%%~B\Games"
    if not defined GAME_DIR if exist "%%~B\StarRail.exe" set "GAME_DIR=%%~B"
)
if not defined GAME_DIR (
    for /f "tokens=2*" %%A in ('reg query "HKCU\Software\Cognosphere\Star Rail" /v "InstallPath" 2^>nul') do (
        if exist "%%~B\Games\StarRail.exe" set "GAME_DIR=%%~B\Games"
        if not defined GAME_DIR if exist "%%~B\StarRail.exe" set "GAME_DIR=%%~B"
    )
)
if not defined GAME_DIR (
    for /f "tokens=2*" %%A in ('reg query "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\HoYoPlay" /v "InstallPath" 2^>nul') do (
        if exist "%%~B\games\Star Rail Games\StarRail.exe" set "GAME_DIR=%%~B\games\Star Rail Games"
    )
)
if not defined GAME_DIR (
    for /d %%D in ("%~dp0..\StarRail*" "%~dp0..\Star Rail*" "%USERPROFILE%\Desktop\StarRail*" "%USERPROFILE%\Desktop\Star Rail*") do (
        if not defined GAME_DIR if exist "%%~fD\StarRail.exe" set "GAME_DIR=%%~fD"
    )
)
if not defined GAME_DIR (
    for %%D in (C D E F G H) do (
        if not defined GAME_DIR if exist "%%D:\beta hsr\StarRail_4.6.51_OS\StarRail.exe" set "GAME_DIR=%%D:\beta hsr\StarRail_4.6.51_OS"
        if not defined GAME_DIR if exist "%%D:\HoYoPlay\games\Star Rail Games\StarRail.exe" set "GAME_DIR=%%D:\HoYoPlay\games\Star Rail Games"
        if not defined GAME_DIR if exist "%%D:\Program Files\Star Rail\Games\StarRail.exe" set "GAME_DIR=%%D:\Program Files\Star Rail\Games"
        if not defined GAME_DIR if exist "%%D:\Games\Star Rail\Games\StarRail.exe" set "GAME_DIR=%%D:\Games\Star Rail Games"
        if not defined GAME_DIR if exist "%%D:\Star Rail\Games\StarRail.exe" set "GAME_DIR=%%D:\Star Rail\Games"
    )
)
exit /b 0

:: ========================================================================
:: Helper: ค้นหา Gay Proxy อัตโนมัติ (Zero Hardcoding)
:: ========================================================================
:find_gay_proxy
set "GAY_PROXY_EXE="
set "GAY_PROXY_BAT="
set "GAY_PROXY_DIR="
if exist "%~dp0..\Gay Proxy\bin\Publish\GayProxy.exe" (
    set "GAY_PROXY_EXE=%~dp0..\Gay Proxy\bin\Publish\GayProxy.exe"
    set "GAY_PROXY_DIR=%~dp0..\Gay Proxy\bin\Publish"
)
if not defined GAY_PROXY_EXE if exist "%~dp0..\Gay Proxy\run_proxy.bat" (
    set "GAY_PROXY_BAT=%~dp0..\Gay Proxy\run_proxy.bat"
    set "GAY_PROXY_DIR=%~dp0..\Gay Proxy"
)
if not defined GAY_PROXY_EXE (
    for /d %%D in ("%USERPROFILE%\Desktop\Gay Proxy*" "%~dp0..\Gay Proxy*") do (
        if not defined GAY_PROXY_EXE if exist "%%~fD\bin\Publish\GayProxy.exe" (
            set "GAY_PROXY_EXE=%%~fD\bin\Publish\GayProxy.exe"
            set "GAY_PROXY_DIR=%%~fD\bin\Publish"
        )
        if not defined GAY_PROXY_BAT if exist "%%~fD\run_proxy.bat" (
            set "GAY_PROXY_BAT=%%~fD\run_proxy.bat"
            set "GAY_PROXY_DIR=%%~fD"
        )
    )
)
if not defined GAY_PROXY_EXE (
    for /f "delims=" %%F in ('where /r "%USERPROFILE%\Desktop" GayProxy.exe 2^>nul') do (
        if not defined GAY_PROXY_EXE (
            set "GAY_PROXY_EXE=%%F"
            set "GAY_PROXY_DIR=%%~dpF"
        )
    )
)
exit /b 0

:: ========================================================================
:: Helper: Deploy In-Game Hook DLL (Zero Hardcoding)
:: ========================================================================
:deploy_hook_dll
set "TARGET_DIR=%~1"
for /d %%A in ("%~dp0..\AstralOS" "%USERPROFILE%\Desktop\AstralOS") do (
    if exist "%%~fA\bin\version.dll" (
        copy /y "%%~fA\bin\version.dll" "!TARGET_DIR!\version.dll" >nul 2>&1
        attrib +r "!TARGET_DIR!\version.dll" >nul 2>&1
        echo [*] Deployed version.dll hook into !TARGET_DIR!
        exit /b 0
    )
)
exit /b 0
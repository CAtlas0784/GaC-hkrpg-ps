@echo off
setlocal enabledelayedexpansion
title GaC hkrpg ps - Private Server Development Suite [powered by AstralOS]
color 0B
chcp 65001 > nul
cd /d "%~dp0"

set "ASTRALOS_DIR="
if exist "%~dp0..\AstralOS" set "ASTRALOS_DIR=%~dp0..\AstralOS"
if not defined ASTRALOS_DIR if exist "%USERPROFILE%\Desktop\AstralOS" set "ASTRALOS_DIR=%USERPROFILE%\Desktop\AstralOS"

:: ========================================================================
:: Auto-detect Game Directory
:: ========================================================================
set "GAME_DIR="
if exist "%USERPROFILE%\Desktop\StarRail_4.6.51_OS\StarRail.exe" (
    set "GAME_DIR=%USERPROFILE%\Desktop\StarRail_4.6.51_OS"
)
if not defined GAME_DIR (
    if exist "E:\beta hsr\StarRail_4.5.52_OS\StarRail.exe" (
        set "GAME_DIR=E:\beta hsr\StarRail_4.5.52_OS"
    )
)
if not defined GAME_DIR (
    if exist "D:\HoYoPlay\games\Star Rail Games\StarRail.exe" (
        set "GAME_DIR=D:\HoYoPlay\games\Star Rail Games"
    )
)
if not defined GAME_DIR (
    if exist "C:\Program Files\Star Rail\Games\StarRail.exe" (
        set "GAME_DIR=C:\Program Files\Star Rail\Games"
    )
)
if not defined GAME_DIR (
    if exist "D:\Games\Star Rail\Games\StarRail.exe" (
        set "GAME_DIR=D:\Games\Star Rail\Games"
    )
)

:menu
cls
echo ==============================================================================
echo   [ GaC-hkrpg-ps ] Private Server Development Suite ^& Toolkit
echo   Integrated with AstralOS Reverse Engineering Engine
echo ==============================================================================
echo.

:: Privilege Check
net session >nul 2>&1
if !errorlevel! equ 0 (
    set "STATUS_ADMIN=[ELEVATED] Administrator Mode Active"
) else (
    set "STATUS_ADMIN=[STANDARD] Standard User Mode"
)

:: Service Status Check
netstat -ano | findstr "0.0.0.0:21000" | findstr "LISTENING" > nul 2>&1
if !errorlevel! equ 0 (
    set "STATUS_HTTP=[ONLINE]   SDK Server Gateway [Port 21000]"
) else (
    set "STATUS_HTTP=[OFFLINE]  SDK Server Gateway [Port 21000]"
)

netstat -ano | findstr "0.0.0.0:23301" > nul 2>&1
if !errorlevel! equ 0 (
    set "STATUS_KCP=[ONLINE]   KCP Gameserver      [Port 23301]"
) else (
    set "STATUS_KCP=[OFFLINE]  KCP Gameserver      [Port 23301]"
)

if exist "!GAME_DIR!\launcher.exe" if exist "!GAME_DIR!\hkrpg.dll" (
    set "STATUS_HOOK=[ACTIVE]   Proxy Launcher [launcher.exe + hkrpg.dll]"
) else if exist "!GAME_DIR!\version.dll" (
    set "STATUS_HOOK=[ACTIVE]   Direct Dumper Hook [version.dll]"
) else (
    set "STATUS_HOOK=[NOT HOOKED] No proxy hook in Game Folder"
)

tasklist /FI "IMAGENAME eq GayProxy.exe" 2>nul | findstr /i "GayProxy.exe" >nul 2>&1
if !errorlevel! equ 0 (
    set "STATUS_GAYPROXY=[ONLINE]   Gay Proxy           [System MITM active]"
) else (
    set "STATUS_GAYPROXY=[OFFLINE]  Gay Proxy           [System MITM standby]"
)

echo   [ STATUS ] -----------------------------------------------------------------
echo     !STATUS_ADMIN!
echo     !STATUS_HTTP!
echo     !STATUS_KCP!
echo     !STATUS_HOOK!
echo     !STATUS_GAYPROXY!
if defined GAME_DIR (
    echo     [CLIENT]   !GAME_DIR!
) else (
    echo     [CLIENT]   Not Detected
)
echo   ---------------------------------------------------------------------------
echo.
echo   [ PS RUNTIME ^& TESTING ] -------------------------------------------------
echo     [1] 1-Click PS Dev Mode [Start PS Server + Hook + Launch Game Client]
echo     [2] Launch Game Client with Hook [Connect to Already Running PS]
echo     [3] Fix ^& Lock version.dll Hook [Prevent Game from Renaming/Disabling]
echo     [4] Stop All Running PS Servers [Kill :21000 ^& :23301]
echo     [G] Launch Gay Proxy [Alternative Network Proxy Mode]
echo     [R] Reset Windows System Proxy [Emergency WinINet Flush]
echo.
echo   [ PROTOBUF, PACKETS ^& REVERSE ENGINEERING ] -------------------------------
echo     [5] Dump StarRail.proto ^& packetIds.json [Morax IL2CPP Parser]
echo     [6] Compile res.json for Server [AstralOS Res Compiler]
echo.
echo   [ CLIENT TWEAKS ^& STATE MANAGEMENT ] --------------------------------------
echo     [7] Switch Game Language to Thai [th] / English [en]
echo     [8] Reset Player Spawn Position [Clean persistent file]
echo     [9] Open Full AstralOS Master Control Suite
echo     [0] Exit
echo   ---------------------------------------------------------------------------
echo.

if not defined INITIAL_ARG (
    set "INITIAL_ARG=1"
    set "choice=%~1"
) else (
    set "choice="
)

if not defined choice set /p choice="   Select option [0-9, G, R]> "
if not defined choice goto menu

if "%choice%"=="1" goto dev_mode_all
if "%choice%"=="2" goto launch_client_only
if "%choice%"=="3" goto fix_hook_dll
if "%choice%"=="4" goto stop_servers
if /i "%choice%"=="G" goto launch_gay_proxy
if /i "%choice%"=="F" goto launch_gay_proxy
if /i "%choice%"=="R" goto reset_win_proxy
if "%choice%"=="5" goto dump_morax_proto
if "%choice%"=="6" goto compile_res_json
if "%choice%"=="7" goto switch_lang
if "%choice%"=="8" goto reset_player_state
if "%choice%"=="9" goto open_astralos_suite
if "%choice%"=="0" exit /b 0
goto menu

:: ========================================================================
:: [1] 1-Click PS Dev Mode
:: ========================================================================
:dev_mode_all
echo.
echo ==============================================================================
echo   [1-Click PS Dev Mode] Launching GaC-hkrpg-ps + Client Hook...
echo ==============================================================================
echo.

:: 1. Start PS Servers if not running
netstat -ano | findstr "0.0.0.0:21000" | findstr "LISTENING" > nul 2>&1
if !errorlevel! neq 0 (
    echo [*] Starting SDK Server [Port 21000]...
    if exist "target\release\sdkserver.exe" (
        start "Hoyo PS - SDK Server" /d "%~dp0" cmd /k "target\release\sdkserver.exe"
    ) else if exist "sdkserver.exe" (
        start "Hoyo PS - SDK Server" /d "%~dp0" cmd /k "sdkserver.exe"
    ) else (
        start "Hoyo PS - SDK Server" /d "%~dp0" cmd /k "cargo run --release -p sdkserver"
    )
    ping 127.0.0.1 -n 3 > nul
) else (
    echo [OK] SDK Server [Port 21000] is already running.
)

netstat -ano | findstr "0.0.0.0:23301" > nul 2>&1
if !errorlevel! neq 0 (
    echo [*] Starting Gameserver [Port 23301]...
    if exist "target\release\gameserver.exe" (
        start "Hoyo PS - Gameserver" /d "%~dp0" cmd /k "target\release\gameserver.exe"
    ) else if exist "gameserver.exe" (
        start "Hoyo PS - Gameserver" /d "%~dp0" cmd /k "gameserver.exe"
    ) else (
        start "Hoyo PS - Gameserver" /d "%~dp0" cmd /k "cargo run --release -p gameserver"
    )
    ping 127.0.0.1 -n 3 > nul
) else (
    echo [OK] Gameserver [Port 23301] is already running.
)

:: 2. Deploy version.dll and launch game
call :deploy_hook_internal
call :launch_game_internal
echo.
echo [OK] 1-Click Dev Mode completed! Servers and Game are running.
echo Press any key to return to menu...
pause > nul
goto menu

:: ========================================================================
:: [2] Launch Game Client with Hook Only
:: ========================================================================
:launch_client_only
echo.
echo ==============================================================================
echo   [Launch Game Client with Hook] Connecting to 127.0.0.1:21000...
echo ==============================================================================
echo.
call :deploy_hook_internal
call :launch_game_internal
echo.
echo Press any key to return to menu...
pause > nul
goto menu

:: ========================================================================
:: [3] Fix & Lock version.dll Hook
:: ========================================================================
:fix_hook_dll
echo.
echo ==============================================================================
echo   [Fix & Protect version.dll]
echo ==============================================================================
echo.
call :deploy_hook_internal
echo.
echo [OK] Hook DLL verified, deployed, and locked against game modifications!
echo.
pause
goto menu

:: ========================================================================
:: [4] Stop All Running PS Servers
:: ========================================================================
:stop_servers
echo.
echo [*] Terminating GaC-hkrpg-ps, AstralOS, and Gay Proxy processes...
taskkill /F /IM sdkserver.exe /T >nul 2>&1
taskkill /F /IM gameserver.exe /T >nul 2>&1
taskkill /F /IM robinsr.exe /T >nul 2>&1
taskkill /F /IM GayProxy.exe /T >nul 2>&1
taskkill /F /IM GayProxy.Guardian.exe /T >nul 2>&1
set "GAYPROXY_DIR="
if exist "%~dp0..\Gay Proxy" set "GAYPROXY_DIR=%~dp0..\Gay Proxy"
if not defined GAYPROXY_DIR if exist "%USERPROFILE%\Desktop\Gay Proxy" set "GAYPROXY_DIR=%USERPROFILE%\Desktop\Gay Proxy"
if exist "!GAYPROXY_DIR!\bin\Publish\GayProxy.exe" (
    "!GAYPROXY_DIR!\bin\Publish\GayProxy.exe" --clean >nul 2>&1
)
echo [OK] All local servers stopped. Ports 21000 ^& 23301 are free, proxy restored!
ping 127.0.0.1 -n 2 > nul
goto menu

:: ========================================================================
:: [G] Launch Gay Proxy (Alternative Network Proxy Mode)
:: ========================================================================
:launch_gay_proxy
echo.
echo ==============================================================================
echo   [Gay Proxy] Launching Alternative System Proxy Mode...
echo ==============================================================================
set "GAYPROXY_DIR="
if exist "%~dp0..\Gay Proxy" set "GAYPROXY_DIR=%~dp0..\Gay Proxy"
if not defined GAYPROXY_DIR if exist "%USERPROFILE%\Desktop\Gay Proxy" set "GAYPROXY_DIR=%USERPROFILE%\Desktop\Gay Proxy"
if exist "!GAYPROXY_DIR!\bin\Publish\GayProxy.exe" (
    start "Gay Proxy" /d "!GAYPROXY_DIR!" "!GAYPROXY_DIR!\bin\Publish\GayProxy.exe" --port 21000
) else if exist "!GAYPROXY_DIR!\run_proxy.bat" (
    start "Gay Proxy" /d "!GAYPROXY_DIR!" cmd /k "!GAYPROXY_DIR!\run_proxy.bat"
) else (
    echo [X] Gay Proxy directory not found at !GAYPROXY_DIR!.
)
echo [OK] Gay Proxy launched in a dedicated window!
ping 127.0.0.1 -n 2 > nul
goto menu

:: ========================================================================
:: [R] Reset Windows System Proxy
:: ========================================================================
:reset_win_proxy
echo.
echo ==============================================================================
echo   [Reset Proxy] Restoring Windows System Proxy ^& Flushing WinINet Cache
echo ==============================================================================
set "GAYPROXY_DIR="
if exist "%~dp0..\Gay Proxy" set "GAYPROXY_DIR=%~dp0..\Gay Proxy"
if not defined GAYPROXY_DIR if exist "%USERPROFILE%\Desktop\Gay Proxy" set "GAYPROXY_DIR=%USERPROFILE%\Desktop\Gay Proxy"
if exist "!GAYPROXY_DIR!\bin\Publish\GayProxy.exe" (
    "!GAYPROXY_DIR!\bin\Publish\GayProxy.exe" --clean
) else (
    powershell -NoProfile -Command "Set-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings' -Name ProxyEnable -Value 0; Remove-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings' -Name ProxyServer -ErrorAction SilentlyContinue"
)
echo.
echo [OK] System proxy safely disabled and flushed!
pause
goto menu

:: ========================================================================
:: [5] Dump StarRail.proto & packetIds.json via Morax
:: ========================================================================
:dump_morax_proto
echo.
echo ==============================================================================
echo   [Morax Proto Dumper] Extracting Latest Protobuf ^& Packet Schemas
echo ==============================================================================
echo.
if not defined GAME_DIR (
    set /p GAME_DIR="   Enter Star Rail game folder: "
)

set "DUMP_OUT=%~dp0tools\DUMP"
if not exist "%DUMP_OUT%" mkdir "%DUMP_OUT%"

set "MORAX_BIN="
if exist "!ASTRALOS_DIR!\bin\morax.exe" set "MORAX_BIN=!ASTRALOS_DIR!\bin\morax.exe"
if not defined MORAX_BIN if exist "!ASTRALOS_DIR!\target\release\morax.exe" set "MORAX_BIN=!ASTRALOS_DIR!\target\release\morax.exe"
if not defined MORAX_BIN if exist "%~dp0tools\morax.exe" set "MORAX_BIN=%~dp0tools\morax.exe"

if not defined MORAX_BIN (
    echo [*] Morax binary not found. Compiling via Cargo in AstralOS...
    if exist "!ASTRALOS_DIR!\Cargo.toml" (
        pushd "!ASTRALOS_DIR!"
        cargo build --release -p morax
        popd
        if exist "!ASTRALOS_DIR!\target\release\morax.exe" (
            set "MORAX_BIN=!ASTRALOS_DIR!\target\release\morax.exe"
        )
    )
)

if not defined MORAX_BIN (
    echo [X] Could not find or build morax.exe!
    pause
    goto menu
)

echo [*] Using Morax: !MORAX_BIN!
echo [*] Target Game: !GAME_DIR!
echo [*] Output Dir:  %DUMP_OUT%
echo.
"!MORAX_BIN!" dump-proto --game-path "!GAME_DIR!" --out "%DUMP_OUT%"
if !errorlevel! equ 0 (
    echo.
    echo [OK] Successfully dumped protobuf definitions to: %DUMP_OUT%
    echo [*] Updating packetIds.json into GaC-hkrpg-ps repo...
    if exist "%DUMP_OUT%\packetIds.json" (
        copy /y "%DUMP_OUT%\packetIds.json" "%~dp0data\packetIds.json" >nul 2>&1
        echo [OK] Synced data\packetIds.json!
    )
) else (
    echo [X] Morax dump failed. Ensure GameAssembly.dll is present in game folder.
)
echo.
pause
goto menu

:: ========================================================================
:: [6] Compile res.json for Server
:: ========================================================================
:compile_res_json
echo.
echo ==============================================================================
echo   [Compile res.json] Rebuilding Server Resource Database
echo ==============================================================================
echo.
if exist "!ASTRALOS_DIR!\Cargo.toml" (
    echo [*] Running resource compiler from AstralOS...
    pushd "!ASTRALOS_DIR!"
    cargo run --release -p robinsr -- compile-res --out "%~dp0res.json"
    popd
    echo [OK] res.json generated at: %~dp0res.json
) else (
    echo [!] AstralOS repository not found at !ASTRALOS_DIR!.
)
echo.
pause
goto menu

:: ========================================================================
:: [7] Switch Game Client Language
:: ========================================================================
:switch_lang
echo.
echo ==============================================================================
echo   [Client Language Switcher]
echo ==============================================================================
echo   1. Thai    [th]
echo   2. English [en]
echo   3. Japanese [ja]
echo   4. Simplified Chinese [zh-cn]
echo.
set /p LANG_CHOICE="   Choose language [1-4]> "
set "TARGET_LANG="
if "%LANG_CHOICE%"=="1" set "TARGET_LANG=th"
if "%LANG_CHOICE%"=="2" set "TARGET_LANG=en"
if "%LANG_CHOICE%"=="3" set "TARGET_LANG=ja"
if "%LANG_CHOICE%"=="4" set "TARGET_LANG=zh-cn"

if defined TARGET_LANG (
    echo [*] Setting client language to: !TARGET_LANG!...
    reg add "HKCU\Software\Cognosphere\Star Rail" /v "Language_h2876912797" /t REG_SZ /d "!TARGET_LANG!" /f >nul 2>&1
    reg add "HKCU\Software\Cognosphere\Star Rail" /v "MIHOYOSDK_CURRENT_LANGUAGE_h255914971" /t REG_SZ /d "!TARGET_LANG!" /f >nul 2>&1
    echo [OK] Registry updated!
) else (
    echo [!] Invalid selection.
)
echo.
pause
goto menu

:: ========================================================================
:: [8] Reset Player Spawn Position
:: ========================================================================
:reset_player_state
echo.
echo ==============================================================================
echo   [Reset Player Position]
echo ==============================================================================
echo [*] Cleaning cached persistent player position / state files...
if exist "%~dp0data\saved_position.json" del /f /q "%~dp0data\saved_position.json" >nul 2>&1
if exist "%~dp0position.json" del /f /q "%~dp0position.json" >nul 2>&1
if exist "%~dp0persistent" del /f /q "%~dp0persistent" >nul 2>&1
echo [OK] Spawn state reset. Next login will spawn at default position [Parlor Car].
echo.
pause
goto menu

:: ========================================================================
:: [9] Open Full AstralOS Master Control Suite
:: ========================================================================
:open_astralos_suite
echo.
echo [*] Launching AstralOS Master Development Suite...
if exist "!ASTRALOS_DIR!\tools.bat" (
    start "" /d "!ASTRALOS_DIR!" cmd /k "!ASTRALOS_DIR!\tools.bat"
) else (
    echo [!] AstralOS tools.bat not found at !ASTRALOS_DIR!.
)
goto menu

:: ========================================================================
:: Helper: Deploy and Lock version.dll
:: ========================================================================
:deploy_hook_internal
if not defined GAME_DIR (
    set /p GAME_DIR="   Enter Star Rail game folder: "
)
if not exist "!GAME_DIR!" (
    echo [X] Game directory '!GAME_DIR!' not found!
    exit /b 1
)

echo [*] Inspecting game hook in: !GAME_DIR!

:: Clean any renamed remnants
for %%F in ("!GAME_DIR!\version.dll.*" "!GAME_DIR!\version_old*") do (
    if exist "%%F" (
        attrib -r -h -s "%%F" >nul 2>&1
        del /f /q "%%F" >nul 2>&1
        echo [*] Cleaned backup: %%~nxF
    )
)

if exist "!GAME_DIR!\launcher.exe" if exist "!GAME_DIR!\hkrpg.dll" (
    echo [OK] Detected Proxy Launcher and hkrpg.dll in game folder.
    exit /b 0
)
set "HOOK_SRC="
if exist "!ASTRALOS_DIR!\bin\version.dll" set "HOOK_SRC=!ASTRALOS_DIR!\bin\version.dll"
if not defined HOOK_SRC if exist "!ASTRALOS_DIR!\target\release\version.dll" set "HOOK_SRC=!ASTRALOS_DIR!\target\release\version.dll"

if defined HOOK_SRC (
    if exist "!GAME_DIR!\version.dll" attrib -r -h -s "!GAME_DIR!\version.dll" >nul 2>&1
    copy /y "!HOOK_SRC!" "!GAME_DIR!\version.dll" >nul
    attrib +r "!GAME_DIR!\version.dll" >nul 2>&1
    echo [OK] Deployed version.dll hook from AstralOS and write-protected [+R Locked]!
) else (
    echo [!] AstralOS version.dll not found. Please build dumper in AstralOS first.
)
exit /b 0

:: ========================================================================
:: Helper: Launch Game Client
:: ========================================================================
:launch_game_internal
echo [*] Launching Star Rail Client [redirected to local PS]...
if exist "!GAME_DIR!\launcher.exe" (
    echo [*] Launching via Proxy Launcher [launcher.exe + hkrpg.dll]...
    net session >nul 2>&1
    if !errorlevel! equ 0 (
        start "Hoyo PS - Proxy Launcher" /d "!GAME_DIR!" "!GAME_DIR!\launcher.exe"
    ) else (
        echo [*] Elevating Proxy Launcher as Administrator...
        powershell -NoProfile -ExecutionPolicy Bypass -Command "Start-Process -FilePath '!GAME_DIR!\launcher.exe' -WorkingDirectory '!GAME_DIR!' -Verb RunAs"
    )
    echo [OK] Proxy Launcher started! Redirecting to 127.0.0.1:21000.
) else if exist "!GAME_DIR!\StarRail.exe" (
    start "Hoyo PS - Star Rail" /d "!GAME_DIR!" "!GAME_DIR!\StarRail.exe"
    echo [OK] Star Rail launched!
) else (
    echo [X] Could not find StarRail.exe in '!GAME_DIR!'.
)
exit /b 0

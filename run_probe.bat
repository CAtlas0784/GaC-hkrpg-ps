@echo off
title HSR Protocol Probe CLI
color 0a
echo ============================================================
echo   HSR Protocol Probe CLI - Packet Testing Tool
echo ============================================================
target\debug\probe.exe 127.0.0.1:23301
pause

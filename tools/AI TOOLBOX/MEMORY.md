# AI TOOLBOX - Project Long-Term Memory

## Project Overview
Honkai Star Rail Game Server (.NET / C# and Rust Architecture & Protocol Simulation).

## Core Directives & User Preferences
1. **Approval Automation**: Group tasks into single automated commands or use subagents; avoid asking repetitive approval prompts.
2. **FreeCam System**: Preserve live-reloading of `scripts/freecam.lua` via `PlayerHeartBeatScRsp.download_data` (CmdID: 62). Maintain user's exact controls and rainbow watermark.
3. **Challenge Unlocks**: Keep stages unlocked (`hgpkmhfpmbj: true` / `is_unlock: true`) across Forgotten Hall (Jarilo-VI 15, Xianzhou Luofu 6), MoC (12 floors), Pure Fiction (4 stages), and Apocalyptic Shadow (4 difficulties).
4. **Hotfix Sync**: Hotfix URLs are stored in `versions.json`. SDK server delivers hotfix payloads for target client versions (e.g. `OSBETAWin4.5.51`, `OSBETAWin4.5.52`, `CNBETAWin4.3.51..55`).
5. **No Code Overwrites**: Never blindly wipe or revert user source code without explicit verification.
6. **External Reference**: `https://git.neonteam.dev/amizing/robinsr` (Third-party repo used only as an external reference to inspect when game updates break server compatibility).
7. **No Vanity / Slop Logs**: Never output fake or vanity console logs pretending handlers or systems are registered/functional when they are placeholders. Only log real operations that actually execute.
8. **Dimbreath Game Data**: `https://gitlab.com/Dimbreath/turnbasedgamedata` (Primary source for exact game tables: ExcelOutput, ChallengeMazeConfig, stages, etc.).
9. **Daily Memory Logging (Continuous)**: Maintain daily logs in `tools/AI TOOLBOX/memory/YYYY-MM-DD.md`. Update this file continuously throughout active sessions to ensure all technical discoveries, reverse engineering findings, and architectural shifts are persistently saved across chat resets.
10. **Open Source & Mod-Friendly Data**: Do NOT compress or pack game tables into opaque binary blobs like March7th's `Resource.bin`. Keep resources open, transparent, and moddable in standard JSON directories (`resources/Config`, `ExcelOutput`, `TextMap`).
11. **MhyKcp Protocol (28-byte Header)**: HoYoverse games modify standard KCP by inserting a 4-byte `token` after `conv`: `conv(4) + token(4) + cmd(1) + frg(1) + wnd(2) + ts(4) + sn(4) + una(4) + len(4)` (Little-Endian). Standard KCP libraries cannot read this; use the native C# `MhyKcp.cs`.
12. **Login Security (login_random)**: In `PlayerLoginCsReq` (CmdId: 34), client sends a 64-bit random number (`login_random`, Tag 6). The server MUST echo this exact value in `PlayerLoginScRsp` (CmdId: 25, Tag 14). Failing to do so triggers Error 1001_3.
13. **Dispatch HTTP Server**: Must use ASP.NET Core Kestrel on port 21000 to prevent raw TCP `StreamReader` buffer hangs (which triggers Error 1001_1).
14. **MoC & Endgame Unlock Protocol**:
    - MoC floors 11–12 require matching actual `RewardID`s in `ChallengeHistoryMaxLevel.reward_display_type` (`101201..=101212` for level 12).
    - Apocalyptic Shadow tab requires active schedule in `GetActivityScheduleConfigScRsp` (CmdID: 2616) with `panel_id: 21001` and `activity_id: 21001..=21030`.
    - Total challenge stages: 790 stages across 101 groups.
15. **Exact Command IDs (4.5.52)**:
    - `PlayerGetToken`: CsReq: 60 -> ScRsp: 16
    - `PlayerLogin`: CsReq: 34 -> ScRsp: 25
    - `PlayerLoginFinish`: CsReq: 87 -> ScRsp: 63
    - `PlayerHeartBeat`: CsReq: 44 -> ScRsp: 62
    - `GetCurSceneInfo`: CsReq: 1458 -> ScRsp: 1456
    - `GetChallenge`: CsReq: 1734 -> ScRsp: 1725
    - `GetCurChallenge`: CsReq: 1715 -> ScRsp: 1742
    - `GetBasicInfo`: CsReq: 80 -> ScRsp: 67
    - `GetActivityScheduleConfig`: CsReq: 2625 -> ScRsp: 2616
16. **Dependency & Pip Installation Tracking**:
    - Whenever any script, tool, or component requires installing packages via `pip install`, ALWAYS record the package and its version in `requirements.txt` and note it in `tools/AI TOOLBOX/memory/YYYY-MM-DD.md`.
    - This ensures friends or other users who receive the project can simply run `pip install -r requirements.txt` and execute all scripts/tools without encountering missing module errors.
17. **Skill & Knowledge Crystallization**:
    - Whenever a complex protocol mechanism, reverse engineering pattern, build pipeline, or operational workflow is solved, ALWAYS immediately create/update a dedicated Skill in `tools/AI TOOLBOX/skills/<skill-name>/SKILL.md` and write the exact findings into memory.
    - Never rely on memory alone or re-search from scratch in subsequent sessions; consult and maintain the skill library first.

# GaC hkrpg ps

A native, high-performance Rust private server implementation for Honkai: Star Rail (Supported Versions: **4.4.5x - 4.5.52+**).

---

## 🌟 About This Project / Contribution Notice

This repository is forked and actively contributed/developed upon:
- **Original / Upstream Repository**: [horoyoi-san/Hoyo (branch: `hkrpg-RobinSR`)](https://github.com/horoyoi-san/Hoyo/tree/hkrpg-RobinSR)
- Built upon the foundation of **RobinSR & JadeSR** by [reversedrooms](https://git.xeondev.com/reversedrooms):
  - [RobinSR Repository](https://git.xeondev.com/reversedrooms/RobinSR)

### 🚀 Major Enhancements & New Features
- **Full Endgame Challenge Protocol Overhaul**:
  - **Forgotten Hall**: Complete story stages (Jarilo-VI 15 floors & Luofu 6 floors) with full 3-star bitmasks.
  - **Memory of Chaos (MoC)**: All floors (1–12) unlocked with 36/36 ⭐, 20-cycle records, and active season stages (GroupID 1035).
  - **Pure Fiction (PF)**: All floors (1–4) unlocked with 12/12 ⭐, 80,000 score records (clearing the "No Data" lock state).
  - **Apocalyptic Shadow (AS)**: Floors (1–4) unlocked with full trial statistics and 4,000/8,000 score ratings.
  - **Starward Mode / 3-Node Challenge Tierce**: Fully supported Tierce mode data (`8981`/`8980`), 3-node lineup management (`8979`/`8995`), and active boss data.
- **Direct Challenge Battle Entry**: (i need new res.json for fix this bug)
  - Clicking **Start Challenge** on the lineup UI smoothly loads the arena scene (`load_challenge_scene`) with monsters, props, and custom player placement.
  - Automatically transfers your chosen lineup from the UI into the turn-based combat system (`battle_config.custom_battle_lineup`).
  - Precomputed `challenge_data.json` containing 783 standard stages + 7 Tierce stages and 1,509 wave configs with exact monster IDs and levels.
- **Protocol & Network Fixes for v4.5.52 Client**:
  - Replaced naive byte slicing with full protobuf varint decoding for Challenge Group IDs (`GetChallengeGroupStatisticsCsReq` CmdID 1711).
  - Corrected `retcode` tags across Challenge responses (`GetCurChallengeScRsp` Tag 13, `GetChallengeTierceDataScRsp` Tag 12, `GetChallengeTierceControllerScRsp` Tag 7).
  - Implemented response handlers for Clearance Lineup (`2966`/`2916`/`2919`) and Lineup Avatar data (`740`).
- **Ease of Use**:
  - Quick build script `build.bat` (`cargo build --release`).
  - One-click launcher `run.bat` supporting both root directory binaries and `target/release/`.

---

## 🛠️ Installation & Building

### Requirements
- [Rust](https://www.rust-lang.org/tools/install) (Nightly toolchain required)

Install Rust and configure the nightly toolchain:
```sh
rustup toolchain install nightly
rustup default nightly
```

*(Optional: If you use auxiliary Python scripts in `tools/`, install dependencies via `pip install -r requirements.txt`)*

### Building the Project
Simply run:
```cmd
build.bat
```
*(Or run `cargo build --release` in the terminal)*

---

## 🎮 Running the Server

Double-click:
```cmd
run.bat
```
This will start both the **SDK Server** (port 21000) and the **Game Server** (port 21101).

---

## 📝 Notes & Tips
- **Tool Website**: [https://srtools.neonteam.dev](https://srtools.neonteam.dev)
- If you experience scene loading issues, delete or reset the `persistent` file in the root directory.
- For development notes, reverse engineering research, and protocol specs, check `tools/AI TOOLBOX/`.

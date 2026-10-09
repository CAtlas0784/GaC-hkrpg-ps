---
name: hkrpg-ps-version-update
description: คู่มือ Honkai Star Rail Private Server (GaC-hkrpg-go ภาษา Go + GaC-hkrpg-ps Rust ตัวเก่า) บทเรียนอัปเดตข้ามเวอร์ชัน client และกับดักที่พิสูจน์แล้ว ใช้เมื่อ client อัปเดตแล้วเซิร์ฟเวอร์มีอาการ จอดำ/ค้าง/กล้องหลุดตอนกดตี/packet Unknown/dialogue ค้าง/crash
---

# คู่มือ HSR Private Server — บทเรียนที่พิสูจน์แล้ว (อัปเดตล่าสุด: 2026-10-07)

> **สถานะปัจจุบัน:** เซิร์ฟเวอร์หลักคือ **`C:\Users\Phitchayut\Desktop\GaC-hkrpg-go` (ภาษา Go, client 4.6.51)**
> โปรเจกต์ Rust `GaC-hkrpg-ps` เป็นตัวต้นทาง/reference — โค้ด Rust ล่าสุดแตกตัวกับ proto ใน repo (ดูหัวข้อ 10)
> เอกสารฉบับเต็มของตัว Go อยู่ที่ `GaC-hkrpg-go/README.md`

---

## 1. ลำดับการวินิจฉัยเมื่อ client ใหม่เข้าไม่ได้

อ่าน log ตามลำดับนี้ **ก่อนแก้โค้ด** เพราะอาการเดียวกันมีหลายต้นเหตุ:

| อาการ | แหล่งข้อมูล | สิ่งที่มองหา |
|---|---|---|
| ค้างก่อนหน้า login | client hook log | URL ที่ถูก redirect/block |
| จอดำหลังกด login | `Player.log` (LocalLow/Cognosphere/Star Rail) | `NullReferenceException` + ชื่อ handler `_CmdXxxScRsp` + **offset** (offset ต่างกัน = field คนละตัวใน handler เดียวกัน) |
| จอดำ/ค้างหลัง login finish | `Player.log` + log หน้าต่าง Game Server | packet ที่ client ส่งต่อแต่เซิร์ฟ "Unknown command ID" |
| ค้างหน้าโหลดฉาก | `Player.log` | `CreateMemberEntityLeader FAILED`, `LuaException` (GotoMapParam ฯลฯ), ExcelTable FAILED |
| กล้องหลุดจากตัว/ค้างหลังกดตี | `Player.log` + `server_debug.log` (บรรทัด `SCENE_CAST_SKILL`) | battle transition จาก target ที่ไม่ใช่ monster (ดูหัวข้อ 11) |
| เกมดับ | `Temp/Cognosphere/Star Rail/Crashes` | crash report |
| จุดวาป/คุยผิดปกติ | `server_debug.log` ในโฟลเดอร์เซิร์ฟ | บรรทัด `ENTER_SCENE` / `INTERACT_PROP` (บันทึก entry_id/teleport_id/prop จริง) |

กฎสำคัญ: **`Player.log` ถูกเขียนทับทุกครั้งที่เปิดเกม** — ต้องอ่านทันทีหลังทดสอบ และอย่าสรุปจาก error ของ session เก่าที่จำ offset ไว้

## 2. Checklist สิ่งที่ต้องอัปเดตต่อเวอร์ชัน client

1. **Proto/cmd dump ใหม่** — dump จาก client ด้วย Morax หรือใช้ dump จาก AstralOS รุ่นใหม่ (ตัว Go: `protocol/proto/StarRail.proto` + generate ใหม่ด้วย protoc-gen-go)
2. **`cmd_ids_<ver>.json`** — mapping cmd ทั้งหมดของรุ่นนั้น (ตัว Go generate cmdid map จาก `// CmdID:` comments ใน proto อัตโนมัติ)
3. **Data files จาก AstralOS dump** (ใช้ได้ — เป็น data จาก client ไม่ใช่โค้ด): `challenge_data.json`, `res.json`, `teleports.json`, `mission_unlocks.json`, `freesr-data.json`
4. **`BASE_AVATAR_IDS`** — เพิ่ม avatar ตัวใหม่ (ตัว Go: `internal/gameserver/dispatch.go` — 89 ตัวใน 4651 มี 1511/1512/1513)
5. **`versions.json`** — hotfix URLs ต่อเวอร์ชัน (ดูหัวข้อ 5)
6. **Tutorial ID lists** — ตัว Go generate จาก Rust tutorial.rs แล้ว (`handlers_tutorial_ids.go` 1249+855 ids)

## 3. Cmd ID และ response layout เปลี่ยนทุกรุ่น

- **ScRsp id เลื่อนตำแหน่ง**: `StartChallengeScRsp` เคยเป็น 1775 → 4.6.51 กลายเป็น **1787** และ 1775 ไปเป็น `GetChallengeGroupStatisticsCsReq` — ส่ง response ผิด id = client รอไม่ได้ หมุนค้างตลอด
- **Field tag เลื่อน**: `StartChallengeCsReq.challenge_id` tag 12 → **tag 14** — decoder มือต้องอัปเดต
- **Response body layout เปลี่ยน**: tag ของ retcode/scene/lineup ใน ScRsp ต้องแกะใหม่จาก proto ทุกรุ่น
- cmd ที่ไม่มีใน dump (เช่น 1452, Anomaly Arbitration) = เนื้อหาหลัง beta build ที่ client ถือ — ทำไม่ได้จนกว่าจะมี dump ใหม่
- **ชื่อ message เปลี่ยนทุกรุ่น**: 4.6.51 เปลี่ยน `PveBattleResultCsReq → PVEBattleResultCsReq`, `QuickStartCocoonStageCsReq → StartQuickCocoonStageCsReq/StartQuickCocoonStageRsp`, `GateServer → Gateserver`, `GetGachaInfoCsReq/DoGachaCsReq หายไป` — **โค้ดต้องผูกกับ proto ของรุ่นนั้นจริงๆ เสมอ**

## 4. Optional message fields — สาเหตุจอดำที่ไล่ยากที่สุด

`_CmdGetBasicInfoScRsp` NRE วนซ้ำเพราะ `PlayerSettingInfo` มี optional message ซ่อนหลายตัว (`ojpaodihaje` Tag 13, `ankidjjinei` Tag 1171, `cmfcooeeecb` Tag 1552) — ปล่อย None แล้ว client unwrap → NRE → จอดำ

วิธีไล่: NRE offset ใน Player.log ต่างกัน = field คนละตัว ใส่ทีละตัวจนหมด ใช้ `field_overrides_<build>.json` ช่วยแปลชื่อ

## 5. Design data CDN — ต้นเหตุ "ตารางพัง/ค้างหน้าโหลด"

- `query_gateway` response (message `Gateserver`) มี `asset_bundle_url` / `ex_resource_url` / `lua_url` — มาจาก `versions.json` (sdkserver fetch จาก CDN จริงผ่าน proxy)
- client beta **hot-update design data** จาก URL เหล่านี้ — ถ้า CDN push ชุดใหม่กว่า GameAssembly ที่ติดตั้ง → ExcelTable โหลด FAILED → ค้างหน้าโหลด (ดู D number: 4.6.51 ติดตั้ง = D16700702 ≠ D16706787)
- **แก้**: `ex_resource_url` ต้องชี้ `design_data/BetaLive/output_<D ที่ตรงกับ client>` (ตัวอย่าง: `https://autopatchcn.bhsr.com/design_data/BetaLive/output_16700702_...`) — **ห้ามชี้ /no_update** เพราะ client beta ไม่ fallback ใช้ local
- ไฟล์ที่โหลดมาแล้วพังค้างใน `StreamingAssets/DesignData/Windows/` — restore จาก `.bak` หรือ re-verify

## 6. กับดักที่พบจริงใน 4.6.51 (เก็บไว้เจอซ้ำ)

- **ทีมเกิน 4 ตัว**: client ส่งทีม 2 ช่องทาง (`avatar_lineup_first` + `first_lineup`) รวมได้ 8 ตัว → `TeamManager` IndexOutOfRange ตอนเข้า arena — แก้ด้วย dedupe + truncate(4)
- **InteractProp**: เก็บ mapping entity_id → prop ตอนโหลดฉาก แล้วตอบ `prop_state` **จริงจาก config** — ห้ามตอบ `interact_id2` (มันคือ interaction id เช่น 1100 ไม่ใช่ state)
- **GetNpcTakenReward (2174)**: client ถามก่อนคุย NPC — ไม่ตอบ = dialogue ไม่มีตัวเลือกและค้าง
- **Tutorial**: mark ทุก id ว่าดูแล้ว (`GetTutorial`/`GetTutorialGuide` status enum `Oekdmamgidc`=2) ไม่งั้น popup เด้งวนและ**บล็อก input**; `FinishTutorial/UnlockTutorial*` ต้องตอบพร้อม Tutorial/TutorialGuide message
- **GetMissionStatusScRsp ต้องส่ง `finished_main_mission_id_list` + `curversion_finished_main_mission_id_list` (tag 13)** — ตอบเปล่า = client Lua NRE → บล็อก input + client ส่ง cmd 1204 ซ้ำรัวๆ
- **GetSwitchHandData**: 4.6.51 CsReq 8115 → ตอบ ScRsp **8104** (tag 3 retcode 0)
- **arena ใหม่ไม่มีใน client เก่า**: challenge phase ใหม่ (เช่น 5312) ใช้ arena 3014101 ซึ่ง client build เก่าไม่มี MapEntrance → Lua `GotoMapParam: attempt to index nil with 'ID'` — fallback ไป arena มาตรฐาน 3000101
- **Scene ไม่มี teleport**: ฉาก arena ไม่มี teleport ใน res — fallback จาก `teleports.json` หรือ spawn ใกล้ prop แรกหันหน้าเข้าหา
- **full coverage ตอบ ScRsp ว่างทุก cmd = อันตราย!** — client modules (`GridFight`, `ChessRogue`, `EvolveBuild`, `ChimeraDuel`, `Jukebox`) รับว่างแล้ว `Sync(nil)` → NRE วน → **จอดำ** — cmd ไม่รู้จักต้อง **drop เงียบๆ** ตอบว่างได้เฉพาะ dummy list ที่ทดสอบแล้ว
- **BASE_AVATAR_IDS ห้ามใส่ตัวที่ client build นี้ไม่มี** (เช่น id จาก reference รุ่นใหม่กว่า) → พังตอน init โลก; ขาดตัวที่มีจริงก็ crash ตอน battle เช่นกัน
- **freesr-data.json ฟอร์แมตใหม่** (จาก SRTools): avatars 99 ตัว ไม่มี key leader/lineups — parser ต้อง tolerate ฟิลด์แปลก (Go: `LoadFreesrData` รองรับแล้ว)

## 7. เครื่องมือและแหล่งข้อมูล

- **AstralOS dump**: `C:\Users\Phitchayut\Downloads\New folder` (bin/*.json = data 4651, tools/reference/* = proto/cmdids/**field_overrides 4651** แปล obfuscated → ชื่อจริง) — ห้ามก็อปโค้ดทั้งดุ้น ใช้ data ได้
- **`field_overrides_<build>.json`** — ดัชนีแปลชื่อ field obfuscated → ชื่อจริงของทุก message
- **Firefly Shelter** (`C:\Program Files\Firefly Shelter\server\firefly-go`) — PS ภาษา Go ที่เวิร์กกับ client นี้ **ปิด source** — สำรวจได้จาก `tools/firefly_dump` (symbols: handlers/pb fields/cmds) และ data dir ของมัน (ใช้ freesr-data.json / data-in-game.json / version.json ฟอร์แมตเดียวกับ FreeSR)
- **`server_debug.log` / `gameserver.log`** — บันทึก ENTER_SCENE/INTERACT_PROP/SCENE_CAST_SKILL + log ทุก cmd
- **Dimbreath Game Data**: `https://gitlab.com/Dimbreath/turnbasedgamedata` (ExcelOutput แม่นยำ)
- **robinsr reference**: `https://git.neonteam.dev/amizing/robinsr` (อ่านเทียบได้ ห้ามก็อปทั้งดุ้น)

## 8. กระบวนการทำงานที่พิสูจน์แล้วว่าเร็ว

1. ทดสอบ 1 อย่าง → อ่าน log ทันที → แก้ 1 จุด → build → ทดสอบซ้ำ (อย่าแก้หลายจุดโดยไม่มี log ยืนยัน)
2. Reference จาก PS อื่น: **อ่านเทียบได้ ห้ามก็อปโค้ดทั้งดุ้น** — ดึงเฉพาะ data dump และค่า config
3. **commit ทุกครั้งที่แก้เสร็จ** — ไฟล์เคยถูกย้อนกลับโดยไม่ทราบสาเหตุ 2 ครั้ง
4. แก้ path hardcode ใน .bat ทุกครั้งที่ย้าย/rename โฟลเดอร์
5. เปลี่ยน versions.json ทีละ field ด้วย script — อย่าแก้มือตอนเซิร์ฟเวอร์กำลังรัน

## 9. สถานะที่รู้ว่ายังไม่สำเร็จ (ณ 4.6.51)

- Anomaly Arbitration — ไม่มี cmd ใน dump ของ client รุ่นนี้
- Currency War (ตระกูล Rogue 101 cmd) — ตอบ default ครบ แต่เล่นได้จริงต้อง implement state เฉพาะ
- Gacha (GetGachaInfo/DoGacha) — proto 4.6.51 เปลี่ยนชื่อ message → ตัว Go skip handler ไว้ (log "handler skipped") ต้องหาชื่อใหม่จาก dump
- StartChallenge กับ challenge_id ที่ไม่มีใน challenge_data.json — ใช้ fallback arena 3014101
- ยังไม่ได้ทดสอบกับ client จริงแบบเต็ม session หลังพอร์ต Go (รอ user ทดสอบ: login → Parlor Car → เดิน → กดตี → วาป → เข้า endgame)

## 10. โค้ด Rust แตกตัวกับ proto ใน repo (บทเรียนสำคัญ)

commit สุดท้ายของ Rust (`ebc27a1`) อ้างชื่อ message ที่ **ไม่มีใน** `proto/StarRail.proto` ของตัวเองแล้ว:
`PveBattleResultCsReq`, `QuickStartCocoonStageCsReq`, `GateServer`, `GetGachaInfoCsReq`, `GroupRefreshInfo`, `LineupSlot`
→ สาเหตุ: proto ถูกแทนด้วย dump 4.6.51 ใหม่ (ชื่อ obfuscated พิมพ์ใหญ่) แต่ handler ไม่ได้ตาม
→ **กฎ:** หลังแทน proto ต้อง compile ทันทีและแก้ชื่อให้ตรง ก่อนปล่อย build ให้ user ทดสอบ
→ ตัว Go แก้ครบโดยผูกทุก handler กับชื่อจริงใน proto (handler ไหน proto ไม่มีจะ log+skip ไม่ panic)

## 11. บัคกล้องหลุดตอนกดตี — SceneCastSkill (แก้แล้วในตัว Go)

**อาการ:** กดโจมตีใน overworld แล้วกล้องหลุดออกจากตัวละคร

**ต้นเหตุ (Rust battle.rs):** handler ตอบ `battle_info` + `monster_battle_info` ให้เข้า battle กับ target ที่ผ่าน filter หลวม `id > 30000 || id < 1000` — ซึ่งรวม **avatar entity ของผู้เล่นเอง** (1-4); ถ้าอยู่ arena (entry >= 3000000) ยังหลอก target เป็น 30001 เสมอ → client เริ่ม battle transition กับ "monster" ที่ไม่มีจริง → กล้อง battle ไม่มี target ให้ตาม → หลุด

**วิธีแก้ (Go `internal/gameserver/handlers_battle.go`):**
1. ตอนโหลดฉาก เก็บ monster entity จริง (entity_id → event_id) ไว้ใน `Session.MonsterEntities`
2. `SceneCastSkill` ยอมรับเฉพาะ hit target ที่เป็น monster จริง
3. ไม่มี monster จริง → ตอบ `retcode 0 + cast_entity_id` **ไม่เข้า battle** (client เล่นแอนิเมชันตีปกติ กล้องค้างกับตัว)
4. ตีโดน monster จริง (รวมบอส arena 30001) → เข้า battle ปกติ
5. ยืนยันด้วย `internal/gameserver/smoke_test.go` (จำลอง client เต็มรูปแบบผ่าน UDP จริง)

**บทเรียน:** อย่าตอบ battle/transition ให้ client จาก target ที่ server ไม่ได้ spawn เอง — ตรวจกับ entity registry ของฉากปัจจุบันเสมอ

## 12. Go server (GaC-hkrpg-go) — สถาปัตยกรรมและข้อควรรู้

- **mihoyo KCP header 28 ไบต์**: `conv(4) + token(4) + cmd(1) + frg(1) + wnd(2) + ts(4) + sn(4) + una(4) + len(4)` Little-Endian — standard KCP lib อ่านไม่ได้; Go port อยู่ `pkg/kcp` (port จาก kcp.rs 1:1, UDP loopback test ผ่าน)
- **นาฬิกา KCP ต้องเป็นมิลลิวินาที** — Rust เดิมใช้วินาที (session_time secs) ทำ RTT/resend เพี้ยนทั้งระบบ
- **Handshake**: 20-byte NetOperation — client `head=0xFF, tail=0xFFFFFFFF`, server ตอบ `head=0x145, tail=0x14514545` (param1=conv, param2=token), ตัดการเชื่อมต่อ `head=0x194, tail=0x19419494`
- **Packet framing ใน KCP stream**: `HEAD 0x9D74C714 + cmd(u16 BE) + headLen + bodyLen + head + body + TAIL 0xD7A152C8` — ไม่มี XOR encryption ใน build นี้
- **cmdid map** generate จาก `// CmdID:` comments ใน proto → `protocol/proto/cmdid.go` (regenerate ตอนเปลี่ยน proto)
- **ใช้ data files Rust PS ได้ทันที** (freesr-data.json / persistent / res.json / teleports.json / challenge_data.json / mission_unlocks.json / versions.json) — ฟอร์แมต JSON ตรงกัน รวม enum string อย่าง `main_character: "FemaleHarmony"`
- **Build**: `go build -o gameserver-go.exe ./cmd/gameserver` + `./cmd/sdkserver`; ทดสอบ: `go test ./...`; เปิดด้วย `run_go.bat` (UDP 23301 + HTTP 21000 เหมือนเดิม)
- **ที่ตั้ง skill library**: `tools/AI TOOLBOX/` (ทั้งสองโปรเจกต์มี copy — อัปเดตต้นฉบับที่เดียวแล้วก็อป)

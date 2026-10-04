---
name: hkrpg-ps-version-update
description: คู่มืออัปเดต Honkai Star Rail Private Server (GaC-hkrpg-ps) ให้รองรับ client เวอร์ชันใหม่ เรียนรู้จากการอัปเดต 4.5.x → 4.6.51 ใช้เมื่อ client อัปเดตแล้วเซิร์ฟเวอร์เก่ามีอาการ จอดำ/ค้างหน้าโหลด/packet Unknown/dialogue ค้าง/เกม crash
---

# คู่มืออัปเดต HSR Private Server ข้ามเวอร์ชัน client

เอกสารนี้สรุปบทเรียนจากการอัปเดตจริง 4.5.x → 4.6.51 (beta) ทั้งกระบวนการวินิจฉัย สิ่งที่ต้องแก้ต่อเวอร์ชัน และกับดักที่พบ

---

## 1. ลำดับการวินิจฉัยเมื่อ client ใหม่เข้าไม่ได้

อ่าน log ตามลำดับนี้ **ก่อนแก้โค้ด** เพราะอาการเดียวกันมีหลายต้นเหตุ:

| อาการ | แหล่งข้อมูล | สิ่งที่มองหา |
|---|---|---|
| ค้างก่อนหน้า login | client hook log | URL ที่ถูก redirect/block |
| จอดำหลังกด login | `Player.log` (LocalLow/Cognosphere/Star Rail) | `NullReferenceException` + ชื่อ handler `_CmdXxxScRsp` + **offset** (offset ต่างกัน = field คนละตัวใน handler เดียวกัน) |
| จอดำ/ค้างหลัง login finish | `Player.log` + log หน้าต่าง Game Server | packet ที่ client ส่งต่อแต่เซิร์ฟ "Unknown command ID" |
| ค้างหน้าโหลดฉาก | `Player.log` | `CreateMemberEntityLeader FAILED`, `LuaException` (GotoMapParam ฯลฯ), ExcelTable FAILED |
| เกมดับ | `Temp/Cognosphere/Star Rail/Crashes` | crash report |
| จุดวาป/คุยผิดปกติ | `server_debug.log` ในโฟลเดอร์เซิร์ฟ | บรรทัด `ENTER_SCENE` / `INTERACT_PROP` (บันทึก entry_id/teleport_id/prop จริง) |

กฎสำคัญ: **`Player.log` ถูกเขียนทับทุกครั้งที่เปิดเกม** — ต้องอ่านทันทีหลังทดสอบ และอย่าสรุปจาก error ของ session เก่าที่จำ offset ไว้

## 2. Checklist สิ่งที่ต้องอัปเดตต่อเวอร์ชัน client

1. **Proto/cmd dump ใหม่** — dump จาก client ด้วย Morax (`morax dump-proto --game-path <game_dir>`) หรือใช้ dump จาก AstralOS รุ่นใหม่
2. **`cmd_ids_<ver>.json`** — mapping cmd ทั้งหมดของรุ่นนั้น (cmd id ของ response เลื่อนบ่อย ดูหัวข้อ 3)
3. **`full_dummy` map** — regenerate `gameserver/src/net/full_dummy.rs` จาก proto ใหม่ (CsReq→ScRsp ทุกคู่) เพื่อให้ทุก cmd มีการตอบ ไม่มี "Unknown" ตกหล่น
4. **Data files จาก AstralOS dump** (ใช้ได้ — เป็น data จาก client ไม่ใช่โค้ด): `challenge_data.json` (challenges/tierce/stages/peak), `res.json` (level_output_configs), `teleports.json`, `mission_unlocks.json`
5. **`BASE_AVATAR_IDS`** — เพิ่ม avatar ตัวใหม่ของรุ่นนั้น (เทียบกับ robinsr reference รุ่นเดียวกัน เช่น 4651 มี 89 ตัว มี 1511)
6. **`versions.json`** — hotfix URLs ต่อเวอร์ชัน (ดูหัวข้อ 5)
7. **ระบบใหม่ที่ client เรียก** — ดู cmd ใหม่ใน dump เช่น tutorial, NPC taken reward, ChallengePeak

## 3. Cmd ID และ response layout เปลี่ยนทุกรุ่น

- **ScRsp id เลื่อนตำแหน่ง**: `StartChallengeScRsp` เคยเป็น 1775 → 4.6.51 กลายเป็น **1787** และ 1775 ไปเป็น `GetChallengeGroupStatisticsCsReq` — ส่ง response ผิด id = client รอไม่ได้ หมุนค้างตลอด
- **Field tag เลื่อน**: `StartChallengeCsReq.challenge_id` tag 12 → **tag 14** — decoder มือต้องอัปเดต
- **Response body layout เปลี่ยน**: tag ของ retcode/scene/lineup ใน ScRsp ต้องแกะใหม่จาก proto ทุกรุ่น
- ตรวจเสมอด้วย `cmd_ids_<ver>.json` (มีใน repo) + proto dump ใหม่
- cmd ที่ไม่มีใน dump (เช่น 1452, Anomaly Arbitration) = เนื้อหาหลัง beta build ที่ client ถือ — ยอมรับว่าทำไม่ได้จนกว่าจะมี dump ใหม่

## 4. Optional message fields — สาเหตุจอดำที่ไล่ยากที่สุด

`_CmdGetBasicInfoScRsp` NRE วนซ้ำเพราะ `PlayerSettingInfo` มี optional message ซ่อนหลายตัว (`ojpaodihaje` Tag 13, `ankidjjinei` Tag 1171, `cmfcooeeecb` Tag 1552) — ปล่อย None ผ่าน `..Default::default()` แล้ว client unwrap → NRE → จอดำ

วิธีไล่: NRE offset ใน Player.log ต่างกัน = field คนละตัว ใส่ทีละตัวจนหมด ใช้ `field_overrides_<build>.json` ช่วยแปลชื่อ

## 5. Design data CDN — ต้นเหตุ "ตารางพัง/ค้างหน้าโหลด"

- `query_gateway` response มี `asset_bundle_url` / `ex_resource_url` / `lua_url` — มาจาก `versions.json` (sdkserver fetch จาก CDN จริงผ่าน proxy)
- client beta จะ **hot-update design data** จาก URL เหล่านี้ — ถ้า CDN push ชุดใหม่กว่า GameAssembly ที่ติดตั้ง → ExcelTable โหลด FAILED → `PlayerModule.Init` พัง → ค้างหน้าโหลด (ดู D number บนหน้าโหลด: D16700702 ≠ D16706787)
- **แก้**: `ex_resource_url` ต้องชี้ `design_data/BetaLive/output_<D ที่ตรงกับ client>` (ตัวอย่าง 4.6.51: `https://autopatchcn.bhsr.com/design_data/BetaLive/output_16700702_...` — ดูตัวอย่างจาก PS ที่เวิร์ก เช่น firefly) — **ห้ามชี้ /no_update ปล่อยว่าง** เพราะ client beta ไม่ fallback ใช้ local จะฟ้อง "resource download failed"
- ไฟล์ที่โหลดมาแล้วพังจะค้างใน `StreamingAssets/DesignData/Windows/` — ต้อง restore จาก `.bak` หรือ re-verify

## 6. กับดักที่พบจริงใน 4.6.51 (เก็บไว้เจอซ้ำ)

- **ทีมเกิน 4 ตัว**: client ส่งทีม 2 ช่องทาง (`avatar_lineup_first` tag 4 + `first_lineup` tag 6) — รวมกันได้ 8 ตัว → `TeamManager._CreateMemberEntity` IndexOutOfRange → crash ตอนเข้า arena — แก้ด้วย dedupe + truncate(4)
- **InteractProp**: ต้องเก็บ mapping entity_id → prop ตอนโหลดฉาก แล้วตอบ `prop_state` **จริงจาก config** — ห้ามตอบ `interact_id2` (มันคือ interaction id เช่น 1100 ไม่ใช่ state 0-8)
- **GetNpcTakenReward (2174)**: client ถามทุกครั้งก่อนคุย NPC — ไม่ตอบ = dialogue ไม่มีตัวเลือกและค้าง
- **Tutorial**: ต้อง mark ทุก id ว่าดูแล้ว (`GetTutorial` 1674, `GetTutorialGuide` 1631 — status enum `Oekdmamgidc`=2) ไม่งั้น popup เด้งวนและ**บล็อก input เดินไม่ได้**
- **arena ใหม่ไม่มีใน client เก่า**: challenge phase ใหม่ (เช่น 5312) ใช้ arena 3014101 ซึ่ง client build เก่าไม่มี MapEntrance → Lua `GotoMapParam: attempt to index nil with 'ID'` — fallback ไป arena มาตรฐาน 3000101
- **Scene ไม่มี teleport**: ฉาก arena ไม่มี teleport ใน res — fallback จาก `teleports.json` หรือ spawn ใกล้ prop แรกหันหน้าเข้าหา
- **full coverage**: ตอบ ScRsp ว่างให้ทุก CsReq ที่มีคู่ (865 คู่ใน 4.6.51) — กันค้างจาก packet ที่ client รอโดยไม่รู้ตัว ทำใน fallback `_` arm ของ on_message
- **full coverage ตอบทุก cmd = อันตราย!** — การตอบ ScRsp ว่างให้ "ทุก" cmd ทำให้ client modules (`GridFightModule`, `ChessRogueModule`, `EvolveBuildModule`, `ChimeraDuelModule`, `MusicAlbumModule/Jukebox`) รับข้อมูลว่างแล้ว `Sync(nil)` → NRE วน → **game state machine ไม่ไปต่อ → จอดำ** — cmd ที่ไม่รู้จักต้อง **drop เงียบๆ** (client โหลดต่อได้ปกติ) ตอบได้เฉพาะ cmd ที่รู้ว่า client รับ empty ได้ (ทดสอบทีละตัว)
- **BASE_AVATAR_IDS ห้ามใส่ตัวที่ client build นี้ไม่มี** — id จาก reference รุ่นใหม่กว่า (เช่น 1511 จาก build 4651) ทำให้ client ไม่มี config ตัวนั้น → พังตอน init โลก (จอดำก่อนถึง Pom-Pom)
- **BASE_AVATAR_IDS ขาดตัวใหม่** = battle/arena crash (เช่น 1511 ใน 4.6)

## 7. เครื่องมือและแหล่งข้อมูล

- **AstralOS dump (ใช้ได้ ไม่ก็อปโค้ด)**: `challenge_data.json`, `stages.json`, `teleports.json`, `res.json`, `mission_unlocks.json` — data ที่ dump จาก client ครบกว่าเดิม
- **`field_overrides_<build>.json`** — ดัชนีแปลชื่อ field obfuscated → ชื่อจริงของทุก message
- **`pkg_version` + `StreamingAssets/Asb`** — ไฟล์ client เป็น hash ทั้งหมด ตรวจ asset ทางอ้อมไม่ได้ ใช้ Player.log เป็นหลัก
- **`server_debug.log`** — บันทึก ENTER_SCENE/INTERACT_PROP (เพิ่ม `scene_debug_log` แล้ว) อ่านเองได้ไม่ต้องขอ user ก๊อป console

## 8. กระบวนการทำงานที่พิสูจน์แล้วว่าเร็ว

1. ทดสอบ 1 อย่าง → อ่าน log ทันที → แก้ 1 จุด → build → ทดสอบซ้ำ (อย่าแก้หลายจุดโดยไม่มี log ยืนยัน)
2. ถ้าต้องการ reference จาก PS อื่นที่เวิร์ก: **อ่านเพื่อเทียบได้ ห้ามก็อปโค้ดทั้งดุ้น** — ดึงเฉพาะ data dump และค่า config
3. **commit ทุกครั้งที่แก้เสร็จ** — ไฟล์เคยถูกย้อนกลับโดยไม่ทราบสาเหตุ 2 ครั้ง (mod.rs/packet.rs และทั้งโปรเจกต์) งานหายทั้งชุดถ้าไม่ commit
4. แก้ path hardcode ใน .bat ทุกครั้งที่ย้าย/rename โฟลเดอร์ (tools.bat เคย hardcode path เกมเวอร์ชันเก่า)
5. เปลี่ยน versions.json ทีละ field ด้วย script — อย่าแก้มือตอนเซิร์ฟเวอร์กำลังรัน

## 9. สถานะที่รู้ว่ายังไม่สำเร็จ (ณ 4.6.51)

- Anomaly Arbitration — ไม่มี cmd ใน dump ของ client รุ่นนี้
- Currency War (ตระกูล Rogue 101 cmd) — ตอบ default ครบแล้ว แต่เล่นได้จริงต้อง implement state เฉพาะไล่ตาม log
- Starward Mode — มี handler (GetChallengePeakData 8942 / StartChallengePeak 8948 ใช้ peak data จริง) ต้องทดสอบต่อ
- tutorial ID list เป็นช่วง 1-1000 (estimate) — ถ้ายังมี popup ต้องหา id จริงจาก design data

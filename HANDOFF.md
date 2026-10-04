# HANDOFF: HSR Private Server 4.6.51 — สถานะปัจจุบันและปัญหาค้าง (สำหรับส่งต่อ AI อื่น)

วันที่: 2026-10-05 | เขียนโดย: ZCode session ก่อนหน้า

---

## 1. สภาพแวดล้อม

- **โปรเจกต์ (Rust PS)**: `C:\Users\Phitchayut\Desktop\GaC-hkrpg-ps` (เพิ่ง rename จาก Hoyo-hkrpg-PS)
  - gameserver (KCP :23301) + sdkserver (HTTP :21000), `run.bat` มีเมนู 1/2/3, `tools.bat`
- **Client**: `C:\Users\Phitchayut\Desktop\StarRail_4.6.51_OS` (beta OSBETAWin4.6.51, GameAssembly รุ่นแรกของ 4.6.51, design data ติดตั้ง = D16700702)
- **Client log**: `C:\Users\Phitchayut\AppData\LocalLow\Cognosphere\Star Rail\Player.log` (**ถูกเขียนทับทุกครั้งที่เปิดเกม — ต้องอ่านทันทีหลังทดสอบ**)
- **Crash report**: `C:\Users\Phitchayut\AppData\Local\Temp\Cognosphere\Star Rail\Crashes`
- **Server debug log**: `server_debug.log` ใน root โปรเจกต์ (บันทึก ENTER_SCENE / INTERACT_PROP / GET_NPC_TAKEN_REWARD)
- **AstralOS dump ใหม่**: `C:\Users\Phitchayut\Downloads\New folder` (bin/*.json = data dump จาก client 4651, tools/reference/* = proto/cmdids/field_overrides 4651)
- **PS อื่นที่เวิร์ก**: Firefly Shelter (`C:\Program Files\Firefly Shelter\server\firefly-go`) — client เข้าเกมได้ผ่าน Firefly (หลังรอโหลด resource)
- **ข้อห้าม**: ห้ามรัน StarRail.exe / ห้ามแตะ `C:\Users\Phitchayut\Desktop\C#` / ห้ามก็อปโค้ด AstralOS ทั้งดุ้น (ใช้ data dump ได้, อ่าน reference เพื่อเทียบได้)

## 2. ไฟล์ที่แก้ไขทั้งหมด (สถานะปัจจุบันใน GaC-hkrpg-ps)

| ไฟล์ | การแก้ไข |
|---|---|
| `gameserver/src/net/handlers/player.rs` | GetBasicInfo: PlayerSettingInfo มี ojpaodihaje(Tag13)+ankidjjinei(Tag1171)+cmfcooeeecb(Tag1552) — แก้ NRE `_CmdGetBasicInfoScRsp` (offset 0xe501b3d→0xe501b33 = field คนละตัว); on_get_tutorial_cs_req + on_get_tutorial_guide_cs_req ส่ง id 1..=1000 status=2 (กัน popup tutorial) |
| `gameserver/src/net/handlers/mission.rs` | แก้ main/sub mission สลับกัน |
| `gameserver/src/net/handlers/lineup.rs` | แก้ on_replace_lineup_cs_req (ใช้ item.slot/item.id) + เพิ่ม swap/switch/getlineupavatar handlers |
| `gameserver/src/net/handlers/avatar.rs` | BASE_AVATAR_IDS = 88 ตัว (**1511 ถูกถอนออก** — client beta รุ่นแรกไม่มี config ตัวนี้ สงสัยว่าทำให้จอดำ); GetAvatarData รองรับ skin |
| `gameserver/src/net/handlers/challenge.rs` | ChallengeConfigData + PeakStageData (peak field); GetChallengeScRsp: taken_stars_count_reward ต่อกลุ่ม + kkiafpfklge ปลดทุกกลุ่ม + ดาว=7 + คะแนน (PF 80000, AS/5xxx 4000); decode_start_challenge_req แบบ 4.6 (challenge_id tag 14, first_lineup tag 6, second_lineup tag 8, avatar_lineup_first tag 4, stage_info tag 10); handle_start_challenge ตอบ **cmd 1787** layout ใหม่ (retcode tag3, cur_challenge tag5, lineup tag6, scene tag10); ทีม dedupe+truncate(4); arena จริงตาม data + fallback 3000101 ถ้าไม่มีใน res; on_get_challenge_peak_data ใช้ CHALLENGE_DATA.peak (fallback dummy peaks ถ้า data เก่าไม่มี peak); on_start_challenge_peak_cs_req (cmd 8948 → arena 3000101 group 2 + EnterSceneByServerScNotify 1427) |
| `gameserver/src/net/handlers/scene.rs` | scene_debug_log → server_debug.log; prop_entities mapping (entity_id → prop_id/inst/state); on_interact_prop ตอบ prop_state จริง (ห้ามตอบ interact_id2 เช่น 1100); load_scene fallback spawn: teleport ใน res → TELEPORT_DB → first prop; on_get_npc_taken_reward_cs_req (ตอบ npc_id+retcode0 — แก้ dialogue ค้าง) |
| `gameserver/src/net/session.rs` | PlayerSession.prop_entities; sync_player แนบ dressed_skins |
| `gameserver/src/net/full_dummy.rs` | แผนที่ CsReq→ScRsp 865 คู่ (generate จาก proto) — **ปัจจุบันไม่ได้ใช้** (full coverage ทำให้จอดำ — ดูหัวข้อ 5) |
| `common/src/structs/persistent.rs` | dressed_skins: HashMap<u32,u32> |
| `common/src/sr_tools.rs` | FreesrData.dressed_skins + load/save |
| `common/src/structs/avatar.rs` | to_avatar_path_data_proto รับ dressed_skin_id |
| `common/src/resources.rs` | skin_ids config + TELEPORT_DB (teleports.json) |
| `gameserver/src/net/handlers/avatar.rs` (tutorial) | อยู่ใน player.rs ไม่ใช่ avatar.rs |
| **Data files (root)** | `res.json` = ใหม่ 13MB จาก AstralOS (+skinIds 8 ตัว); `teleports.json` = ใหม่ (ทุกแมพ); `challenge_data.json` = **ย้อนกลับเป็นเวอร์ชัน git HEAD** (819 challenges, ไม่มี peak — เพราะสงสัยว่า 843 records ทำให้จอดำ); `versions.json` = OSBETAWin4.6.51 ใช้ URL จาก Firefly (autopatchcn.bhsr.com, ex_resource_url = design_data/BetaLive/output_16700702_5084ff364848_ebdcee37054070); `persistent` มี dressed_skins field |
| `run.bat` | เมนู 1/2/3 (server+proxy/server+GayProxy/server only) — auto-find game dir 6 ชั้น (registry → Desktop wildcard → drive scan) — แก้ pattern 4.5.52 → 4.6.51 แล้ว |
| `tools.bat` | เพิ่ม GAME_DIR `%USERPROFILE%\Desktop\StarRail_4.6.51_OS` เป็นลำดับแรก, เปลี่ยนชื่อโปรเจกต์ |
| `SKILL.md` | คู่มือบทเรียนทั้งหมด (อัปเดตแล้ว: full coverage อันตราย, 1511 ห้ามใส่) |

**ไบนารี**: `gameserver.exe` 2,898,432 bytes (build ล่าสุด — ยังไม่ได้ทดสอบโดย user หลัง revert challenge_data)

## 3. ไทม์ไลน์อาการ (สำคัญมาก — อ่านก่อนเดา)

1. **เคยไปถึง Pom-Pom ได้** (session ~15:0x): เข้าฉากได้, tutorial เด้งและใช้ได้, กดคุย prop บนรถไฟได้, เปิด Endgame UI ได้ (ส่งภาพ 4 ใบมา) — **แต่เดินไม่ได้** (สงสัย tutorial modal บล็อก input)
2. จากนั้น user ทดสอบ start challenge 5512 → ค้างหน้าโหลด (ตอนนั้น arena 3014101 + ทีม 8 ตัวจาก decode bug + response 1775 ผิด id)
3. หลังแก้ decoder/1787/ทีม cap 4 + เพิ่ม 1511 + res ใหม่ + challenge_data ใหม่ → **จอดำกลับมาหลัง login** (ไม่ถึง Pom-Pom)
4. หลังถอน 1511 + ตัด full coverage + ย้อน challenge_data → **user รายงานยังจอดำอยู่** (build 2,898,432 — อาจยังไม่ได้ทดสอบรอบล่าสุด)

## 4. ข้อเท็จจริงที่พิสูจน์แล้ว (อย่าเดาซ้ำ)

- ฉากรถไฟ P10000_F10000000 ใน res.json เก่า/ใหม่ **เหมือนกันเป๊ะ** (groups=11 props=14 npcs=47 teleports=2) — res.json ไม่ใช่ต้นเหตุจอดำ
- `cmd 1452` **ไม่ใช่ command** — ชนกับ enum SceneType_KNDDOLEHBKE ใน proto (client รุ่นนี้ใช้ id ที่ไม่มีใน dump) — ตอบไม่ได้และไม่ใช่ต้นเหตุ
- Anomaly Arbitration **ไม่มี cmd ใน dump ทั้งสองรุ่น** (เก่า+4651) — ทำไม่ได้จนกว่าจะมี dump ใหม่จาก client
- `GetNpcTakenReward` (cmd 2174) ต้องตอบ — ไม่งั้น dialogue NPC ไม่มีตัวเลือกและค้าง (แก้แล้ว)
- Tutorial ต้อง mark finished — ไม่งั้น popup เด้งและบล็อก input (แก้แล้ว — popup หายตามที่ user ยืนยัน)
- `StartChallengeScRsp` ของ 4.6.51 = **cmd 1787** (1775 ใน 4.6 = GetChallengeGroupStatisticsCsReq) — response layout: retcode tag3, cur_challenge tag5, lineup_list tag6, scene tag10
- `StartChallengeCsReq.challenge_id` = **tag 14** (tag 12 = รุ่นเก่า)
- NRE จาก empty response (full coverage): GridFightModule, CycleScore, ChessRogueModule, EvolveBuildModule, ChimeraDuelModule, MusicAlbumModule(Jukebox) — **cmd พวกนี้ต้อง drop เงียบๆ ห้ามตอบว่าง**
- Firefly PS (เวิร์ก) ใช้ `ex_resource_url` = `https://autopatchcn.bhsr.com/design_data/BetaLive/output_16700702_5084ff364848_ebdcee37054070` (design data ตรง client)
- client โหลด design data ใหม่ (D16706787) จาก autopatchos.starrails.com ทำให้ ExcelTable โหลด FAILED → ค้างหน้าโหลดรถไฟ (เคยเกิด — แก้ด้วย ex_resource_url แล้ว)
- ตระกูล Currency War = Rogue (101 cmd: EnterRogue 1805, StartRogue 1831, GetRogueInfo 1874 ฯลฯ) — ตอบ default ครบแล้วแต่ยังเล่นไม่ได้จริง

## 5. อาการค้างล่าสุดที่ยังไม่แก้ (จุดเริ่มของ Gemini)

**จอดำหลัง login** — client ส่ง packet ครบ (GetCurSceneInfo 1459 → 1418 → PlayerLoginFinish 9 → 7548+13 → GetChallenge 1774 → GetChallengePeakData 8942 → ...) แล้วนิ่ง ไม่มี SceneEntityMove (1474) — client ไม่ยอมเริ่มโหลดฉาก

สมมติฐานที่ยังไม่ได้ทดสอบ (เรียงตามความน่าจะเป็น):
1. **build 2,898,432 (ล่าสุด) ยังไม่ได้ทดสอบจริง** — ทดสอบก่อน แล้วดู Player.log
2. **GetChallengeScRsp 856 records** (จาก challenge_data เก่าที่ยังใช้อยู่) — อาจยังใหญ่ไป / มี id ที่ client ไม่รู้จัก → ลองตัดเหลือเฉพาะ phase ปัจจุบัน
3. **Tutorial 1000 entries** — ลองลดเหลือ empty เทียบกัน (แยกตัวแปร)
4. **res.json ใหม่ 13MB** — ลองย้อนเป็น res เก่าจาก git (`git show HEAD:res.json`) เทียบกัน
5. **SceneInfo ของรถไฟขาด field ใหม่ของ 4.6** — เทียบกับ field_overrides_4651.json (SceneInfo/SceneActorInfo message) หา field ที่ client 4651 ต้องการ

## 6. ขั้นตอนแนะนำสำหรับ AI ตัวถัดไป

1. ตรวจว่าเซิร์ฟเวอร์รันจาก build 2,898,432 จริง (`gameserver.exe` ขนาดตรง) และ Firefly ปิดอยู่
2. ให้ user เปิดเกม → login → ถ้าจอดำ: อ่าน `Player.log` ทันที — แยกแยะ: มี NRE ที่ handler ไหน / มี AdventurePhase ไหม (0 = client ไม่เริ่มโหลดฉาก)
3. แยกตัวแปรทีละอย่าง (อย่าแก้หลายจุดพร้อมกัน):
   a. GetChallengeScRsp → ส่งเฉพาะ challenge_list ช่วง phase ปัจจุบัน (5301-5312) ไม่ส่ง 856 ตัว
   b. GetTutorial/GetTutorialGuide → ตอบ empty (ไม่ใช่ 1000 entries)
   c. ย้อน res.json เป็น git HEAD
4. ถ้า client เริ่มโหลดฉากได้ (เห็น AdventurePhase ใน Player.log) แต่ค้างระหว่างโหลด → ดู `CreateMemberEntityLeader` / LuaException แล้วแก้ตามจุด
5. เรื่องเสาวาป PF: ให้ user กด Teleport แล้วอ่าน `ENTER_SCENE` ใน server_debug.log (entry_id + teleport_id) แล้ว hardcode จุดที่ถูก
6. **commit ทุกครั้ง** — ไฟล์ถูกย้อนกลับมาแล้ว 3 ครั้ง (git add -A && git commit)

## 7. ไฟล์อ้างอิงที่ใช้ได้ (ห้ามก็อปโค้ดทั้งดุ้น — อ่านเทียบได้)

- `C:\Users\Phitchayut\Downloads\New folder\tools\reference\robinsr_4651_StarRail.proto` — proto ย่อ 4651 (มี ChallengeBossEquipmentInfo/RelicInfo/AvatarRelicInfo ใหม่)
- `robinsr_4651_cmdids.json` — 150 cmd ที่ robinsr ใช้กับ 4651
- `field_overrides_4651.json` — **ดัชนีแปลชื่อ field obfuscated → ชื่อจริงทุก message** (ล้ำค่าสุด)
- `bin/stages.json` (2.2MB), `bin/teleports.json` (712KB), `bin/res.json` (13MB), `bin/challenge_data.json` (มี peak)
- PS อ้างอิงที่เวิร์ก: Firefly Shelter (firefly-go) — client เข้าได้แน่นอน

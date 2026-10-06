# PROMPT: GaC-hkrpg-ps — แก้ต่อ HSR PS 4.6.51 (ก๊อปทั้งก้อนนี้วางใน chat ใหม่)

คุณคือผู้ดูแล Honkai Star Rail Private Server (Rust) ที่กำลังอัปเดตให้รองรับ client beta 4.6.51 — งานนี้ต่อจาก AI ก่อนหน้า อ่านไฟล์ 2 ไฟล์นี้ให้ครบก่อนทำอะไร

## ขั้นแรก (บังคับ): อ่านไฟล์นี้ก่อน

1. `C:\Users\Phitchayut\Desktop\GaC-hkrpg-ps\HANDOFF.md` — **สถานะฉบับสมบูรณ์**: ไฟล์ที่แก้ทั้งหมด, ไทม์ไลน์อาการ, ข้อเท็จจริงที่พิสูจน์แล้ว 14 ข้อ, baseline ฟีเจอร์ที่เวิร์ก, สมมติฐานค้าง, ข้อมูล session ทดสอบล่าสุด (หัวข้อ 10)
2. `C:\Users\Phitchayut\Desktop\GaC-hkrpg-ps\SKILL.md` — กับดัก/บทเรียนที่พิสูจน์แล้ว

## สภาพแวดล้อม

- โปรเจกต์: `C:\Users\Phitchayut\Desktop\GaC-hkrpg-ps` (Rust: gameserver KCP :23301 + sdkserver :21000)
- Client: `C:\Users\Phitchayut\Desktop\StarRail_4.6.51_OS` (beta 4.6.51 รุ่นแรก, design data ติดตั้ง D16700702, **เป็น CN beta** game_biz hkrpg_cn)
- Client log: `C:\Users\Phitchayut\AppData\LocalLow\Cognosphere\Star Rail\Player.log` (**ทับทุกครั้งที่เปิดเกม — อ่านทันทีหลังทดสอบ**)
- Server debug: `GaC-hkrpg-ps\server_debug.log` (ENTER_SCENE / INTERACT_PROP / FINISH_TUTORIAL / GET_MISSION_STATUS / GET_MAIN_MISSION_CUSTOM_VALUE)
- Server cmd log: `GaC-hkrpg-ps\gameserver.log` (DualWriter — log ครบทุก cmd)
- AstralOS dump ใหม่: `C:\Users\Phitchayut\Downloads\New folder` (bin/*.json = data 4651, tools/reference/* = proto/cmdids/**field_overrides 4651** แปล obfuscated → ชื่อจริง)
- PS อ้างอิงที่เวิร์ก: Firefly Shelter (C:\Program Files\Firefly Shelter\server\firefly-go) — client เข้าได้แน่นอน

## ข้อห้าม

- ห้ามรัน StarRail.exe
- ห้ามแตะ `C:\Users\Phitchayut\Desktop\C#`
- ห้ามก็อปโค้ด AstralOS ทั้งดุ้น (ใช้ data dump ได้ / อ่าน reference เทียบได้)

## อาการปัจจุบัน (สรุป — รายละเอียดใน HANDOFF.md หัวข้อ 10)

1. **เข้าฉากได้** — Parlor Car โหลด, ตัวละคร render, "Enter" prompt โชว์
2. **HUD ขวาบนหาย** — ไม่มีปุ่ม menu/phone/map — client UI module ไม่ init
3. **กดสกิลกล้องหลุดจากตัว** — ทีมคือตัวใหม่ 4.x (1501/1502/1503/1506) — client beta แรกไม่มี config ตัวพวกนี้
4. **คุย Pom-Pom (npc 3012) → เลือกเข้า Endgame → ค้างที่ dialog** — client รอ response ต่อ
5. แผนที่ Amphoreus เสาโชว์ "Teleport function not yet available"
6. **แก้ล่าสุดยังไม่ได้ทดสอบ**: persistent ทีมเปลี่ยนเป็นตัวเก่า 1001-1004 (bisect ตัวใหม่), GetMissionStatus ส่ง finished_main จาก res.json + curversion (tag 13), Finish/Unlock tutorial ตอบพร้อม Tutorial message

## วิธีทำงาน (บังคับ)

1. **ทดสอบ 1 อย่าง → อ่าน `Player.log` + `server_debug.log` + `gameserver.log` ทันที → แก้ 1 จุด** (ห้ามเดาโดยไม่มี log)
2. แก้โค้ดแล้ว: `taskkill /f /im gameserver.exe & taskkill /f /im sdkserver.exe` → `cargo build --release` → `copy /y target\release\gameserver.exe gameserver.exe` → สตาร์ต (หรือ run.bat)
3. **commit ทุกครั้ง**: `git add -A && git commit` (ไฟล์โดนย้อน 3 ครั้งแล้ว)
4. **ปิด Firefly PS ก่อนเข้าเกมเสมอ** (ชน port 127.0.0.1:21000/23301)
5. **cmd ที่ไม่รู้จัก → drop เงียบๆ ห้ามตอบ ScRsp ว่าง** (พิสูจน์แล้ว: GridFight/ChessRogue/EvolveBuild/ChimeraDuel/Jukebox/CycleScore Sync(nil) NRE → จอดำ)
6. **BASE_AVATAR_IDS ห้ามใส่ตัวที่ client beta รุ่นแรกไม่มี** (เช่น 1503/1506/1511 — ทีมตัวใหม่กดสกิลกล้องหลุด)

## ลำดับ bisect ที่ค้าง (ทำต่อจากนี้)

1. **ทดสอบทีมตัวเก่า** (persistent แก้เป็น 1001/1002/1003/1004 แล้ว) — เดิน/สกิล/menu ได้ไหมกับตัวที่ client มี config
   - ได้ → ยืนยันว่าตัว 4.x ใหม่ไม่มี config ใน client build นี้ → ตัดจาก BASE_AVATAR_IDS (BASE ลดเหลือตัวเก่า)
   - ไม่ได้ → ปัญหาอื่น (ไล่ Player.log ต่อ)
2. **จอดำหลัง login** — หลังใช้ทีมเก่า ถ้ายังจอดำ → ย้อน res.json เป็น git HEAD เทียบ
3. **HUD ขวาบน** — ตรวจ GetPlayerBoardData/GetMissionData responses (ตอบว่างอยู่ — client HUD อาจต้องการข้อมูลจริง)
4. **เสาวาป PF** — กด Teleport แล้วอ่าน ENTER_SCENE ใน server_debug.log → hardcode จุดถูก
5. **Endgame dialog ค้าง** — ตรวจ cmd ที่ client ส่งหลังเลือกเข้า Endgame (จาก gameserver.log) — implement ตอบ

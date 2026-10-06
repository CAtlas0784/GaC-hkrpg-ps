# คำสั่งสำหรับ GLM: แก้ไขบั๊กค้างหน้า PomPom และจอดำ (Black Screen) ใน Client 4.6.51

จาก Commit `45bbb73` (bug fix + version 4.6.51) พบว่าโค้ดทำให้ Client รุ่น `CNBETAWin4.6.51` ติดปัญหาจอดำค้างที่หน้าโหลด PomPom ให้ตรวจสอบและแก้ไขตาม 6 ข้อนี้ทั้งหมด:

---

### 1. ไฟล์ `versions.json` (SDK Server Hotfix Config)
* **ปัญหา**: Client ส่ง version เป็น `CNBETAWin4.6.51` แต่ไฟล์มีเฉพาะ `OSBETAWin4.6.51` ทำให้ Client เกิด Error `-1005: load cur env BETA_CN server config failed` และไม่โหลดต่อ
* **วิธีแก้**: เพิ่มคอนฟิก `CNBETAWin4.6.51` ต่อท้ายเวอร์ชันเดิม:
```json
  "CNBETAWin4.6.51": {
    "asset_bundle_url": "https://autopatchcn.bhsr.com/asb/BetaLive/output_16677734_6b334c11ab3a_653e489328e98e",
    "ex_resource_url": "https://autopatchcn.bhsr.com/design_data/BetaLive/output_16706787_90982cd003eb_5ed57e0de4e2b4",
    "lua_url": "https://autopatchcn.bhsr.com/lua/BetaLive/output_16679499_5abbd35a74c5_a28e83e88b78d5",
    "ifix_url": "https://autopatchcn.bhsr.com/ifix/BetaLive/output_0_40d2ce0253_c61ba99f70b885"
  },
```

---

### 2. ไฟล์ `common/src/structs/avatar.rs` & `gameserver/src/net/handlers/avatar.rs` (คืนค่า Aha 1511)
* **ปัญหา**: เผลอตัด Avatar ID ของตัวละครใหม่ออก ทำให้ตัวละครหายหรือโหลดผิดพลาด
* **วิธีแก้**: ใน `BASE_AVATAR_IDS` ต้องมี ID ตัวละคร 4.6.51 ครบถ้วน:
  * `1511` (Aha)
  * `1512`, `1513`
  * ตรวจสอบว่า `1511` สามารถแสดงผลใน Avatar List และเข้าจัดทีม Lineup ได้ปกติ

---

### 3. ไฟล์ `gameserver/src/net/packet.rs` (Opcode และ Full Dummy Handling)
* **ปัญหา**:
  1. มีการส่งแพ็กเก็ตคู่ (Dual-send) ของ Opcode 4.5 เก่าที่ 4.6.51 ตัดทิ้งแล้ว
  2. การตอบ Dummy ScRsp เปล่า (Empty Response) ให้กับ Req ที่ไม่รู้จัก ทำให้โค้ด C# เกิด NullReferenceException (NRE) และ Game State Machine ค้าง
* **วิธีแก้**:
  * **ตัดการส่ง Opcode เก่าของ 4.5 ทิ้งทั้งหมด**:
    * ห้ามส่ง CmdID `1736` (ให้ส่งเฉพาะ `1708` ตอบคู่กับ Req `1775`)
    * ห้ามส่ง CmdID `8982` (ให้ส่งเฉพาะ `8981` ตอบคู่กับ Req `8993`)
    * ห้ามส่ง CmdID `8994` (ให้ส่งเฉพาะ `8996` ตอบคู่กับ Req `8995`)
  * **ห้ามตอบ ScRsp เปล่าให้กับระบบที่ยังไม่ได้ทำ**:
    * โมดูลเช่น `GridFight` (8441..8795), `ChessRogue` (5402..5595), `EvolveBuild` (7103..7130), `ChimeraDuel` (9213..9255), Jukebox (`3174`), และ `CycleScore` (4143, 4146)
    * **ให้ Drop ทิ้ง (ไม่ต้องตอบกลับ)** เหมือนใน AstralOS ซึ่งจะไม่ทำให้ Client ค้าง NRE

---

### 4. ไฟล์ `gameserver/src/net/handlers/challenge.rs` (Abyss Group IDs & Peak Snapshot)
* **ปัญหา**:
  1. มีการสร้าง Group ID ช่วง `1..99` ปลอม ทำให้เกิด Error `深渊分组数据不存在`
  2. การตอบ `GetCurChallengePeakScRsp` ใส่ `peak_id: 1` ปลอม ทำให้ Client แครชที่ `CreateMemberEntityLeader`
* **วิธีแก้**:
  * Abyss Group ID ให้ส่งเฉพาะ ID จริง: MoC (`1001..=1035`), PF (`2001..=2025`), AS (`3001..=3020`), และ Forgotten Hall (`100..=119`, `900`)
  * ใน `on_get_cur_challenge_peak_cs_req`: ถ้าผู้เล่นไม่ได้เล่น Peak ให้ตอบแค่ `res.retcode = 0` ว่างๆ ห้ามใส่ `peak_id: 1`

---

### 5. ไฟล์ `gameserver/src/net/handlers/scene.rs` (UID Entity และการ Spawn ฉาก)
* **ปัญหา**: `SceneActorInfo.uid` มีการใส่ค่าเป็น `25` ทำให้ Client ไม่ยอมรับ Entity ตัวละครหลักของผู้เล่น
* **วิธีแก้**:
  * แก้ไข `SceneActorInfo.uid` ให้เป็น `1337` (ตรงกับ Session UID ของ Player)
  * จุดเกิดเริ่มต้นใช้ Astral Express Parlor Car:
    * `entry_id: 1000101`, `plane_id: 10001`, `floor_id: 10001001` (หรือ `100000104` / `10000`)

---

### 6. ไฟล์ `gameserver/src/net/handlers/authentication.rs` (Sanitize Login)
* **วิธีแก้**: ตรวจสอบลำดับ Auth ให้สมบูรณ์:
  * `PlayerGetTokenScRsp` (71): `retcode: 0`, `msg: "OK"`, `uid: 1337`
  * `PlayerLoginScRsp` (19): `retcode: 0`, `cur_timezone: 8`, timestamps ครบถ้วน
  * `PlayerLoginFinishScRsp` (13): `retcode: 0`

use std::sync::OnceLock;
use super::*;

#[derive(serde::Deserialize, Default, Clone, Debug)]
pub struct MissionUnlocks {
    #[serde(rename = "funcUnlockMissions", default)]
    pub func_unlock_missions: Vec<u32>,
    #[serde(rename = "allMainMissions", default)]
    pub all_main_missions: Vec<u32>,
    #[serde(rename = "teleportIds", default)]
    pub teleport_ids: Vec<u32>,
    #[serde(rename = "entranceIds", default)]
    pub entrance_ids: Vec<u32>,
    #[serde(rename = "raidIds", default)]
    pub raid_ids: Vec<u32>,
    #[serde(rename = "challengeRaidIds", default)]
    pub challenge_raid_ids: Vec<u32>,
    #[serde(rename = "npcIds", default)]
    pub npc_ids: Vec<u32>,
    #[serde(rename = "subMissionIds", default)]
    pub sub_mission_ids: Vec<u32>,
    #[serde(rename = "questIds", default)]
    pub quest_ids: Vec<u32>,
}

pub static MISSION_UNLOCKS: OnceLock<MissionUnlocks> = OnceLock::new();

pub fn get_mission_unlocks() -> &'static MissionUnlocks {
    MISSION_UNLOCKS.get_or_init(|| {
        match std::fs::read_to_string("mission_unlocks.json") {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(data) => {
                    tracing::info!("Loaded mission_unlocks.json successfully");
                    data
                }
                Err(e) => {
                    tracing::error!("Failed to parse mission_unlocks.json: {e}");
                    MissionUnlocks::default()
                }
            },
            Err(e) => {
                tracing::warn!("mission_unlocks.json not found: {e}");
                MissionUnlocks::default()
            }
        }
    })
}

pub async fn on_get_mission_status_cs_req(
    _session: &mut PlayerSession,
    body: &GetMissionStatusCsReq,
    res: &mut GetMissionStatusScRsp,
) {
    res.retcode = 0;
    let unlocks = get_mission_unlocks();

    // main missions ที่จบแล้ว — รวมจาก GAME_RES + funcUnlockMissions + client requested mains
    // (client Lua อ่าน mission progress ตอนเข้าแมพ/ปลดฟังก์ชัน — ถ้าไม่ส่งจะล็อค HUD, เสาวาป, และ Endgame)
    let mut finished: Vec<u32> = common::resources::GAME_RES
        .level_output_configs
        .values()
        .flat_map(|m| m.values())
        .flat_map(|sc| sc.scenes.values())
        .flat_map(|s| s.finished_main_missions.iter().copied())
        .collect();

    finished.extend(unlocks.func_unlock_missions.iter().copied());
    finished.extend(body.main_mission_id_list.iter().copied());
    finished.sort_unstable();
    finished.dedup();
    res.finished_main_mission_id_list = finished.clone();
    // curversion finished — client beta ถามซ้ำถ้าไม่ส่ง field นี้ (tag 13)
    res.curversion_finished_main_mission_id_list = finished;

    // sub missions → SubMissionStatusList (tag 12) พร้อมสถานะ MissionFinish
    res.sub_mission_status_list = body
        .sub_mission_id_list
        .iter()
        .map(|id| Mission {
            id: *id,
            progress: 1,
            status: MissionStatus::MissionFinish.into(),
        })
        .collect();

    crate::net::handlers::scene_debug_log(&format!(
        "GET_MISSION_STATUS main_req={} sub_req={} finished_sent={}",
        body.main_mission_id_list.len(),
        body.sub_mission_id_list.len(),
        res.finished_main_mission_id_list.len()
    ));
}

// cmd 1274 → 1219: GetMissionDataScRsp
pub async fn on_get_mission_data_cs_req(
    _session: &mut PlayerSession,
    _body: &GetMissionDataCsReq,
    res: &mut GetMissionDataScRsp,
) {
    res.retcode = 0;
    let unlocks = get_mission_unlocks();
    let mut finished = unlocks.func_unlock_missions.clone();
    finished.sort_unstable();
    finished.dedup();

    res.main_mission_list = finished
        .iter()
        .map(|&id| MainMission {
            id,
            status: MissionStatus::MissionFinish.into(),
            custom_value_list: Vec::new(),
        })
        .collect();
    res.finished_main_mission_id_list = finished;

    crate::net::handlers::scene_debug_log(&format!(
        "GET_MISSION_DATA finished_sent={}",
        res.finished_main_mission_id_list.len()
    ));
}

// cmd 974 → 919: GetQuestDataScRsp
pub async fn on_get_quest_data_cs_req(
    _session: &mut PlayerSession,
    _body: &GetQuestDataCsReq,
    res: &mut GetQuestDataScRsp,
) {
    res.retcode = 0;
    let unlocks = get_mission_unlocks();
    res.quest_list = unlocks
        .quest_ids
        .iter()
        .map(|&id| Quest {
            id,
            status: QuestStatus::QuestFinish.into(),
            progress: 1,
            finish_time: 0,
            ebbfioiopad: Vec::new(),
        })
        .collect();

    crate::net::handlers::scene_debug_log(&format!(
        "GET_QUEST_DATA quests_sent={}",
        res.quest_list.len()
    ));
}

// cmd 1289 → 1250: สถานะละเอียดของ main missions (รวม custom values ตัวแปรเนื้อเรื่อง)
// client Lua (_GetActiveGroupInfoByMissionCustomValue) อ่านจาก response นี้ — ถ้า drop จะบล็อก input
pub async fn on_get_main_mission_custom_value_cs_req(
    _session: &mut PlayerSession,
    body: &GetMainMissionCustomValueCsReq,
    res: &mut GetMainMissionCustomValueScRsp,
) {
    res.retcode = 0;
    // ตอบ MainMission ของทุก id ที่ถาม: status = MissionFinish, custom_value_list ว่าง
    res.main_mission_list = body
        .sub_mission_id_list
        .iter()
        .map(|&id| MainMission {
            status: MissionStatus::MissionFinish.into(),
            id,
            custom_value_list: Vec::new(),
        })
        .collect();

    crate::net::handlers::scene_debug_log(&format!(
        "GET_MAIN_MISSION_CUSTOM_VALUE ids={}",
        body.sub_mission_id_list.len()
    ));
}

// cmd 1281 → 1222: ติดตาม mission (echo track_mission_id กลับ)
pub async fn on_update_track_main_mission_cs_req(
    _session: &mut PlayerSession,
    body: &UpdateTrackMainMissionCsReq,
    res: &mut UpdateTrackMainMissionScRsp,
) {
    res.retcode = 0;
    res.track_mission_id = body.track_mission_id;
}

// cmd 2774 → 2719: message groups ของ NPC contact (ส่งว่าง — ไม่มีระบบแชท)
pub async fn on_get_npc_message_group_cs_req(
    _session: &mut PlayerSession,
    _body: &GetNpcMessageGroupCsReq,
    res: &mut GetNpcMessageGroupScRsp,
) {
    res.retcode = 0;
}

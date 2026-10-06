use super::*;

pub async fn on_get_mission_status_cs_req(
    session: &mut PlayerSession,
    body: &GetMissionStatusCsReq,
    res: &mut GetMissionStatusScRsp,
) {
    res.retcode = 0;

    // main missions ที่จบแล้ว — รวมจากทุกฉากใน res.json
    // (client Lua อ่าน mission progress ตอนเข้าแมพ — ถ้าไม่ส่ง จะ NRE และบล็อก input)
    let mut finished: Vec<u32> = common::resources::GAME_RES
        .level_output_configs
        .values()
        .flat_map(|m| m.values())
        .flat_map(|sc| sc.scenes.values())
        .flat_map(|s| s.finished_main_missions.iter().copied())
        .collect();
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

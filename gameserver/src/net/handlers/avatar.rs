use super::*;
use common::resources::GAME_RES;
// หากมีการอัพเดท ของตัวละครใหม่ ๆ ให้เพิ่ม ID ของตัวละครเหล่านั้นใน BASE_AVATAR_IDS
// หมายเหตุ: ห้ามใส่ avatar ที่ client build นี้ไม่มี (เช่น 1511 ของ beta รุ่นหลัง)
// — client จะไม่มี config ของตัวนั้นและพังตอน init โลก (จอดำหลัง login)
pub const BASE_AVATAR_IDS: [u32; 88] = [
    8001, 1001, //
    //
    1002, 1003, 1004, 1005, 1006, 1008, 1009, 1013, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1108,
    1109, 1110, 1111, 1112, 1201, 1202, 1203, 1204, 1205, 1206, 1207, 1208, 1209, 1210, 1211, 1212,
    1213, 1214, 1215, 1217, 1301, 1302, 1303, 1304, 1305, 1306, 1307, 1308, 1309, 1312, 1315, 1310,
    1314, 1218, 1221, 1220, 1222, 1223, 1317, 1313, 1225, 1402, 1401, 1404, 1403, 1405, 1407, 1406,
    1409, 1014, 1015, 1408, 1410, 1412, 1413, 1414, 1415, 1321, 1501, 1502, 1503, 1504, 1505, 1506,
    1507, 1508, 1509, 1510, 1512, 1513,
];

pub async fn on_get_avatar_data_cs_req(
    session: &mut PlayerSession,
    body: &GetAvatarDataCsReq,
    res: &mut GetAvatarDataScRsp,
) {
    let Some(json) = session.json_data.get() else {
        tracing::error!("data is not set!");
        return;
    };

    res.is_get_all = body.is_get_all;
    res.avatar_list = BASE_AVATAR_IDS
        .into_iter()
        .map(|id| {
            json.avatars
                .get(&id)
                .map(|v| {
                    v.to_avatar_proto(
                        json.lightcones.iter().find(|v| v.equip_avatar == id),
                        json.main_character as u32,
                        json.march_type as u32,
                    )
                })
                .unwrap_or(Avatar {
                    base_avatar_id: id,
                    level: 80,
                    promotion: 6,
                    first_met_timestamp: 1712924677,
                    cur_multi_path_avatar_type: 0,
                    equipment_unique_id: 0,
                    has_taken_promotion_reward_list: vec![1, 3, 5],
                    is_marked: false,
                    exp: 0,
                })
        })
        .collect();

    res.avatar_path_data_info_list = json
        .avatars
        .values()
        .map(|avatar| {
            avatar.to_avatar_path_data_proto(
                json.lightcones
                    .iter()
                    .find(|l| l.equip_avatar == avatar.avatar_id),
                json.relics
                    .iter()
                    .filter(|r| r.equip_avatar == avatar.avatar_id)
                    .collect(),
                json.dressed_skins
                    .get(&avatar.avatar_id)
                    .copied()
                    .unwrap_or(0),
            )
        })
        .collect();

    // skin ที่ปลดล็อก: จาก res.json (skinIds) + ชุดที่ avatar ใส่อยู่
    res.skin_list = json
        .dressed_skins
        .values()
        .copied()
        .chain(GAME_RES.skin_ids.iter().copied())
        .collect::<std::collections::BTreeSet<u32>>()
        .into_iter()
        .collect();
}

pub async fn on_dress_avatar_skin_cs_req(
    session: &mut PlayerSession,
    req: &DressAvatarSkinCsReq,
    _res: &mut DressAvatarSkinScRsp,
) {
    let Some(player) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    player
        .dressed_skins
        .insert(req.avatar_id, req.avatar_skin_id);
    player.save_persistent().await;
}

pub async fn on_take_off_avatar_skin_cs_req(
    session: &mut PlayerSession,
    req: &TakeOffAvatarSkinCsReq,
    _res: &mut TakeOffAvatarSkinScRsp,
) {
    let Some(player) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    player.dressed_skins.remove(&req.avatar_id);
    player.save_persistent().await;
}

pub async fn on_set_player_outfit_cs_req(
    _session: &mut PlayerSession,
    _req: &SetPlayerOutfitCsReq,
    _res: &mut SetPlayerOutfitScRsp,
) {
    // retcode defaults to 0
}

pub async fn on_take_promotion_reward_cs_req(
    _session: &mut PlayerSession,
    _req: &TakePromotionRewardCsReq,
    _res: &mut TakePromotionRewardScRsp,
) {
    // retcode defaults to 0, reward_list defaults to empty
    // All avatars already report has_taken_promotion_reward_list = [0..=6] in GetAvatarData
}

pub async fn on_set_avatar_enhanced_id_cs_req(
    _session: &mut PlayerSession,
    _req: &SetAvatarEnhancedIdCsReq,
    _res: &mut SetAvatarEnhancedIdScRsp,
) {
    // let Some(json) = session.json_data.get_mut() else {
    //     return;
    // };
    // let Some(avatar) = json.avatars.get_mut(&req.avatar_id) else {
    //     return;
    // };
    // avatar.enhanced_id = if req.enhanced_id == 0 {
    //     Option::<u32>::None
    // } else {
    //     Some(req.enhanced_id)
    // };
    // res.growth_avatar_id = avatar.avatar_id;
    // res.unk_enhanced_id = req.enhanced_id;
}

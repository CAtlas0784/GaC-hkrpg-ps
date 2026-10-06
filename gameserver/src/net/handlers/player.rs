use tokio::fs;

use super::*;

pub async fn on_get_basic_info_cs_req(
    _session: &mut PlayerSession,
    _body: &GetBasicInfoCsReq,
    res: &mut GetBasicInfoScRsp,
) {
    res.retcode = 0;
    res.gender = Gender::Woman as u32;
    res.is_gender_set = true;
    res.player_setting_info = Some(PlayerSettingInfo {
        ojpaodihaje: Some(Pnolliakgdb {
            belkcbagmfc: false,
            jhjkcjkdhib: 0,
            mlpgdpgifcj: false,
        }),
        // optional message fields ที่ client 4.6.51 อ่านต่อ — ปล่อย None จะ NRE ที่ _CmdGetBasicInfoScRsp
        ankidjjinei: Some(Ljfjogeegpj {
            mbiaimkhndb: false,
        }),
        cmfcooeeecb: Some(Jpgackplgcd {
            kalacjbapoa: false,
            fokjipgmkan: false,
            iblgocfboco: false,
        }),
        ..Default::default()
    });
}

pub async fn on_player_heart_beat_cs_req(
    _session: &mut PlayerSession,
    body: &PlayerHeartBeatCsReq,
    res: &mut PlayerHeartBeatScRsp,
) {
    res.client_time_ms = body.client_time_ms;
    res.server_time_ms = body.client_time_ms;

    // สคริปต์ข้อความด้านบน (ดึงจาก player.rs ตามเดิม)
    let top_watermark_script = r#"
local function beta_text()
    local gameObject = CS.UnityEngine.GameObject.Find("UIRoot/AboveDialog/BetaHintDialog(Clone)")
    if gameObject then
        local textComponent = gameObject:GetComponentInChildren(typeof(CS.RPG.Client.LocalizedText))
        if textComponent then
            textComponent.text = "<color=#FF2D00><b>If you read this U R GAY </b></color>"
        end
    end
end
beta_text()
"#;

    let mut final_data = top_watermark_script.as_bytes().to_vec();

    // ดึงไฟล์ FreeCam จาก scripts/freecam.lua มาต่อท้าย
    if let Ok(freecam_data) = fs::read("scripts/freecam.lua").await {
        final_data.extend_from_slice(b"\n");
        final_data.extend_from_slice(&freecam_data);
    }

    res.download_data = Some(ClientDownloadData {
        version: 51,
        time: res.server_time_ms as i64,
        data: final_data,
        ..Default::default()
    });
}

// หากมีการอัพเดท package ใหม่ ๆ ให้เพิ่ม ID ของ package เหล่านั้นใน ContentPackageSyncDataScNotify
pub async fn on_player_login_finish_cs_req(
    session: &mut PlayerSession,
    _req: &PlayerLoginFinishCsReq,
    res: &mut PlayerLoginFinishScRsp,
) -> Result<()> {
    res.retcode = 0;
    session
        .send(ContentPackageSyncDataScNotify {
            data: Some(ContentPackageData {
                content_package_list: [
                    200001, 200002, 200003, 200004, 200005, 200006, 200007, 200008, 200009, 200010,
                    200011, 200012, 200013, 200014, 150017, 150015, 150021, 150018, 130011, 130012,
                    130013, 150025, 140006, 150026, 130014, 150034, 150029, 150035, 150041, 150039,
                    150045, 150057, 150042, 150067, 150064, 150063, 150024, 171002, 150068, 150070,
                    150071, 150073, 150074, 150075, 150076, 150077, 150078, 150079, 150132, 150134,
                    150135,
                ]
                .into_iter()
                .map(|v| ContentPackageInfo {
                    status: ContentPackageStatus::Finished.into(),
                    content_id: v,
                })
                .collect(),
                ..Default::default()
            }),
        })
        .await?;

    Ok(())
}

pub async fn on_get_tutorial_cs_req(
    _session: &mut PlayerSession,
    _req: &GetTutorialCsReq,
    res: &mut GetTutorialScRsp,
) {
    // mark ทุก tutorial ว่าดูแล้ว — กันหน้าต่างสอนเล่นเด้งขึ้นมาเอง (และไม่บล็อก input)
    res.retcode = 0;
    res.tutorial_list = (1..=1000)
        .map(|id| Tutorial {
            id,
            status: Hhjgmfealeb::Oekdmamgidc.into(),
        })
        .collect();
}

pub async fn on_get_tutorial_guide_cs_req(
    _session: &mut PlayerSession,
    _req: &GetTutorialGuideCsReq,
    res: &mut GetTutorialGuideScRsp,
) {
    res.retcode = 0;
    res.tutorial_guide_list = (1..=1000)
        .map(|id| TutorialGuide {
            id,
            status: Hhjgmfealeb::Oekdmamgidc.into(),
            r#type: 0,
        })
        .collect();
}

// ตอบกลับพร้อม Tutorial message สถานะ finished — client จะปลด input หลังปิดหน้าต่างสอนเล่น
pub async fn on_finish_tutorial_cs_req(
    _session: &mut PlayerSession,
    req: &FinishTutorialCsReq,
    res: &mut FinishTutorialScRsp,
) {
    crate::net::handlers::scene_debug_log(&format!(
        "FINISH_TUTORIAL id={} type={}", req.hpimdinccnn, req.hfiblhafonm
    ));
    res.retcode = 0;
    res.tutorial = Some(Tutorial {
        id: req.hpimdinccnn,
        status: Hhjgmfealeb::Oekdmamgidc.into(),
    });
}

pub async fn on_finish_tutorial_guide_cs_req(
    _session: &mut PlayerSession,
    req: &FinishTutorialGuideCsReq,
    res: &mut FinishTutorialGuideScRsp,
) {
    crate::net::handlers::scene_debug_log(&format!(
        "FINISH_TUTORIAL_GUIDE type={} group_id={}", req.r#type, req.group_id
    ));
    res.retcode = 0;
    res.tutorial_guide = Some(TutorialGuide {
        id: req.group_id,
        status: Hhjgmfealeb::Oekdmamgidc.into(),
        r#type: req.r#type,
    });
}

pub async fn on_unlock_tutorial_cs_req(
    _session: &mut PlayerSession,
    req: &UnlockTutorialCsReq,
    res: &mut UnlockTutorialScRsp,
) {
    crate::net::handlers::scene_debug_log(&format!(
        "UNLOCK_TUTORIAL id={}", req.hpimdinccnn
    ));
    res.retcode = 0;
    res.tutorial = Some(Tutorial {
        id: req.hpimdinccnn,
        status: Hhjgmfealeb::Oekdmamgidc.into(),
    });
}

pub async fn on_unlock_tutorial_guide_cs_req(
    _session: &mut PlayerSession,
    req: &UnlockTutorialGuideCsReq,
    res: &mut UnlockTutorialGuideScRsp,
) {
    res.retcode = 0;
    res.tutorial_guide = Some(TutorialGuide {
        id: req.group_id,
        status: Hhjgmfealeb::Oekdmamgidc.into(),
        r#type: req.r#type,
    });
}

pub async fn on_sync_client_res_version_cs_req(
    _session: &mut PlayerSession,
    req: &SyncClientResVersionCsReq,
    res: &mut SyncClientResVersionScRsp,
) {
    res.retcode = 0;
    res.ljkmpimkomo = req.ljkmpimkomo;
}

// cmd 2874 → 2819: GetPlayerBoardDataScRsp
pub async fn on_get_player_board_data_cs_req(
    _session: &mut PlayerSession,
    _body: &GetPlayerBoardDataCsReq,
    res: &mut GetPlayerBoardDataScRsp,
) {
    res.retcode = 0;
    res.current_head_icon_id = 200001;
    res.current_personal_card_id = 253001;
    res.signature = String::from("Trailblazer");
    res.jjddekiaofc = vec![200001, 200002, 200101, 200102, 200103, 200104];
    res.kjaabdabljj = vec![253001, 253002];
    crate::net::handlers::scene_debug_log("GET_PLAYER_BOARD_DATA sent");
}

// cmd 5174 → 5119: GetPhoneDataScRsp
pub async fn on_get_phone_data_cs_req(
    _session: &mut PlayerSession,
    _body: &GetPhoneDataCsReq,
    res: &mut GetPhoneDataScRsp,
) {
    res.retcode = 0;
    res.cur_phone_theme = 1;
    res.cur_phone_case = 200001;
    res.cur_chat_bubble = 220001;
    res.owned_phone_themes = vec![1];
    res.owned_phone_cases = vec![200001];
    res.owned_chat_bubbles = vec![220001];
    crate::net::handlers::scene_debug_log("GET_PHONE_DATA sent");
}

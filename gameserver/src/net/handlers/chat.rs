use common::structs::MultiPathAvatar;
use proto::chat_data::ExtendType;
use std::path::Path;
use tokio::fs;

use crate::{
    net::PlayerSession,
    util::{self, cur_timestamp_ms},
};

use super::*;

const SERVER_UID: u32 = 727;
const SERVER_HEAD_ICON: u32 = 201402;
const SERVER_CHAT_BUBBLE_ID: u32 = 220005;
const SERVER_CHAT_HISTORY: &[&str] = &[
    "'mode {aa/moc/pf/as/su/default}' set battle mode (Anomaly Arbitration, MoC, etc.)",
    "'stage {stage_id}' set stage ID",
    "'cycle {count}' set battle cycle limit",
    "'status' view current battle mode and stage settings",
    "'lua {path_to_lua_script}' execute lua script",
    "'sw {on/off}' enable/disable silver wolf global buff",
    "'castorice {on/off}' enable/disable castorice global buff",
    "'sync' to synchronize stats between json and in-game view",
    "'mc {mc_id}' mc_id can be set from 8001 to 8008",
    "'march {march_id}' march_id can be set 1001 or 1224",
    "available commands:",
    "visit srtools.neonteam.dev to configure the PS! (you configure relics, equipment, monsters from there)",
];

pub async fn on_get_friend_login_info_cs_req(
    _session: &mut PlayerSession,
    _req: &GetFriendLoginInfoCsReq,
    res: &mut GetFriendLoginInfoScRsp,
) {
    res.black_uid_list = vec![SERVER_UID];
    res.friend_uid_list = vec![SERVER_UID];
}

pub async fn on_get_friend_list_info_cs_req(
    _session: &mut PlayerSession,
    _req: &GetFriendListInfoCsReq,
    res: &mut GetFriendListInfoScRsp,
) {
    res.friend_list = vec![FriendSimpleInfo {
        remark_name: String::from("RobinSR"),
        player_info: Some(PlayerSimpleInfo {
            uid: SERVER_UID,
            platform: PlatformType::Pc.into(),
            online_status: FriendOnlineStatus::Online.into(),
            head_icon: SERVER_HEAD_ICON,
            chat_bubble_id: SERVER_CHAT_BUBBLE_ID,
            level: 70,
            nickname: String::from("Server"),
            signature: String::from("omg"),
            ..Default::default()
        }),
        is_marked: true,
        create_time: 0,
        ..Default::default()
    }];
}

pub async fn on_get_private_chat_history_cs_req(
    _session: &mut PlayerSession,
    req: &GetPrivateChatHistoryCsReq,
    res: &mut GetPrivateChatHistoryScRsp,
) {
    let cur_time = cur_timestamp_ms();
    res.chat_message_list = SERVER_CHAT_HISTORY
        .iter()
        .map(|text| ChatMessageData {
            create_time: cur_time,
            ckhpffenobe: Some(Eknabklpeel {
                kpobmnlklok: 1,
                role_id: SERVER_UID,
            }),
            bkoalkhdlob: Some(Eknabklpeel {
                kpobmnlklok: 1,
                role_id: SERVER_UID,
            }),
            message_datas: vec![MessageChatData {
                message_type: 1,
                chat_data: Some(ChatData {
                    extend_type: Some(ExtendType::MessageText(text.to_string())),
                }),
            }],
            ..Default::default()
        })
        .collect();
    res.target_side = req.target_side;
    res.contact_side = SERVER_UID;
}

pub async fn on_send_msg_cs_req(
    session: &mut PlayerSession,
    body: &SendMsgCsReq,
    _res: &mut SendMsgScRsp,
) {
    let Some(json) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    let msg = body
        .message_datas
        .as_ref()
        .and_then(|x| x.chat_data.as_ref())
        .and_then(|x| x.extend_type.as_ref())
        .and_then(|x| match x {
            ExtendType::MessageText(s) => Some(s.as_str()),
            _ => Option::<&str>::None,
        })
        .unwrap_or("");

    if let Some((cmd, args)) = parse_command(msg) {
        match cmd {
            "sync" => {
                let _ = session.sync_player().await;
                session
                    .send(create_send_message(
                        25,
                        SERVER_UID,
                        body.message_datas
                            .as_ref()
                            .map(|v| v.message_type)
                            .unwrap_or_default(),
                        body.chat_type,
                        String::from("Inventory Synced"),
                    ))
                    .await
                    .unwrap();
            }
            "sw" | "castorice" => {
                let status = args.first().unwrap_or(&"on").to_lowercase();
                let enabled = match status.as_str() {
                    "on" | "1" | "true" => true,
                    "off" | "0" | "false" => false,
                    _ => true,
                };

                if cmd == "sw" {
                    json.enable_sw_global = Some(enabled);
                } else {
                    json.enable_castorice_global = Some(enabled);
                }

                json.save_persistent().await;

                session
                    .send(create_send_message(
                        25,
                        SERVER_UID,
                        body.message_datas
                            .as_ref()
                            .map(|v| v.message_type)
                            .unwrap_or_default(),
                        body.chat_type,
                        format!(
                            "{} Global Buff: {}",
                            if cmd == "sw" { "SW" } else { "Castorice" },
                            if enabled { "Enabled" } else { "Disabled" }
                        ),
                    ))
                    .await
                    .unwrap();
            }
            "mc" => {
                let mc = MultiPathAvatar::from(
                    args.first()
                        .unwrap_or(&"")
                        .parse::<u32>()
                        .unwrap_or(json.main_character as u32),
                );

                json.main_character = mc;
                json.save_persistent().await;

                session
                    .send(AvatarPathChangedNotify {
                        base_avatar_id: 8001,
                        cur_multi_path_avatar_type: mc as i32,
                    })
                    .await
                    .unwrap();

                let _ = session.sync_player().await;

                session
                    .send(create_send_message(
                        25,
                        SERVER_UID,
                        body.message_datas
                            .as_ref()
                            .map(|v| v.message_type)
                            .unwrap_or_default(),
                        body.chat_type,
                        format!("Success change mc to {mc:#?}"),
                    ))
                    .await
                    .unwrap();
            }
            "march" => {
                let mut march_type = MultiPathAvatar::from(
                    args.first()
                        .unwrap_or(&"")
                        .parse::<u32>()
                        .unwrap_or(json.march_type as u32),
                );

                if march_type != MultiPathAvatar::MarchPreservation
                    && march_type != MultiPathAvatar::MarchHunt
                {
                    march_type = MultiPathAvatar::MarchHunt
                }

                json.march_type = march_type;
                json.save_persistent().await;

                session
                    .send(AvatarPathChangedNotify {
                        base_avatar_id: 1001,
                        cur_multi_path_avatar_type: march_type as i32,
                    })
                    .await
                    .unwrap();

                session
                    .send(create_send_message(
                        25,
                        SERVER_UID,
                        body.message_datas
                            .as_ref()
                            .map(|v| v.message_type)
                            .unwrap_or_default(),
                        body.chat_type,
                        format!("Success change march to {march_type:#?}"),
                    ))
                    .await
                    .unwrap();
            }
            "lua" => {
                let path = Path::new(args.first().unwrap_or(&""));

                if !path.is_file() {
                    session
                        .send(create_send_message(
                            25,
                            SERVER_UID,
                            body.message_datas
                                .as_ref()
                                .map(|v| v.message_type)
                                .unwrap_or_default(),
                            body.chat_type,
                            format!("File {path:?} does not exist!"),
                        ))
                        .await
                        .unwrap();
                }

                let data = match fs::read(&path).await {
                    Ok(file) => file,
                    Err(err) => {
                        session
                            .send(create_send_message(
                                25,
                                SERVER_UID,
                                body.message_datas
                                    .as_ref()
                                    .map(|v| v.message_type)
                                    .unwrap_or_default(),
                                body.chat_type,
                                format!("Failed to read file: {err:?}"),
                            ))
                            .await
                            .unwrap();

                        return;
                    }
                };

                session
                    .send(ClientDownloadDataScNotify {
                        download_data: Some(ClientDownloadData {
                            version: 51,
                            time: util::cur_timestamp_ms() as i64,
                            data,
                            ..Default::default()
                        }),
                    })
                    .await
                    .unwrap();

                session
                    .send(create_send_message(
                        25,
                        SERVER_UID,
                        body.message_datas
                            .as_ref()
                            .map(|v| v.message_type)
                            .unwrap_or_default(),
                        body.chat_type,
                        format!("Executed {path:?}"),
                    ))
                    .await
                    .unwrap();
            }
            "mode" | "battle" => {
                let mode_str = args.first().unwrap_or(&"").to_lowercase();
                let new_mode = match mode_str.as_str() {
                    "aa" => Some(common::structs::BattleType::AA),
                    "moc" => Some(common::structs::BattleType::Moc),
                    "pf" => Some(common::structs::BattleType::PF),
                    "as" => Some(common::structs::BattleType::AS),
                    "su" => Some(common::structs::BattleType::SU),
                    "default" => Some(common::structs::BattleType::Default),
                    _ => None,
                };

                let response_msg = if let Some(mode) = new_mode {
                    json.battle_config.battle_type = mode.clone();
                    format!("Battle mode set to: {mode:?}")
                } else {
                    format!(
                        "Current mode: {:?}. Available: aa, moc, pf, as, su, default",
                        json.battle_config.battle_type
                    )
                };

                session
                    .send(create_send_message(
                        25,
                        SERVER_UID,
                        body.message_datas
                            .as_ref()
                            .map(|v| v.message_type)
                            .unwrap_or_default(),
                        body.chat_type,
                        response_msg,
                    ))
                    .await
                    .unwrap();
            }
            "stage" => {
                let response_msg = if let Some(Ok(stage_id)) = args.first().map(|s| s.parse::<u32>()) {
                    json.battle_config.stage_id = stage_id;
                    format!("Stage ID set to: {stage_id}")
                } else {
                    format!("Current Stage ID: {}", json.battle_config.stage_id)
                };

                session
                    .send(create_send_message(
                        25,
                        SERVER_UID,
                        body.message_datas
                            .as_ref()
                            .map(|v| v.message_type)
                            .unwrap_or_default(),
                        body.chat_type,
                        response_msg,
                    ))
                    .await
                    .unwrap();
            }
            "cycle" | "cycles" => {
                let response_msg = if let Some(Ok(cycles)) = args.first().map(|s| s.parse::<u32>()) {
                    json.battle_config.cycle_count = cycles;
                    format!("Cycle limit set to: {cycles}")
                } else {
                    format!("Current Cycle limit: {}", json.battle_config.cycle_count)
                };

                session
                    .send(create_send_message(
                        25,
                        SERVER_UID,
                        body.message_datas
                            .as_ref()
                            .map(|v| v.message_type)
                            .unwrap_or_default(),
                        body.chat_type,
                        response_msg,
                    ))
                    .await
                    .unwrap();
            }
            "status" => {
                let status_msg = format!(
                    "Battle Status:\n- Mode: {:?}\n- Stage ID: {}\n- Cycles: {}",
                    json.battle_config.battle_type,
                    json.battle_config.stage_id,
                    json.battle_config.cycle_count
                );

                session
                    .send(create_send_message(
                        25,
                        SERVER_UID,
                        body.message_datas
                            .as_ref()
                            .map(|v| v.message_type)
                            .unwrap_or_default(),
                        body.chat_type,
                        status_msg,
                    ))
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }
}

fn parse_command(command: &str) -> Option<(&str, Vec<&str>)> {
    let parts: Vec<&str> = command.split_whitespace().collect();

    if parts.is_empty() {
        return Option::None;
    }

    Some((parts[0], parts[1..].to_vec()))
}

fn create_send_message(
    to: u32,
    from: u32,
    message_type: i32,
    chat_type: i32,
    msg: String,
) -> RevcMsgScNotify {
    RevcMsgScNotify {
        chat_type,
        pffpfkoglpo: to,
        msg: Some(ChatMessageData {
            create_time: cur_timestamp_ms(),
            ckhpffenobe: Some(Eknabklpeel {
                kpobmnlklok: 1,
                role_id: from,
            }),
            bkoalkhdlob: Some(Eknabklpeel {
                kpobmnlklok: 1,
                role_id: from,
            }),
            message_datas: vec![MessageChatData {
                message_type,
                chat_data: Some(ChatData {
                    extend_type: Some(chat_data::ExtendType::MessageText(msg)),
                }),
            }],
            ..Default::default()
        }),
    }
}

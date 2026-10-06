use common::{
    resources::GAME_RES,
    structs::{AvatarJson, Position},
};
use scene_entity_info::Entity;

use crate::util::{self};

use super::*;

/// เขียน log สำคัญลงไฟล์เพื่อวิเคราะห์พฤติกรรม client (วาป/คุย)
pub fn scene_debug_log(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("server_debug.log")
    {
        let ts = util::cur_timestamp_ms();
        let _ = writeln!(f, "[{ts}] {msg}");
    }
}

pub async fn on_get_cur_scene_info_cs_req(
    session: &mut PlayerSession,
    _body: &GetCurSceneInfoCsReq,
    res: &mut GetCurSceneInfoScRsp,
) {
    let Some(player) = session.json_data.get() else {
        tracing::error!("data is not set!");
        return;
    };

    // If player logged out or got stuck inside a challenge arena, recover back to Parlor Car
    let entry_id = if player.scene.entry_id >= 3000000 && player.scene.entry_id < 4000000 {
        100000104
    } else {
        player.scene.entry_id
    };

    let default_scene = SceneInfo {
        game_mode_type: 1,
        entry_id,
        plane_id: if entry_id == 100000104 { 10000 } else { player.scene.plane_id },
        floor_id: if entry_id == 100000104 { 10000000 } else { player.scene.floor_id },
        ..Default::default()
    };

    let scene = load_scene(session, default_scene.entry_id, false, Option::<u32>::None, None).await;

    res.retcode = 0;
    res.scene = if let Ok(scene) = scene {
        Some(scene)
    } else {
        Some(default_scene)
    };
}

pub async fn on_enter_scene_cs_req(
    session: &mut PlayerSession,
    req: &EnterSceneCsReq,
    res: &mut EnterSceneScRsp,
) {
    tracing::info!(
        "on_enter_scene_cs_req: entry_id={}, teleport_id={}, interact_id={}, scene_identifier={:?}",
        req.entry_id, req.teleport_id, req.interact_id, req.scene_identifier
    );
    scene_debug_log(&format!(
        "ENTER_SCENE entry_id={} teleport_id={} interact_id={} scene_identifier={:?}",
        req.entry_id, req.teleport_id, req.interact_id, req.scene_identifier
    ));

    let resolved_entry_id = if req.entry_id != 0 && GAME_RES.level_output_configs.contains_key(&req.entry_id) {
        req.entry_id
    } else if let Some(&e) = req.scene_identifier.as_ref().and_then(|si| GAME_RES.map_default_entrance_map.get(&si.floor_id)) {
        e
    } else if let Some(&e) = GAME_RES.map_default_entrance_map.get(&req.entry_id) {
        e
    } else if req.teleport_id != 0 {
        let mut found_entry = None;
        for (&eid, map) in &GAME_RES.level_output_configs {
            for sc in map.values() {
                for grp in sc.scenes.values() {
                    if grp.teleports.contains_key(&req.teleport_id) {
                        found_entry = Some(eid);
                        break;
                    }
                }
                if found_entry.is_some() { break; }
            }
            if found_entry.is_some() { break; }
        }
        found_entry.unwrap_or_else(|| {
            session.json_data.get().map(|p| p.scene.entry_id).unwrap_or(100000104)
        })
    } else if req.interact_id != 0 {
        let mut found_entry = None;
        let id32 = req.interact_id as u32;
        for (&eid, map) in &GAME_RES.level_output_configs {
            for sc in map.values() {
                for grp in sc.scenes.values() {
                    if grp.props.iter().any(|p| p.inst_id == id32 || p.prop_id == id32) {
                        found_entry = Some(eid);
                        break;
                    }
                }
                if found_entry.is_some() { break; }
            }
            if found_entry.is_some() { break; }
        }
        found_entry.unwrap_or_else(|| {
            session.json_data.get().map(|p| p.scene.entry_id).unwrap_or(100000104)
        })
    } else {
        session.json_data.get().map(|p| p.scene.entry_id).unwrap_or(100000104)
    };

    let teleport_id = if req.teleport_id != 0 {
        Some(req.teleport_id)
    } else if let Some(scene_identifier::TeleportNigger::Mdaidppkopo(t)) =
        req.scene_identifier.as_ref().and_then(|si| si.teleport_nigger.as_ref())
    {
        if t.plidnbmcijh != 0 {
            Some(t.plidnbmcijh)
        } else {
            None
        }
    } else if req.interact_id != 0 {
        let id32 = req.interact_id as u32;
        GAME_RES.level_output_configs.get(&resolved_entry_id).and_then(|map| {
            map.values().find_map(|sc| {
                for grp in sc.scenes.values() {
                    if grp.props.iter().any(|p| p.inst_id == id32 || p.prop_id == id32) {
                        if let Some(&tid) = grp.teleports.keys().next() {
                            return Some(tid);
                        }
                    }
                }
                None
            })
        })
    } else if req.entry_id != 0 && !GAME_RES.level_output_configs.contains_key(&req.entry_id) {
        Some(req.entry_id)
    } else if (resolved_entry_id == 100000104 || resolved_entry_id == 1000001)
        && req.scene_identifier.as_ref().map_or(false, |si| si.content_id != 0)
    {
        Some(2206)
    } else {
        None
    };

    match load_scene(session, resolved_entry_id, true, teleport_id, req.scene_identifier).await {
        Ok(scene_info) => {
            scene_debug_log(&format!(
                "ENTER_SCENE_RESOLVED requested_entry={} resolved_entry={}",
                req.entry_id, resolved_entry_id
            ));
            res.retcode = 0;
            res.scene_identifier = scene_info.scene_identifier;
            res.is_close_map = req.is_close_map;
        }
        Err(e) => {
            scene_debug_log(&format!(
                "ENTER_SCENE_FAILED requested_entry={} resolved_entry={} err={e:?}",
                req.entry_id, resolved_entry_id
            ));
            tracing::error!("Failed to enter scene {}: {:?}", resolved_entry_id, e);
            res.retcode = 2605;
        }
    }
}

pub async fn on_interact_prop_cs_req(
    session: &mut PlayerSession,
    req: &InteractPropCsReq,
    res: &mut InteractPropScRsp,
) {
    tracing::info!(
        "on_interact_prop_cs_req: prop_entity_id={}, interact_id={}, interact_id2={}",
        req.prop_entity_id, req.interact_id, req.interact_id2
    );
    scene_debug_log(&format!(
        "INTERACT_PROP prop_entity_id={} interact_id={} interact_id2={} (current scene: entry={} plane={} floor={})",
        req.prop_entity_id,
        req.interact_id,
        req.interact_id2,
        session
            .json_data
            .get()
            .map(|p| p.scene.entry_id)
            .unwrap_or(0),
        session.json_data.get().map(|p| p.scene.plane_id).unwrap_or(0),
        session.json_data.get().map(|p| p.scene.floor_id).unwrap_or(0),
    ));

    // ค้น prop จริงจาก mapping ที่เก็บตอนโหลดฉาก เพื่อตอบ state ที่ถูกต้อง
    // หมายเหตุ: interact_id2 คือ interaction id ฝั่ง client ไม่ใช่ prop state
    // ตอบ state ผิด (เช่น 1100) จะทำให้ interaction พัง/จุดเข้าเปิด UI ไม่ได้
    let real_prop = session.prop_entities.get(&req.prop_entity_id).copied();
    scene_debug_log(&format!(
        "INTERACT_PROP_RESOLVED entity_id={} -> {:?} interact_id2={}",
        req.prop_entity_id, real_prop, req.interact_id2
    ));

    res.retcode = 0;
    res.prop_entity_id = req.prop_entity_id;
    res.prop_state = real_prop.map(|(_, _, state)| state).unwrap_or(1);
}

pub async fn on_get_npc_taken_reward_cs_req(
    _session: &mut PlayerSession,
    req: &GetNpcTakenRewardCsReq,
    res: &mut GetNpcTakenRewardScRsp,
) {
    // client เรียกตอนโต้ตอบ NPC เพื่อถามสถานะ talk/reward ของ NPC
    // ถ้าไม่ตอบ หน้าต่างคุยจะไม่มีตัวเลือกให้กดและค้าง
    scene_debug_log(&format!("GET_NPC_TAKEN_REWARD npc_id={}", req.npc_id));
    res.retcode = 0;
    res.npc_id = req.npc_id;
}

pub async fn on_get_scene_map_info_cs_req(
    _sesison: &mut PlayerSession,
    req: &GetSceneMapInfoCsReq,
    res: &mut GetSceneMapInfoScRsp,
) {
    for si in &req.scene_identifiers {
        let floor_id = si.floor_id;
        let mut map_info = SceneMapInfo {
            chest_list: vec![
                ChestInfo {
                    chest_type: 101,
                    ..Default::default()
                },
                ChestInfo {
                    chest_type: 102,
                    ..Default::default()
                },
                ChestInfo {
                    chest_type: 104,
                    ..Default::default()
                },
            ],
            floor_id,
            scene_identifier: Some(*si),
            ..Default::default()
        };

        let floor_configs = GAME_RES
            .map_default_entrance_map
            .get(&floor_id)
            .and_then(|v| {
                GAME_RES
                    .level_output_configs
                    .get(v)
                    .and_then(|v| v.iter().next())
            });

        if let Some((_, floor_config)) = floor_configs {
            for (group_id, group) in floor_config.scenes.iter() {
                map_info.group_list.push(MapInfoGroup {
                    group_id: *group_id,
                    ..Default::default()
                });

                for teleport in group.teleports.keys() {
                    map_info.unlocked_teleport_list.push(*teleport)
                }

                for prop in &group.props {
                    map_info.map_info_prop_list.push(MazePropState {
                        group_id: prop.group_id,
                        state: prop.prop_state,
                        config_id: prop.inst_id,
                        extra_info: Option::<PropExtraInfo>::None,
                    });
                    // map_info.maze_group_list.push(MazeGroup {
                    //     group_id: prop.group_id,
                    //     state: prop.prop_state,
                    //     config_id: prop.inst_id,
                    //     extra_info: Option::None,
                    // });
                }
            }

            map_info.lighten_section_list = floor_config.sections.clone();
            map_info.floor_saved_data = floor_config.saved_values.clone();
            // #TODO!
            // map_info
            //     .chest_unlock_progress_list
            //     .push(ChestUnlockProgress {
            //         r#type: 0,
            //         total_chest_count: 25,
            //         unlocked_chest_count: 25,
            //     });
        }

        res.scene_map_info.push(map_info)
    }
}

pub async fn on_scene_entity_move_cs_req(
    session: &mut PlayerSession,
    req: &SceneEntityMoveCsReq,
    _res: &mut SceneEntityMoveScRsp,
) {
    let Some(player) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    // Don't save position if the player is currently inside a challenge arena
    if player.scene.entry_id >= 3000000 && player.scene.entry_id < 4000000 {
        return;
    }

    if util::cur_timestamp_ms() <= session.next_scene_save {
        return;
    }

    // save every 5 sec
    session.next_scene_save = util::cur_timestamp_ms() + (5 * 1000);

    for entity in &req.entity_motion_list {
        if entity.entity_id != 0 {
            continue;
        }

        if let Some(motion) = &entity.motion {
            if let Some(pos) = &motion.pos {
                if pos.y < -5000 {
                    return;
                }
                player.position.x = pos.x;
                player.position.y = pos.y;
                player.position.z = pos.z;
            }
            if let Some(rot) = &motion.rot {
                player.position.rot_y = rot.y;
            }
        }
    }

    player.save_persistent().await;
}

pub async fn on_get_entered_scene_cs_req(
    _session: &mut PlayerSession,
    _req: &GetEnteredSceneCsReq,
    res: &mut GetEnteredSceneScRsp,
) {
    res.entered_scene_info_list = GAME_RES
        .level_output_configs
        .iter()
        .flat_map(|(_, v)| {
            v.iter()
                .filter(|(_, v)| v.is_entered_scene_info)
                .map(|(k, _)| {
                    let split: Vec<_> = k.split("_").collect();
                    let plane_id = &split[0][1..];
                    let floor_id = &split[1][1..];
                    EnteredSceneInfo {
                        floor_id: floor_id.parse().unwrap(),
                        plane_id: plane_id.parse().unwrap(),
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
}

async fn load_scene(
    session: &mut PlayerSession,
    entry_id: u32,
    is_enter_scene: bool,
    teleport_id: Option<u32>,
    req_scene_identifier: Option<SceneIdentifier>,
) -> Result<SceneInfo> {
    let Some(json) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return Err(anyhow::format_err!("data is not set!"));
    };

    let (name, scene) = GAME_RES
        .level_output_configs
        .get(&entry_id)
        .and_then(|v| v.iter().next())
        .ok_or_else(|| {
            tracing::error!("Map Entrance Not Found {}", entry_id);
            anyhow::format_err!("Map Entrance Not Found {}", entry_id)
        })?;

    let split: Vec<_> = name.split("_").collect();
    let plane_id = split[0][1..].parse::<u32>()?;
    let floor_id = split[1][1..].parse::<u32>()?;

    let mut json_pos = json.position.clone();
    if let Some(teleport_id) = teleport_id {
        if let Some(teleport) = scene
            .scenes
            .iter()
            .find_map(|(_, v)| v.teleports.get(&teleport_id))
        {
            json_pos.x = teleport.pos.x;
            json_pos.y = teleport.pos.y;
            json_pos.z = teleport.pos.z;
            json_pos.rot_y = teleport.rot.y;
        } else if let Some((_, teleport)) = scene
            .scenes
            .iter()
            .find_map(|v| v.1.teleports.iter().next())
        {
            json_pos.x = teleport.pos.x;
            json_pos.y = teleport.pos.y;
            json_pos.z = teleport.pos.z;
            json_pos.rot_y = teleport.rot.y;
        }
    } else if is_enter_scene || json_pos.y < -5000 {
        let first_teleport = scene
            .scenes
            .values()
            .find_map(|g| g.teleports.values().next())
            .or_else(|| {
                // fallback: หาจาก teleports.json (ครอบคลุมทุกแมพ รวมจุดเข้า Endgame)
                common::resources::TELEPORT_DB
                    .get(&floor_id)
                    .and_then(|m| m.values().next())
            });
        let first_prop = scene
            .scenes
            .values()
            .flat_map(|g| g.props.iter())
            .next();

        if let Some(teleport) = first_teleport {
            json_pos.x = teleport.pos.x;
            json_pos.y = teleport.pos.y;
            json_pos.z = teleport.pos.z;
            json_pos.rot_y = teleport.rot.y;
        } else if let Some(prop) = first_prop {
            // Endgame/arena floors in res.json have no teleports at all.
            // Spawn the player near the first prop (the interactable entrance)
            // facing it, so the lobby is reachable on foot.
            let dx = 0 - prop.pos.x;
            let dz = 0 - prop.pos.z;
            let dist = ((dx * dx + dz * dz) as f64).sqrt();
            let (nx, nz) = if dist > 1000.0 {
                (dx as f64 / dist, dz as f64 / dist)
            } else {
                (0.0, 1.0)
            };
            let offset = 2500.0_f64.min(dist.max(1000.0));
            json_pos.x = prop.pos.x + (nx * offset) as i32;
            json_pos.y = prop.pos.y;
            json_pos.z = prop.pos.z + (nz * offset) as i32;

            let ang = (-nx).atan2(-nz) * 180.0 / std::f64::consts::PI;
            let mut deg = ang as i32;
            if deg < 0 {
                deg += 360;
            }
            json_pos.rot_y = deg * 1000;
        }
    }

    let scene_identifier = if let Some(mut si) = req_scene_identifier {
        if si.floor_id == 0 {
            si.floor_id = floor_id;
        }
        si
    } else {
        SceneIdentifier {
            floor_id,
            ..Default::default()
        }
    };

    let mut scene_info = SceneInfo {
        floor_id,
        plane_id,
        entry_id,
        game_mode_type: scene.plane_type,
        leader_entity_id: 1,
        world_id: if scene.world_id == 100 {
            501
        } else {
            scene.world_id
        },
        lighten_section_list: scene.sections.clone(),
        opened_chests_list: scene
            .scenes
            .values()
            .flat_map(|v| v.chests.clone())
            .collect::<Vec<_>>(),
        scene_mission_info: Some(MissionStatusBySceneInfo {
            finished_main_mission_id_list: scene
                .scenes
                .values()
                .flat_map(|s| s.finished_main_missions.clone())
                .collect::<Vec<_>>(),
            sub_mission_status_list: scene
                .scenes
                .values()
                .flat_map(|s| {
                    s.finished_sub_missions.iter().map(|sm| Mission {
                        id: *sm,
                        status: MissionStatus::MissionFinish.into(),
                        progress: 0,
                    })
                })
                .collect::<Vec<_>>(),
            ..Default::default()
        }),
        floor_saved_data: scene.saved_values.clone(),
        scene_identifier: Some(scene_identifier),
        ..Default::default()
    };

    let lineup_info = AvatarJson::to_lineup_info(&json.lineups);
    let player_pos = MotionInfo {
        rot: Some(Vector {
            x: 0,
            y: json_pos.rot_y,
            z: 0,
        }),
        pos: Some(Vector {
            x: json_pos.x,
            y: json_pos.y,
            z: json_pos.z,
        }),
    };

    let mut loaded_npc: Vec<u32> = vec![];
    let mut prop_entity_id = 1_000;
    let mut npc_entity_id = 20_000;
    let mut monster_entity_id = 30_000;

    session.prop_entities.clear();

    for (group_id, group) in &scene.scenes {
        let mut group_info = SceneEntityGroupInfo {
            group_id: *group_id,
            ..Default::default()
        };

        // Load Props
        for prop in &group.props {
            prop_entity_id += 1;
            session.prop_entities.insert(
                prop_entity_id,
                (prop.prop_id, prop.inst_id, prop.prop_state),
            );

            let prop_position = Position {
                x: (prop.pos.x),
                y: (prop.pos.y),
                z: (prop.pos.z),
                rot_y: (prop.rot.y),
            };

            let entity_info = SceneEntityInfo {
                inst_id: prop.inst_id,
                group_id: prop.group_id,
                motion: Some(prop_position.into()),
                entity: Some(Entity::Prop(ScenePropInfo {
                    prop_state: prop.prop_state,
                    prop_id: prop.prop_id,
                    ..Default::default()
                })),
                entity_id: prop_entity_id,
            };

            group_info.entity_list.push(entity_info);
        }

        // Load NPCs
        for npc in &group.npcs {
            if loaded_npc.contains(&(npc.npc_id)) || json.avatars.contains_key(&(npc.npc_id)) {
                continue;
            }
            npc_entity_id += 1;
            loaded_npc.push(npc.npc_id);

            let npc_position = Position {
                x: npc.pos.x,
                y: npc.pos.y,
                z: npc.pos.z,
                rot_y: npc.rot.y,
            };

            let info = SceneEntityInfo {
                inst_id: npc.inst_id,
                group_id: npc.group_id,
                entity_id: npc_entity_id,
                motion: Some(npc_position.into()),
                entity: Some(Entity::Npc(SceneNpcInfo {
                    npc_id: npc.npc_id,
                    ..Default::default()
                })),
            };

            group_info.entity_list.push(info);
        }

        // Load Monsters
        for monster in &group.monsters {
            monster_entity_id += 1;
            let monster_position = Position {
                x: monster.pos.x,
                y: monster.pos.y,
                z: monster.pos.z,
                rot_y: monster.rot.y,
            };

            let npc_monster = SceneNpcMonsterInfo {
                monster_id: monster.monster_id,
                event_id: monster.event_id,
                world_level: 6,
                ..Default::default()
            };

            let info = SceneEntityInfo {
                inst_id: monster.inst_id,
                group_id: monster.group_id,
                entity_id: monster_entity_id,
                motion: Some(monster_position.into()),
                entity: Some(Entity::NpcMonster(npc_monster)),
            };

            group_info.entity_list.push(info);
        }

        // TODO: for now don't load group that have nothing in it
        if group.props.is_empty() && group.npcs.is_empty() && group.monsters.is_empty() {
            continue;
        }

        scene_info.entity_group_list.push(group_info);

        // TODO: ?
        // scene_info.group_state_list.push(SceneGroupState {
        //     group_id: *group_id,
        //     is_default: true,
        //     state: 0,
        // });
    }

    // load player entity
    scene_info.entity_group_list.push(SceneEntityGroupInfo {
        state: 0,
        group_id: 0,
        entity_list: json
            .lineups
            .iter()
            .map(|(slot, avatar_id)| SceneEntityInfo {
                inst_id: 0,
                entity_id: (*slot) + 1,
                motion: Some(player_pos),
                entity: Some(Entity::Actor(SceneActorInfo {
                    avatar_type: AvatarType::AvatarFormalType.into(),
                    base_avatar_id: *avatar_id,
                    map_layer: 0,
                    uid: 25,
                })),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    });

    if is_enter_scene {
        json.scene.entry_id = entry_id;
        json.scene.floor_id = floor_id;
        json.scene.plane_id = plane_id;
        json.position.x = json_pos.x;
        json.position.y = json_pos.y;
        json.position.z = json_pos.z;
        json.position.rot_y = json_pos.rot_y;

        json.save_persistent().await;

        session
            .send(EnterSceneByServerScNotify {
                scene: Some(scene_info.clone()),
                lineup: Some(lineup_info),
                ..Default::default()
            })
            .await?;
    }

    Ok(scene_info)
}

pub async fn load_challenge_scene(
    session: &mut PlayerSession,
    entry_id: u32,
    target_group_id: u32,
    monster_id: u32,
    event_id: u32,
    avatar_ids: &[u32],
) -> Result<(SceneInfo, MotionInfo)> {
    let Some(json) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return Err(anyhow::format_err!("data is not set!"));
    };

    let (name, scene) = GAME_RES
        .level_output_configs
        .get(&entry_id)
        .and_then(|v| v.iter().next())
        .ok_or_else(|| {
            tracing::error!("Challenge Map Entrance Not Found {}", entry_id);
            anyhow::format_err!("Challenge Map Entrance Not Found {}", entry_id)
        })?;

    let split: Vec<_> = name.split('_').collect();
    let plane_id = split[0][1..].parse::<u32>()?;
    let floor_id = split[1][1..].parse::<u32>()?;

    // 1. Locate boss monster position first from scene config
    let (mut mons_pos, inst_id) = if let Some(grp) = scene.scenes.get(&target_group_id) {
        if let Some(m) = grp.monsters.first() {
            (Position { x: m.pos.x, y: m.pos.y, z: m.pos.z, rot_y: m.rot.y }, m.inst_id)
        } else {
            (Position { x: -61000, y: -2141, z: -170700, rot_y: 90000 }, 1)
        }
    } else if let Some((_, grp)) = scene.scenes.iter().find(|(_, g)| !g.monsters.is_empty()) {
        let m = grp.monsters.first().unwrap();
        (Position { x: m.pos.x, y: m.pos.y, z: m.pos.z, rot_y: m.rot.y }, m.inst_id)
    } else {
        (Position { x: -61000, y: -2141, z: -170700, rot_y: 90000 }, 1)
    };

    // 2. Position player safely on platform between boss and arena center (0, mons_pos.y, 0)
    let center_vx = -mons_pos.x as f64;
    let center_vz = -mons_pos.z as f64;
    let dist_to_center = (center_vx * center_vx + center_vz * center_vz).sqrt();

    let (spawn_x, spawn_y, spawn_z, spawn_rot_y, boss_rot_y) = if dist_to_center > 1000.0 {
        let nx = center_vx / dist_to_center;
        let nz = center_vz / dist_to_center;
        let offset = 6000.0_f64.min(dist_to_center * 0.6);
        let sx = mons_pos.x + (nx * offset) as i32;
        let sz = mons_pos.z + (nz * offset) as i32;

        // Player faces the boss: direction is (-nx, -nz)
        let p_ang = (-nx).atan2(-nz) * 180.0 / std::f64::consts::PI;
        let mut p_deg = p_ang as i32;
        if p_deg < 0 { p_deg += 360; }

        // Boss faces the player: direction is (nx, nz)
        let b_ang = nx.atan2(nz) * 180.0 / std::f64::consts::PI;
        let mut b_deg = b_ang as i32;
        if b_deg < 0 { b_deg += 360; }

        (sx, mons_pos.y, sz, (p_deg * 1000) as i32, (b_deg * 1000) as i32)
    } else {
        // Boss is at center: player spawns 6m away facing center
        (0, mons_pos.y, 6000, 180_000, 0)
    };

    mons_pos.rot_y = boss_rot_y;

    let player_motion = MotionInfo {
        rot: Some(Vector {
            x: 0,
            y: spawn_rot_y,
            z: 0,
        }),
        pos: Some(Vector {
            x: spawn_x,
            y: spawn_y,
            z: spawn_z,
        }),
    };

    json.scene.entry_id = entry_id;
    json.scene.floor_id = floor_id;
    json.scene.plane_id = plane_id;

    let mut scene_info = SceneInfo {
        floor_id,
        plane_id,
        entry_id,
        game_mode_type: scene.plane_type, // 4 for GAME_MODE_CHALLENGE
        leader_entity_id: 1,
        world_id: if scene.world_id == 100 { 501 } else { scene.world_id },
        lighten_section_list: scene.sections.clone(),
        opened_chests_list: Vec::new(),
        floor_saved_data: scene.saved_values.clone(),
        scene_identifier: Some(SceneIdentifier {
            floor_id,
            ..Default::default()
        }),
        scene_mission_info: Some(MissionStatusBySceneInfo::default()),
        ..Default::default()
    };

    // Load props for group 1, target group, and common prop groups
    let mut prop_entity_id = 1_000;
    for (gid, group) in &scene.scenes {
        if *gid == 1 || *gid == target_group_id || *gid == 7 || *gid == 8 {
            let mut group_info = SceneEntityGroupInfo {
                group_id: *gid,
                ..Default::default()
            };
            for prop in &group.props {
                prop_entity_id += 1;
                session.prop_entities.insert(
                    prop_entity_id,
                    (prop.prop_id, prop.inst_id, prop.prop_state),
                );
                group_info.entity_list.push(SceneEntityInfo {
                    inst_id: prop.inst_id,
                    group_id: prop.group_id,
                    motion: Some(Position {
                        x: prop.pos.x,
                        y: prop.pos.y,
                        z: prop.pos.z,
                        rot_y: prop.rot.y,
                    }.into()),
                    entity: Some(Entity::Prop(ScenePropInfo {
                        prop_state: prop.prop_state,
                        prop_id: prop.prop_id,
                        ..Default::default()
                    })),
                    entity_id: prop_entity_id,
                });
            }
            if !group_info.entity_list.is_empty() {
                scene_info.entity_group_list.push(group_info);
            }
        }
    }

    // Load single boss monster for the challenge
    let mut monster_group = SceneEntityGroupInfo {
        group_id: target_group_id,
        ..Default::default()
    };
    monster_group.entity_list.push(SceneEntityInfo {
        inst_id,
        group_id: target_group_id,
        entity_id: 30_001,
        motion: Some(mons_pos.into()),
        entity: Some(Entity::NpcMonster(SceneNpcMonsterInfo {
            monster_id,
            event_id,
            world_level: 6,
            ..Default::default()
        })),
    });
    scene_info.entity_group_list.push(monster_group);

    // Load player team actors
    let avatars_to_use: Vec<u32> = if avatar_ids.is_empty() {
        json.lineups.values().copied().collect()
    } else {
        avatar_ids.to_vec()
    };

    let player_group = SceneEntityGroupInfo {
        state: 0,
        group_id: 0,
        entity_list: avatars_to_use
            .iter()
            .enumerate()
            .map(|(slot, &aid)| SceneEntityInfo {
                inst_id: 0,
                entity_id: (slot as u32) + 1,
                motion: Some(player_motion.clone()),
                entity: Some(Entity::Actor(SceneActorInfo {
                    avatar_type: AvatarType::AvatarFormalType.into(),
                    base_avatar_id: aid,
                    map_layer: 0,
                    uid: 1337,
                })),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    scene_info.entity_group_list.push(player_group);

    // Note: Do NOT overwrite json.scene in persistent so player returns to open world on logout/finish!

    Ok((scene_info, player_motion))
}

// cmd 1432 → 1421: GetUnlockTeleportScRsp (ปลดล็อคเสาวาปทั้งเกม)
pub async fn on_get_unlock_teleport_cs_req(
    _session: &mut PlayerSession,
    body: &GetUnlockTeleportCsReq,
    res: &mut GetUnlockTeleportScRsp,
) {
    res.retcode = 0;
    let unlocks = mission::get_mission_unlocks();
    let mut tps = unlocks.teleport_ids.clone();
    if tps.is_empty() {
        tps = common::resources::TELEPORT_DB
            .values()
            .flat_map(|m| m.keys().copied())
            .collect();
    }
    tps.sort_unstable();
    tps.dedup();
    res.unlocked_teleport_list = tps;

    scene_debug_log(&format!(
        "GET_UNLOCK_TELEPORT req_entries={} teleports_sent={}",
        body.entry_id_list.len(),
        res.unlocked_teleport_list.len()
    ));
}

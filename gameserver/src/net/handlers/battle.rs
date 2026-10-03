use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use rand::RngExt;

use common::{
    resources::GAME_RES,
    structs::{BattleType, Monster},
};

use super::*;

pub static CURRENT_BATTLE: LazyLock<Mutex<HashMap<u32, SceneBattleInfo>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn on_start_cocoon_stage_cs_req(
    session: &mut PlayerSession,
    req: &StartCocoonStageCsReq,
    res: &mut StartCocoonStageScRsp,
) {
    let battle_info = create_battle_info(session, 0, 0).await;
    CURRENT_BATTLE
        .lock()
        .unwrap()
        .insert(session.token, battle_info.clone());

    res.prop_entity_id = req.prop_entity_id;
    res.cocoon_id = req.cocoon_id;
    res.wave = req.wave;
    res.battle_info = Some(battle_info);
}

pub async fn on_quick_start_cocoon_stage_cs_req(
    session: &mut PlayerSession,
    req: &QuickStartCocoonStageCsReq,
    res: &mut QuickStartCocoonStageScRsp,
) {
    let mut battle_info = create_battle_info(session, 0, 0).await;
    battle_info.world_level = req.world_level;
    CURRENT_BATTLE
        .lock()
        .unwrap()
        .insert(session.token, battle_info.clone());

    res.cocoon_id = req.cocoon_id;
    res.wave = req.wave;
    res.battle_info = Some(battle_info);
}

pub async fn on_scene_enter_stage_cs_req(
    session: &mut PlayerSession,
    _req: &SceneEnterStageCsReq,
    res: &mut SceneEnterStageScRsp,
) {
    let battle_info = create_battle_info(session, 0, 0).await;
    CURRENT_BATTLE
        .lock()
        .unwrap()
        .insert(session.token, battle_info.clone());

    res.battle_info = Some(battle_info);
}

pub async fn on_pve_battle_result_cs_req(
    session: &mut PlayerSession,
    req: &PveBattleResultCsReq,
    res: &mut PveBattleResultScRsp,
) {
    CURRENT_BATTLE.lock().unwrap().remove(&session.token);
    res.end_status = req.end_status;
    res.battle_id = req.battle_id;
    res.stage_id = req.stage_id;
}

pub async fn on_scene_cast_skill_cs_req(
    session: &mut PlayerSession,
    req: &SceneCastSkillCsReq,
    res: &mut SceneCastSkillScRsp,
) {
    res.cast_entity_id = req.cast_entity_id;

    let mut targets = req
        .hit_target_entity_id_list
        .iter()
        .chain(&req.assist_monster_entity_id_list)
        .copied()
        .filter(|id| *id > 30_000 || *id < 1_000)
        .collect::<Vec<_>>();

    if targets.is_empty() {
        if let Some(player) = session.json_data.get() {
            if player.scene.entry_id >= 3000000 && !player.battle_config.monsters.is_empty() {
                targets.push(30_001);
            }
        }
    }

    if targets.is_empty() {
        tracing::warn!("scene cast skill target is empty!");
        return;
    }

    let battle_info = create_battle_info(session, req.caster_id, req.skill_index).await;

    CURRENT_BATTLE
        .lock()
        .unwrap()
        .insert(session.token, battle_info.clone());

    res.cast_entity_id = req.cast_entity_id;
    res.monster_battle_info = targets
        .iter()
        .map(|&id| HitMonsterBattleInfo {
            target_monster_entity_id: id,
            monster_battle_type: 0,
        })
        .collect();
    res.battle_info = Some(battle_info);
}

pub async fn on_get_cur_battle_info_cs_req(
    session: &mut PlayerSession,
    _req: &GetCurBattleInfoCsReq,
    res: &mut GetCurBattleInfoScRsp,
) {
    if let Some(battle_info) = CURRENT_BATTLE.lock().unwrap().get(&session.token) {
        res.retcode = 0;
        res.battle_info = Some(battle_info.clone());
        return;
    }

    if let Some(player) = session.json_data.get() {
        if !player.battle_config.monsters.is_empty() {
            let battle_info = create_battle_info(session, 0, 0).await;
            CURRENT_BATTLE
                .lock()
                .unwrap()
                .insert(session.token, battle_info.clone());
            res.retcode = 0;
            res.battle_info = Some(battle_info);
            return;
        }
    }

    res.retcode = 2000; // RET_BATTLE_STAGE_NOT_MATCH
}

pub async fn create_battle_info(
    session: &mut PlayerSession,
    caster_id: u32,
    skill_index: u32,
) -> SceneBattleInfo {
    // if let Some(player) = session.json_data.get_mut() {
    //     let _ = player.refresh_persistent().await;
    // }

    let Some(player) = session.json_data.get() else {
        tracing::error!("data is not set!");
        return SceneBattleInfo::default();
    };

    let mut battle_info = SceneBattleInfo {
        stage_id: player.battle_config.stage_id,
        logic_random_seed: rand::rng().random::<u32>(),
        battle_id: 1,
        rounds_limit: player.battle_config.cycle_count,
        world_level: 6,
        ..Default::default()
    };

    let is_calyx = caster_id == 0 && skill_index == 0;
    let mut has_dahlia = false;
    let mut first_avatar_id = 0;
    let mut first_avatar_idx = 9999;

    let lineup_uwu = if let Some(custom) = &player.battle_config.custom_battle_lineup {
        custom.iter()
    } else {
        player.lineups.iter()
    };

    // avatars
    for (avatar_index, avatar_id) in lineup_uwu {
        if first_avatar_id == 0 {
            first_avatar_id = *avatar_id;
            first_avatar_idx = *avatar_index;
        }
        if !has_dahlia {
            has_dahlia = *avatar_id == 1321;
        }

        let is_trailblazer = *avatar_id == 8001;
        let is_march = *avatar_id == 1001;

        let avatar_id = if is_trailblazer {
            player.main_character as u32
        } else if is_march {
            player.march_type as u32
        } else {
            *avatar_id
        };

        if let Some(avatar) = player.avatars.get(&avatar_id) {
            let (battle_avatar, techs) = avatar.to_battle_avatar_proto(
                *avatar_index,
                player
                    .lightcones
                    .iter()
                    .find(|v| v.equip_avatar == avatar.avatar_id),
                player
                    .relics
                    .iter()
                    .filter(|v| v.equip_avatar == avatar.avatar_id)
                    .collect::<Vec<_>>(),
            );

            battle_info.buff_list.extend(techs);

            if caster_id > 0
                && *avatar_index == (caster_id - 1)
                && let Some(avatar_config) = GAME_RES.avatar_configs.get(&avatar_id)
                && !avatar.techniques.contains(&1000119)
            {
                battle_info.buff_list.push(BattleBuff {
                    id: avatar_config.weakness_buff_id,
                    level: 1,
                    owner_id: *avatar_index,
                    wave_flag: 0xffffffff,
                    dynamic_values: HashMap::from([(
                        String::from("SkillIndex"),
                        skill_index as f32,
                    )]),
                    ..Default::default()
                });
            }

            battle_info.battle_avatar_list.push(battle_avatar);

            // hardcoded march
            if avatar.avatar_id == 1224 {
                battle_info.buff_list.push(BattleBuff {
                    id: 122401,
                    level: 3,
                    wave_flag: 0xffffffff,
                    owner_id: *avatar_index,
                    dynamic_values: HashMap::from([
                        (String::from("#ADF_1"), 3f32),
                        (String::from("#ADF_2"), 3f32),
                    ]),
                    target_index_list: vec![0],
                });
            }
        };
    }

    // Resolve character-specific technique buffs dynamically
    let first_avatar_attack_id = GAME_RES
        .avatar_configs
        .get(&first_avatar_id)
        .map(|c| c.weakness_buff_id)
        .unwrap_or(0);

    let len = battle_info.buff_list.len();
    let mut has_replaced = false;
    let mut dahlia_buffs = Vec::new();
    for (i, buff) in battle_info.buff_list.iter_mut().enumerate() {
        let is_last = i == len - 1;

        if buff.id == 141202 || buff.id == 141403 {
            buff.owner_id = first_avatar_idx;
            continue;
        }

        if has_dahlia
            && is_calyx
            && let 1000111..=1000117 = buff.id
            && !has_replaced
            && first_avatar_attack_id != 0
        {
            buff.id = first_avatar_attack_id;
            buff.owner_id = first_avatar_idx;
            has_replaced = true;
        }

        if has_dahlia && is_calyx && !has_replaced && first_avatar_attack_id != 0 && is_last {
            dahlia_buffs.push(BattleBuff {
                id: 1000121,
                level: 1,
                wave_flag: u32::MAX,
                target_index_list: vec![first_avatar_idx],
                ..Default::default()
            });

            dahlia_buffs.push(BattleBuff {
                id: first_avatar_attack_id,
                level: 1,
                wave_flag: u32::MAX,
                target_index_list: vec![first_avatar_idx],
                dynamic_values: HashMap::from([(
                    String::from("SkillIndex"),
                    first_avatar_idx as f32,
                )]),
                ..Default::default()
            });
        }
    }

    if !dahlia_buffs.is_empty() {
        battle_info.buff_list.extend(dahlia_buffs);
    }

    // custom stats for avatars
    for stat in &player.battle_config.custom_stats {
        for avatar in &mut battle_info.battle_avatar_list {
            if avatar.relic_list.is_empty() {
                avatar.relic_list.push(BattleRelic {
                    id: 61011,
                    main_affix_id: 1,
                    level: 1,
                    ..Default::default()
                })
            }

            if let Some(sub_affix) = avatar.relic_list[0]
                .sub_affix_list
                .iter_mut()
                .find(|v| v.affix_id == stat.sub_affix_id)
            {
                sub_affix.cnt = stat.count;
            } else {
                avatar.relic_list[0].sub_affix_list.push(RelicAffix {
                    affix_id: stat.sub_affix_id,
                    cnt: stat.count,
                    step: stat.step,
                })
            }
        }
    }

    // blessings
    for blessing in &player.battle_config.blessings {
        let mut buffs = BattleBuff {
            id: blessing.id,
            level: blessing.level,
            wave_flag: 0xffffffff,
            owner_id: 0xffffffff,
            ..Default::default()
        };

        if let Some(dynamic_value) = &blessing.dynamic_key {
            buffs
                .dynamic_values
                .insert(dynamic_value.key.clone(), dynamic_value.value as f32);
        };

        for dynamic_value in &blessing.dynamic_values {
            if buffs.dynamic_values.contains_key(&dynamic_value.key) {
                continue;
            };
            buffs
                .dynamic_values
                .insert(dynamic_value.key.clone(), dynamic_value.value as f32);
        }

        battle_info.buff_list.push(buffs);
    }

    // pf score object
    if player.battle_config.battle_type == BattleType::PF {
        if battle_info.stage_id >= 30309011 {
            battle_info.battle_target_info.insert(
                1,
                BattleTargetList {
                    battle_target_list: vec![BattleTarget {
                        id: 10003,
                        progress: 0,
                        ..Default::default()
                    }],
                },
            );
        } else {
            battle_info.battle_target_info.insert(
                1,
                BattleTargetList {
                    battle_target_list: vec![BattleTarget {
                        id: 10002,
                        progress: 0,
                        ..Default::default()
                    }],
                },
            );
        }

        for i in 2..=4 {
            battle_info
                .battle_target_info
                .insert(i, BattleTargetList::default());
        }

        battle_info.battle_target_info.insert(
            5,
            BattleTargetList {
                battle_target_list: vec![
                    BattleTarget {
                        id: 2001,
                        progress: 0,
                        ..Default::default()
                    },
                    BattleTarget {
                        id: 2002,
                        progress: 0,
                        ..Default::default()
                    },
                ],
            },
        );
    }

    // Apocalyptic Shadow
    if player.battle_config.battle_type == BattleType::AS {
        battle_info.battle_target_info.insert(
            1,
            BattleTargetList {
                battle_target_list: vec![BattleTarget {
                    id: 90005,
                    progress: 0,
                    ..Default::default()
                }],
            },
        );
    }

    //  SU
    if player.battle_config.battle_type == BattleType::SU {
        battle_info.battle_event.push(BattleEventBattleInfo {
            battle_event_id: player.battle_config.path_resonance_id,
            status: Some(BattleEventProperty {
                sp_bar: Some(SpBarInfo {
                    cur_sp: 10_000,
                    max_sp: 10_000,
                }),
            }),
            skill_info: Vec::with_capacity(0),
            ..Default::default()
        })
    }

    // Monsters
    battle_info.monster_wave_list = Monster::to_scene_monster_waves(&player.battle_config.monsters);
    battle_info.jaoiokbbmfc = battle_info.monster_wave_list.len() as u32;
    for wave in &mut battle_info.monster_wave_list {
        wave.battle_stage_id = battle_info.stage_id;
    }

    // Rogue Magic
    // TODO: i dont need these shit
    if !player.battle_config.scepters.is_empty() {
        // battle_info.battle_rogue_magic_info = Some(BattleRogueMagicInfo {
        //     detail_info: Some(BattleRogueMagicDetailInfo {
        //         paiigoggofj: Some(Item::BattleRogueMagicData(BattleRogueMagicData {
        //             round_cnt: Some(BattleRogueMagicRoundCount {
        //                 gpojenhaiba: 3,
        //                 kljklbmlefo: 0,
        //             }),
        //             battle_scepter_list: player
        //                 .battle_config
        //                 .scepters
        //                 .iter()
        //                 .map(|scepter| {
        //                     let mut battle_scepter = BattleRogueMagicScepter {
        //                         level: scepter.level,
        //                         scepter_id: scepter.id,
        //                         magic_list: Vec::new(),
        //                         trench_count: HashMap::from([(3, 0), (4, 0), (5, 0)]),
        //                     };
        //
        //                     let mut index = [0u32; 3];
        //
        //                     for component in &scepter.components {
        //                         let (slot_type, locked) = match component.component_type {
        //                             RogueMagicComponentType::Passive => (3u32, false),
        //                             RogueMagicComponentType::Active => (4, true),
        //                             RogueMagicComponentType::Attach => (5, false),
        //                         };
        //
        //                         let slot_index = &mut index[slot_type as usize - 3];
        //                         battle_scepter.magic_list.push(BattleRogueMagicUnit {
        //                             level: component.level,
        //                             unit_id: component.id,
        //                             slot_id: *slot_index,
        //                             locked,
        //                             counter_map: Default::default(),
        //                         });
        //                         *slot_index += 1;
        //                         *battle_scepter.trench_count.get_mut(&slot_type).unwrap() += 1;
        //                     }
        //
        //                     battle_scepter
        //                 })
        //                 .collect(),
        //         })),
        //     }),
        //     modifier_content: Some(BattleRogueMagicModifierInfo {
        //         rogue_magic_battle_const: 5,
        //     }),
        // });
    }

    // Global buffs
    let (mut has_castorice_global_buff, mut has_silver_wolf_global_buff) = (false, false);
    for buff in &battle_info.buff_list {
        if buff.id == 140703 {
            has_castorice_global_buff = true;
        }
        if buff.id == 150602 {
            has_silver_wolf_global_buff = true;
        }
    }

    if !has_castorice_global_buff && player.enable_castorice_global.unwrap_or(true) {
        battle_info.buff_list.push(BattleBuff {
            id: 140703,
            level: 1,
            owner_id: u32::MAX,
            wave_flag: u32::MAX,
            target_index_list: Vec::with_capacity(0),
            dynamic_values: HashMap::with_capacity(0),
        });
    }

    if !has_silver_wolf_global_buff && player.enable_sw_global.unwrap_or(true) {
        battle_info.buff_list.push(BattleBuff {
            id: 150602,
            level: 1,
            owner_id: u32::MAX,
            wave_flag: u32::MAX,
            target_index_list: Vec::with_capacity(0),
            dynamic_values: HashMap::with_capacity(0),
        });
    }

    battle_info
}

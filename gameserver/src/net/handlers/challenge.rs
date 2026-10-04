use super::*;
use proto::*;
use std::collections::{BTreeMap, HashMap};
use std::sync::{LazyLock, Mutex};
use serde::Deserialize;
use prost::Message;
use common::resources::GAME_RES;
use common::structs::{AvatarJson, BattleType, BattleBuffJson, Monster};

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ChallengeStageData {
    pub entrance: u32,
    pub entrance2: u32,
    pub group1: u32,
    pub group2: u32,
    pub monster1: u32,
    pub monster2: u32,
    pub event1: u32,
    pub event2: u32,
    pub buff: u32,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ChallengeTierceStageData {
    pub entrance: u32,
    pub group: u32,
    pub monster: u32,
    pub event: u32,
    #[serde(default)]
    pub targets: Vec<u32>,
    #[serde(default)]
    pub score: u32,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct StageBattleData {
    pub level: u32,
    pub monsters: Vec<Vec<u32>>,
}

/// หนึ่งสเตจของ Starward Mode (peak)
#[derive(Deserialize, Clone, Debug, Default)]
pub struct PeakStageData {
    #[serde(default)]
    pub event_id: u32,
    #[serde(default)]
    pub hard_event_id: u32,
    #[serde(default)]
    pub monster_id: u32,
    #[serde(default)]
    pub default_buff: u32,
    #[serde(default)]
    pub is_boss: bool,
    #[serde(default)]
    pub targets: Vec<u32>,
    #[serde(default)]
    pub hard_target: u32,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ChallengeConfigData {
    #[serde(default)]
    pub stages_list: Vec<u32>,
    #[serde(default)]
    pub groups_list: Vec<u32>,
    pub challenges: HashMap<u32, ChallengeStageData>,
    pub tierce: HashMap<u32, ChallengeTierceStageData>,
    pub stages: HashMap<u32, StageBattleData>,
    /// Starward Mode stages — key = stage id (101, 102, 201, ...)
    #[serde(default)]
    pub peak: HashMap<String, PeakStageData>,
}

pub static CHALLENGE_DATA: LazyLock<ChallengeConfigData> = LazyLock::new(|| {
    let path = "challenge_data.json";
    if let Ok(content) = std::fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_else(|e| {
            tracing::error!("Failed to parse challenge_data.json: {e}");
            ChallengeConfigData::default()
        })
    } else {
        tracing::error!("challenge_data.json not found!");
        ChallengeConfigData::default()
    }
});

pub static TIERCE_LINEUPS: LazyLock<Mutex<HashMap<u32, Vec<Vec<u32>>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn get_challenge_stages_list() -> Vec<u32> {
    if !CHALLENGE_DATA.stages_list.is_empty() {
        return CHALLENGE_DATA.stages_list.clone();
    }
    let mut stages = Vec::new();
    for &id in CHALLENGE_DATA.challenges.keys() {
        stages.push(id);
    }
    for &id in CHALLENGE_DATA.tierce.keys() {
        stages.push(id);
    }
    stages.sort();
    stages.dedup();
    stages
}

fn get_challenge_groups() -> Vec<u32> {
    if !CHALLENGE_DATA.groups_list.is_empty() {
        return CHALLENGE_DATA.groups_list.clone();
    }
    let mut groups = Vec::new();
    for &id in &get_challenge_stages_list() {
        if id < 100 {
            groups.push(100);
        } else if id < 1000 {
            groups.push(900);
        } else if id < 20000 {
            groups.push(id / 100);
        } else {
            groups.push(id / 10);
        }
    }
    groups.sort();
    groups.dedup();
    groups
}

/// group_id ที่ stage นี้อยู่ (ต้องตรงกับ logic ใน get_challenge_groups)
fn group_of_stage(id: u32) -> u32 {
    if id < 100 {
        100
    } else if id < 1000 {
        900
    } else if id < 20000 {
        id / 100
    } else {
        id / 10
    }
}

/// ดาวรวมของกลุ่ม (3 ดาวต่อ stage) — ใส่ใน taken_stars_count_reward เพื่อให้รางวัลดาวถือว่ารับครบ
fn group_total_stars(group_id: u32) -> u64 {
    get_challenge_stages_list()
        .into_iter()
        .filter(|&id| group_of_stage(id) == group_id)
        .count() as u64
        * 3
}

fn get_challenge_list() -> Vec<Challenge> {
    let make_challenge = |id: u32| Challenge {
        challenge_id: id,
        star: 7, // 7 = (1 << 0) | (1 << 1) | (1 << 2) -> Bitmask for all 3 stars!
        taken_reward: 7, // 7 = (1 << 0) | (1 << 1) | (1 << 2) -> All 3 star rewards claimed
        record_id: 1, // Non-zero record confirms stage cleared
        hgpkmhfpmbj: false, // NOT first open (already cleared, allows unlocking next stages)
        score_two: if (20000..30000).contains(&id) {
            80000
        } else if id >= 30000 || (5000..6000).contains(&id) {
            4000
        } else {
            0
        },
        score_id: if (20000..30000).contains(&id) {
            80000
        } else if id >= 30000 || (5000..6000).contains(&id) {
            4000
        } else {
            0
        },
        ..Default::default()
    };

    get_challenge_stages_list()
        .into_iter()
        .map(make_challenge)
        .collect()
}

pub async fn on_get_challenge_cs_req(
    _session: &mut PlayerSession,
    _req: &GetChallengeCsReq,
    res: &mut GetChallengeScRsp,
) {
    let groups = get_challenge_groups();

    res.retcode = 0;
    res.challenge_group_list = groups
        .iter()
        .map(|&group_id| ChallengeGroup {
            group_id,
            taken_stars_count_reward: group_total_stars(group_id),
        })
        .collect();
    // สถานะปลดล็อกของแต่ละกลุ่ม — ถ้าไม่ส่ง กลุ่มใหม่ (tierce 4.6) จะโชว์ไม่ปลดใน UI
    res.kkiafpfklge = groups
        .iter()
        .map(|&group_id| Fmdaaiklaja {
            group_id,
            jfkmnbhobcl: true,
            egllmgllhdl: true,
        })
        .collect();
    res.challenge_list = get_challenge_list();

    let mut max_levels = Vec::new();
    let mut add_max_level = |level: u32, r_type: u32| {
        for unlocked in [true, false] {
            max_levels.push(ChallengeHistoryMaxLevel {
                level,
                hnhcfjjnjce: unlocked,
                reward_display_type: r_type,
            });
        }
    };

    add_max_level(15, 1); // Jarilo
    add_max_level(6, 2);  // Luofu
    add_max_level(12, 3); // MoC 12 floors
    add_max_level(4, 4);  // Pure Fiction 4 stages
    add_max_level(4, 5);  // Apocalyptic Shadow 4 difficulties
    add_max_level(6, 6);  // Anomaly Arbitration

    for r in 101001..=101015 { add_max_level(15, r); }
    for r in 101016..=101021 { add_max_level(6, r); }
    for r in 101201..=101212 { add_max_level(12, r); } // MoC Floors 1..12
    for r in 101401..=101404 { add_max_level(4, r); }  // Pure Fiction
    for r in [101713, 101913, 102113] { add_max_level(4, r); } // Apocalyptic Shadow

    res.max_level_list = max_levels;
}

/// Starward Mode (ChallengePeak) — ส่งข้อมูลปลดครบทุก peak ให้ UI ไม่โชว์ No Data
/// แบ่ง peak stage เป็นกลุ่มตามหลักร้อย (101-104 → group 1, 201-204 → group 2, ...)
pub fn peak_stage_group(stage_id: u32) -> u32 {
    stage_id / 100
}

pub async fn on_get_challenge_peak_data_cs_req(
    _session: &mut PlayerSession,
    _req: &GetChallengePeakDataCsReq,
    res: &mut GetChallengePeakDataScRsp,
) {
    res.retcode = 0;
    res.current_peak_group_id = 1;

    // จัดกลุ่มจาก peak.stages จริงใน challenge_data.json
    let mut groups: BTreeMap<u32, Vec<(u32, &PeakStageData)>> = BTreeMap::new();
    for (k, stage) in &CHALLENGE_DATA.peak {
        if let Ok(id) = k.parse::<u32>() {
            groups.entry(peak_stage_group(id)).or_default().push((id, stage));
        }
    }

    if groups.is_empty() {
        // fallback ถ้าไม่มี peak data
        let mut peak_group = ChallengePeakGroup {
            peak_group_id: 1,
            obtained_stars: 9,
            count_of_peaks: 3,
            disable_hard_mode: false,
            taken_star_rewards: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            ..Default::default()
        };
        for peak_id in 1..=3 {
            peak_group.peaks.push(ChallengePeak {
                peak_id,
                has_passed: true,
                cycles_used: 0,
                finished_target_list: vec![1, 2, 3, 4, 5],
                ..Default::default()
            });
        }
        res.challenge_peak_groups.push(peak_group);
        return;
    }

    for (group_id, mut stages) in groups {
        stages.sort_by_key(|(id, _)| *id);

        let mut peak_group = ChallengePeakGroup {
            peak_group_id: group_id,
            obtained_stars: 0,
            count_of_peaks: stages.len() as u32,
            disable_hard_mode: false,
            taken_star_rewards: Vec::new(),
            ..Default::default()
        };

        for (peak_id, stage) in &stages {
            let stars = stage.targets.len() as u32;
            peak_group.obtained_stars += stars;
            for t in 1..=stars {
                peak_group
                    .taken_star_rewards
                    .push(peak_group.obtained_stars - stars + t);
            }

            peak_group.peaks.push(ChallengePeak {
                peak_id: *peak_id,
                has_passed: true,
                cycles_used: 0,
                finished_target_list: stage.targets.clone(),
                ..Default::default()
            });
        }

        res.challenge_peak_groups.push(peak_group);
    }
}

pub async fn on_get_cur_challenge_peak_cs_req(
    _session: &mut PlayerSession,
    _req: &GetCurChallengePeakCsReq,
    res: &mut GetCurChallengePeakScRsp,
) {
    res.retcode = 0;
}

/// Starward Mode — เริ่มแชลเลนจ์ peak (cmd 8948)
/// StartChallengePeakScRsp (8950) ไม่มี scene field — ส่งฉากผ่าน EnterSceneByServerScNotify แยก
pub async fn on_start_challenge_peak_cs_req(
    session: &mut PlayerSession,
    req: &StartChallengePeakCsReq,
    res: &mut StartChallengePeakScRsp,
) {
    res.retcode = 0;

    let peak_id = req.peak_id;
    let stage = CHALLENGE_DATA.peak.get(&peak_id.to_string());
    let (event_id, monster_id, buff) = match stage {
        Some(s) => (
            s.event_id,
            s.monster_id,
            if req.boss_buff_id != 0 { req.boss_buff_id } else { s.default_buff },
        ),
        None => {
            tracing::warn!("[PEAK] peak_id={peak_id} not found in challenge_data!");
            return;
        }
    };
    tracing::info!(
        "[PEAK] Start: peak_id={peak_id} event={event_id} monster={monster_id} buff={buff}"
    );

    let mut avatars = req.peak_avatar_id_list.clone();
    avatars.dedup();
    avatars.truncate(4);
    if avatars.is_empty() {
        if let Some(json) = session.json_data.get() {
            avatars = json.lineups.values().copied().collect();
        }
    }

    if let Some(json) = session.json_data.get_mut() {
        let mut custom_lineup = BTreeMap::new();
        for (i, &aid) in avatars.iter().enumerate() {
            custom_lineup.insert(i as u32, aid);
        }
        json.battle_config.custom_battle_lineup = Some(custom_lineup);
        json.battle_config.stage_id = event_id;
        json.battle_config.cycle_count = 30;
        json.battle_config.battle_type = BattleType::Default;
        json.battle_config.monsters = vec![vec![Monster {
            level: 68,
            monster_id,
            max_hp: 0,
        }]];
        if buff != 0 {
            json.battle_config.blessings = vec![BattleBuffJson {
                id: buff,
                level: 1,
                dynamic_key: None,
                dynamic_values: Vec::new(),
            }];
        }
        let _ = json.save_persistent().await;
    }

    // ใช้ arena มาตรฐาน 3000101 (group 2 = ตำแหน่งบอส) สำหรับ peak battle
    let scene_result =
        load_challenge_scene(session, 3000101, 2, monster_id, event_id, &avatars).await;
    let Ok((scene_info, _motion)) = scene_result else {
        tracing::error!("[PEAK] Failed to load peak arena scene!");
        return;
    };

    let mut custom_map = BTreeMap::new();
    for (i, &aid) in avatars.iter().enumerate() {
        custom_map.insert(i as u32, aid);
    }
    let lineup_info = AvatarJson::to_lineup_info(&custom_map);

    if let Err(e) = session
        .send(EnterSceneByServerScNotify {
            scene: Some(scene_info),
            lineup: Some(lineup_info),
            ..Default::default()
        })
        .await
    {
        tracing::error!("[PEAK] Failed to send scene notify: {e:?}");
    }
}

pub async fn on_get_cur_challenge_cs_req(
    _session: &mut PlayerSession,
    _req: &GetCurChallengeCsReq,
    res: &mut GetCurChallengeScRsp,
) {
    res.retcode = 0;
    res.cur_challenge = None;
    res.lineup_list = Vec::new();
}

pub async fn on_get_activity_schedule_config_cs_req(
    _session: &mut PlayerSession,
    _req: &GetActivityScheduleConfigCsReq,
    res: &mut GetActivityScheduleConfigScRsp,
) {
    res.retcode = 0;

    let mut schedule_list = Vec::new();

    // Standard activity panels 1..=100
    for id in 1..=100 {
        schedule_list.push(ActivityScheduleData {
            activity_id: id,
            panel_id: id,
            begin_time: 0,
            end_time: 1924992000,
        });
    }

    // Apocalyptic Shadow Activity (21001..=21030, panel 21001, and sub-modules 2100101..=2100801)
    for act_id in 21001..=21030 {
        schedule_list.push(ActivityScheduleData {
            activity_id: act_id,
            panel_id: 21001,
            begin_time: 0,
            end_time: 1924992000,
        });
    }
    for i in 1..=8 {
        schedule_list.push(ActivityScheduleData {
            activity_id: 2100000 + i * 100 + 1, // 2100101, 2100201, ...
            panel_id: 21001,
            begin_time: 0,
            end_time: 1924992000,
        });
    }

    // Challenge and event activity ranges
    for base in [1000, 2000, 3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000, 20000, 30000, 40000, 50000, 60000] {
        for offset in 1..=50 {
            let act_id = base + offset;
            schedule_list.push(ActivityScheduleData {
                activity_id: act_id,
                panel_id: act_id,
                begin_time: 0,
                end_time: 1924992000,
            });
        }
    }

    res.schedule_data = schedule_list;
}

#[derive(Default, Debug)]
pub struct DecodedChallengeStart {
    pub challenge_id: u32,
    pub stage_index: u32,
    pub is_single_stage: bool,
    pub first_avatars: Vec<u32>,
    pub second_avatars: Vec<u32>,
    pub third_avatars: Vec<u32>,
    pub stage_info_list: Vec<Vec<u32>>,
    pub buff_id: u32,
}

fn skip_wire_field(wire_type: u32, buf: &mut &[u8]) {
    match wire_type {
        0 => {
            let _ = prost::encoding::decode_varint(buf);
        }
        1 => {
            if buf.len() >= 8 {
                *buf = &buf[8..];
            } else {
                *buf = &[];
            }
        }
        2 => {
            if let Ok(len) = prost::encoding::decode_varint(buf) {
                let len = len as usize;
                if buf.len() >= len {
                    *buf = &buf[len..];
                } else {
                    *buf = &[];
                }
            } else {
                *buf = &[];
            }
        }
        5 => {
            if buf.len() >= 4 {
                *buf = &buf[4..];
            } else {
                *buf = &[];
            }
        }
        _ => {
            *buf = &[];
        }
    }
}

pub fn decode_start_challenge_tierce_req(payload: &[u8]) -> DecodedChallengeStart {
    let mut req = DecodedChallengeStart::default();
    let mut buf = payload;
    let mut stages_avatars: Vec<(u32, Vec<u32>)> = Vec::new();

    while !buf.is_empty() {
        if let Ok(tag) = prost::encoding::decode_varint(&mut buf) {
            let field_number = (tag >> 3) as u32;
            let wire_type = (tag & 0x7) as u32;
            match (field_number, wire_type) {
                (10, 0) | (1, 0) => {
                    if let Ok(v) = prost::encoding::decode_varint(&mut buf) {
                        req.stage_index = v as u32;
                    }
                }
                (15, 0) | (3, 0) => {
                    if let Ok(v) = prost::encoding::decode_varint(&mut buf) {
                        req.challenge_id = v as u32;
                    }
                }
                (5, 0) | (13, 0) => {
                    if let Ok(v) = prost::encoding::decode_varint(&mut buf) {
                        req.is_single_stage = v != 0;
                    }
                }
                (12, 2) | (14, 2) => {
                    if let Ok(len) = prost::encoding::decode_varint(&mut buf) {
                        let len = len as usize;
                        if len <= buf.len() {
                            let mut sub = &buf[..len];
                            buf = &buf[len..];
                            let mut stage_buff = 0u32;
                            let mut stage_lineup = Vec::new();
                            while !sub.is_empty() {
                                if let Ok(sub_tag) = prost::encoding::decode_varint(&mut sub) {
                                    let sub_fn = (sub_tag >> 3) as u32;
                                    let sub_wt = (sub_tag & 0x7) as u32;
                                    match (sub_fn, sub_wt) {
                                        (6, 0) | (5, 0) => {
                                            if let Ok(v) = prost::encoding::decode_varint(&mut sub) {
                                                stage_buff = v as u32;
                                            }
                                        }
                                        (4, 2) | (9, 2) => {
                                            if let Ok(alen) = prost::encoding::decode_varint(&mut sub) {
                                                let alen = alen as usize;
                                                if alen <= sub.len() {
                                                    let mut abuf = &sub[..alen];
                                                    sub = &sub[alen..];
                                                    let mut aid = 0u32;
                                                    while !abuf.is_empty() {
                                                        if let Ok(atag) = prost::encoding::decode_varint(&mut abuf) {
                                                            let afn = (atag >> 3) as u32;
                                                            let awt = (atag & 0x7) as u32;
                                                            if (afn == 7 || afn == 9) && awt == 0 {
                                                                if let Ok(v) = prost::encoding::decode_varint(&mut abuf) {
                                                                    aid = v as u32;
                                                                }
                                                            } else {
                                                                skip_wire_field(awt, &mut abuf);
                                                            }
                                                        } else {
                                                            break;
                                                        }
                                                    }
                                                    if aid != 0 {
                                                        stage_lineup.push(aid);
                                                    }
                                                }
                                            }
                                        }
                                        _ => {
                                            skip_wire_field(sub_wt, &mut sub);
                                        }
                                    }
                                } else {
                                    break;
                                }
                            }
                            stages_avatars.push((stage_buff, stage_lineup));
                        }
                    }
                }
                _ => {
                    skip_wire_field(wire_type, &mut buf);
                }
            }
        } else {
            break;
        }
    }

    for (_, lineup) in &stages_avatars {
        req.stage_info_list.push(lineup.clone());
    }

    if let Some((buff, lineup)) = stages_avatars.get(0) {
        req.first_avatars = lineup.clone();
        if req.stage_index == 0 && *buff != 0 { req.buff_id = *buff; }
    }
    if let Some((buff, lineup)) = stages_avatars.get(1) {
        req.second_avatars = lineup.clone();
        if req.stage_index == 1 && *buff != 0 { req.buff_id = *buff; }
    }
    if let Some((buff, lineup)) = stages_avatars.get(2) {
        req.third_avatars = lineup.clone();
        if req.stage_index == 2 && *buff != 0 { req.buff_id = *buff; }
    }

    req
}

pub fn decode_set_challenge_tierce_lineup_req(payload: &[u8]) -> (u32, Vec<Vec<u32>>) {
    let mut challenge_id = 0u32;
    let mut stages = Vec::new();
    let mut buf = payload;

    while !buf.is_empty() {
        if let Ok(tag) = prost::encoding::decode_varint(&mut buf) {
            let field_number = (tag >> 3) as u32;
            let wire_type = (tag & 0x7) as u32;
            match (field_number, wire_type) {
                (7, 0) => {
                    if let Ok(v) = prost::encoding::decode_varint(&mut buf) {
                        challenge_id = v as u32;
                    }
                }
                (12, 2) => {
                    if let Ok(len) = prost::encoding::decode_varint(&mut buf) {
                        let len = len as usize;
                        if len <= buf.len() {
                            let mut sub = &buf[..len];
                            buf = &buf[len..];
                            let mut stage_lineup = Vec::new();
                            while !sub.is_empty() {
                                if let Ok(sub_tag) = prost::encoding::decode_varint(&mut sub) {
                                    let sub_fn = (sub_tag >> 3) as u32;
                                    let sub_wt = (sub_tag & 0x7) as u32;
                                    if sub_fn == 9 && sub_wt == 2 {
                                        if let Ok(alen) = prost::encoding::decode_varint(&mut sub) {
                                            let alen = alen as usize;
                                            if alen <= sub.len() {
                                                let mut abuf = &sub[..alen];
                                                sub = &sub[alen..];
                                                let mut aid = 0u32;
                                                while !abuf.is_empty() {
                                                    if let Ok(atag) = prost::encoding::decode_varint(&mut abuf) {
                                                        let afn = (atag >> 3) as u32;
                                                        let awt = (atag & 0x7) as u32;
                                                        if afn == 9 && awt == 0 {
                                                            if let Ok(v) = prost::encoding::decode_varint(&mut abuf) {
                                                                aid = v as u32;
                                                            }
                                                        } else {
                                                            skip_wire_field(awt, &mut abuf);
                                                        }
                                                    } else {
                                                        break;
                                                    }
                                                }
                                                if aid != 0 {
                                                    stage_lineup.push(aid);
                                                }
                                            }
                                        }
                                    } else {
                                        skip_wire_field(sub_wt, &mut sub);
                                    }
                                } else {
                                    break;
                                }
                            }
                            stages.push(stage_lineup);
                        }
                    }
                }
                _ => {
                    skip_wire_field(wire_type, &mut buf);
                }
            }
        } else {
            break;
        }
    }

    (challenge_id, stages)
}

pub fn decode_start_challenge_req(payload: &[u8]) -> DecodedChallengeStart {
    let mut req = DecodedChallengeStart::default();
    let mut buf = payload;

    // 4.6: challenge_id = tag 14, first_lineup = tag 6, second_lineup = tag 8,
    //       avatar_lineup_first = tag 4 (AvatarLineup), stage_info = tag 10 (ChallengeBuffInfo)
    while !buf.is_empty() {
        if let Ok(tag) = prost::encoding::decode_varint(&mut buf) {
            let field_number = (tag >> 3) as u32;
            let wire_type = (tag & 0x7) as u32;
            match (field_number, wire_type) {
                (14, 0) | (12, 0) => {
                    if let Ok(v) = prost::encoding::decode_varint(&mut buf) {
                        if req.challenge_id == 0 {
                            req.challenge_id = v as u32;
                        }
                    }
                }
                (6, 0) => {
                    if let Ok(v) = prost::encoding::decode_varint(&mut buf) {
                        req.first_avatars.push(v as u32);
                    }
                }
                (6, 2) => {
                    if let Ok(len) = prost::encoding::decode_varint(&mut buf) {
                        let len = len as usize;
                        if len <= buf.len() {
                            let mut sub = &buf[..len];
                            buf = &buf[len..];
                            while !sub.is_empty() {
                                if let Ok(v) = prost::encoding::decode_varint(&mut sub) {
                                    req.first_avatars.push(v as u32);
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
                (8, 0) => {
                    if let Ok(v) = prost::encoding::decode_varint(&mut buf) {
                        req.second_avatars.push(v as u32);
                    }
                }
                (8, 2) => {
                    if let Ok(len) = prost::encoding::decode_varint(&mut buf) {
                        let len = len as usize;
                        if len <= buf.len() {
                            let mut sub = &buf[..len];
                            buf = &buf[len..];
                            while !sub.is_empty() {
                                if let Ok(v) = prost::encoding::decode_varint(&mut sub) {
                                    req.second_avatars.push(v as u32);
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
                (4, 2) => {
                    // AvatarLineup: id = tag 1
                    if let Ok(len) = prost::encoding::decode_varint(&mut buf) {
                        let len = len as usize;
                        if len <= buf.len() {
                            let mut sub = &buf[..len];
                            buf = &buf[len..];
                            while !sub.is_empty() {
                                if let Ok(stag) = prost::encoding::decode_varint(&mut sub) {
                                    let sfn = (stag >> 3) as u32;
                                    let swt = (stag & 0x7) as u32;
                                    if sfn == 1 && swt == 0 {
                                        if let Ok(v) = prost::encoding::decode_varint(&mut sub) {
                                            req.first_avatars.push(v as u32);
                                        }
                                    } else {
                                        skip_wire_field(swt, &mut sub);
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
                (10, 2) => {
                    // ChallengeBuffInfo { story_info = 9, boss_info = 10 }
                    // story: buff_one = 10, buff_two = 15 / boss: buff_one = 15, buff_two = 4
                    if let Ok(len) = prost::encoding::decode_varint(&mut buf) {
                        let len = len as usize;
                        if len <= buf.len() {
                            let mut sub = &buf[..len];
                            buf = &buf[len..];
                            while !sub.is_empty() {
                                if let Ok(stag) = prost::encoding::decode_varint(&mut sub) {
                                    let sfn = (stag >> 3) as u32;
                                    let swt = (stag & 0x7) as u32;
                                    if swt == 2 && (sfn == 9 || sfn == 10) {
                                        if let Ok(l2) = prost::encoding::decode_varint(&mut sub) {
                                            let l2 = l2 as usize;
                                            if l2 <= sub.len() {
                                                let mut inner = &sub[..l2];
                                                sub = &sub[l2..];
                                                while !inner.is_empty() {
                                                    if let Ok(t2) =
                                                        prost::encoding::decode_varint(&mut inner)
                                                    {
                                                        let f2 = (t2 >> 3) as u32;
                                                        let w2 = (t2 & 0x7) as u32;
                                                        if w2 == 0 && (f2 == 10 || f2 == 15) {
                                                            if let Ok(v) = prost::encoding::
                                                                decode_varint(&mut inner)
                                                            {
                                                                if req.buff_id == 0 {
                                                                    req.buff_id = v as u32;
                                                                }
                                                            }
                                                        } else {
                                                            skip_wire_field(w2, &mut inner);
                                                        }
                                                    } else {
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        skip_wire_field(swt, &mut sub);
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
                _ => {
                    skip_wire_field(wire_type, &mut buf);
                }
            }
        } else {
            break;
        }
    }

    req
}

pub async fn send_scene_entity_move_sc_notify(
    session: &PlayerSession,
    entity_id: u32,
    entry_id: u32,
    motion: &MotionInfo,
) -> Result<()> {
    let mut body = Vec::new();
    body.push(0x20); // tag 4
    prost::encoding::encode_varint(entity_id as u64, &mut body);

    let mut motion_buf = Vec::new();
    motion.encode(&mut motion_buf)?;
    body.push(0x42); // tag 8
    prost::encoding::encode_varint(motion_buf.len() as u64, &mut body);
    body.extend_from_slice(&motion_buf);

    body.push(0x58); // tag 11
    prost::encoding::encode_varint(entry_id as u64, &mut body);

    session.send_raw(NetPacket {
        cmd_type: 1471,
        head: Vec::new(),
        body,
    }).await?;

    Ok(())
}

pub fn map_challenge_stage_id(id: u32) -> u32 {
    if (5400..=5600).contains(&id) {
        // Map MoC 1035/1036 (5501..5512, 5401..5412) to available 5301..5312
        5300 + (id % 100).clamp(1, 12)
    } else if (20200..=20300).contains(&id) {
        // Map Pure Fiction (20261..20264) to available 20251..20254
        20250 + (id % 10).clamp(1, 4)
    } else if (30200..=30300).contains(&id) {
        // Map Apocalyptic Shadow (30211..30214) to available 30201..30204
        30200 + (id % 10).clamp(1, 4)
    } else if id > 1 && !CHALLENGE_DATA.challenges.contains_key(&id) {
        id - 1
    } else {
        id
    }
}

pub async fn handle_start_challenge_tierce(session: &mut PlayerSession, payload: &[u8]) -> Result<()> {
    let req = decode_start_challenge_tierce_req(payload);
    let base_id = map_challenge_stage_id(req.challenge_id);
    tracing::info!("handle_start_challenge_tierce: challenge_id={} (mapped {}), stage_index={}, is_single_stage={}",
        req.challenge_id, base_id, req.stage_index, req.is_single_stage);

    if !req.stage_info_list.is_empty() {
        if let Ok(mut map) = TIERCE_LINEUPS.lock() {
            map.insert(req.challenge_id, req.stage_info_list.clone());
        }
    }

    let mut chosen_avatars = match req.stage_index {
        0 => req.first_avatars.clone(),
        1 => req.second_avatars.clone(),
        2 => req.third_avatars.clone(),
        _ => Vec::new(),
    };

    if chosen_avatars.is_empty() {
        if let Ok(map) = TIERCE_LINEUPS.lock() {
            if let Some(list) = map.get(&req.challenge_id) {
                if let Some(avatars) = list.get(req.stage_index as usize) {
                    chosen_avatars = avatars.clone();
                }
            }
        }
    }

    if chosen_avatars.is_empty() {
        if let Some(json) = session.json_data.get() {
            chosen_avatars = json.lineups.values().copied().collect();
        }
    }

    let (entrance, group, monster, event, buff) = if req.stage_index == 2 {
        if let Some(t) = CHALLENGE_DATA.tierce.get(&base_id) {
            (t.entrance, t.group, t.monster, t.event, req.buff_id)
        } else if let Some(c) = CHALLENGE_DATA.challenges.get(&base_id) {
            (c.entrance2, c.group2, c.monster2, c.event2, if req.buff_id != 0 { req.buff_id } else { c.buff })
        } else {
            (3014101, 5, 4033010, 30124011, 0)
        }
    } else if req.stage_index == 1 {
        if let Some(c) = CHALLENGE_DATA.challenges.get(&base_id) {
            (c.entrance2, c.group2, c.monster2, c.event2, if req.buff_id != 0 { req.buff_id } else { c.buff })
        } else {
            (3014101, 6, 8013010, 30124012, 0)
        }
    } else {
        if let Some(c) = CHALLENGE_DATA.challenges.get(&base_id) {
            (c.entrance, c.group1, c.monster1, c.event1, if req.buff_id != 0 { req.buff_id } else { c.buff })
        } else {
            (3014101, 5, 4033010, 30124011, 0)
        }
    };

    tracing::info!("[CHALLENGE] Start Tierce: challenge_id={} (mapped to {}), stage_index={}, is_single={}",
        req.challenge_id, base_id, req.stage_index, req.is_single_stage);
    tracing::info!("[CHALLENGE] Arena: entrance={}, group={}, monster={}, event={}, buff={}",
        entrance, group, monster, event, buff);

    if let Some(json) = session.json_data.get_mut() {
        let mut custom_lineup = BTreeMap::new();
        for (i, &aid) in chosen_avatars.iter().enumerate() {
            custom_lineup.insert(i as u32, aid);
        }
        json.battle_config.custom_battle_lineup = Some(custom_lineup);
        json.battle_config.stage_id = event;

        let is_pf = (20000..30000).contains(&req.challenge_id);
        let is_as = req.challenge_id >= 30000;
        json.battle_config.cycle_count = if is_pf { 4 } else { 30 };
        json.battle_config.battle_type = if is_pf { BattleType::PF } else if is_as { BattleType::AS } else { BattleType::Default };

        if let Some(sb) = CHALLENGE_DATA.stages.get(&event) {
            json.battle_config.monsters = sb.monsters.iter().map(|wave| {
                wave.iter().map(|&mid| Monster {
                    level: sb.level,
                    monster_id: mid,
                    max_hp: 0,
                }).collect()
            }).collect();
        }

        if buff != 0 {
            json.battle_config.blessings = vec![BattleBuffJson {
                id: buff,
                level: 1,
                dynamic_key: None,
                dynamic_values: Vec::new(),
            }];
        }

        let _ = json.save_persistent().await;
    }

    let (scene_info, _motion) = load_challenge_scene(session, entrance, group, monster, event, &chosen_avatars).await?;

    // Prepare all lineups for tierce_info
    let mut all_lineups: Vec<Vec<u32>> = Vec::new();

    let first_lineup = if !req.first_avatars.is_empty() {
        req.first_avatars.clone()
    } else if let Some(s) = req.stage_info_list.get(0) {
        s.clone()
    } else if let Ok(map) = TIERCE_LINEUPS.lock() {
        map.get(&req.challenge_id).and_then(|l| l.get(0)).cloned().unwrap_or_default()
    } else {
        Vec::new()
    };
    let first_lineup = if first_lineup.is_empty() { chosen_avatars.clone() } else { first_lineup };
    all_lineups.push(first_lineup.clone());

    if !req.is_single_stage {
        let second_lineup = if !req.second_avatars.is_empty() {
            req.second_avatars.clone()
        } else if let Some(s) = req.stage_info_list.get(1) {
            s.clone()
        } else if let Ok(map) = TIERCE_LINEUPS.lock() {
            map.get(&req.challenge_id).and_then(|l| l.get(1)).cloned().unwrap_or_default()
        } else {
            Vec::new()
        };
        let second_lineup = if second_lineup.is_empty() { first_lineup.clone() } else { second_lineup };
        all_lineups.push(second_lineup);

        let third_lineup = if !req.third_avatars.is_empty() {
            req.third_avatars.clone()
        } else if let Some(s) = req.stage_info_list.get(2) {
            s.clone()
        } else if let Ok(map) = TIERCE_LINEUPS.lock() {
            map.get(&req.challenge_id).and_then(|l| l.get(2)).cloned().unwrap_or_default()
        } else {
            Vec::new()
        };
        if !third_lineup.is_empty() || req.stage_index == 2 {
            all_lineups.push(if third_lineup.is_empty() { first_lineup.clone() } else { third_lineup });
        }
    }

    // Modern 4.5.52 ChallengeTierceChallengeInfo (stage_index: 4, challenge_id: 6, is_single_stage: 14, lineup_list: 15)
    let mut tierce_info = Vec::new();
    tierce_info.push(0x20); // tag 4: stage_index
    prost::encoding::encode_varint(req.stage_index as u64, &mut tierce_info);
    tierce_info.push(0x30); // tag 6: challenge_id
    prost::encoding::encode_varint(req.challenge_id as u64, &mut tierce_info);
    if req.is_single_stage {
        tierce_info.extend_from_slice(&[0x70, 0x01]); // tag 14: is_single_stage = true
    }

    for (st_idx, stage_avas) in all_lineups.iter().enumerate() {
        let mut custom_map = BTreeMap::new();
        for (i, &aid) in stage_avas.iter().enumerate() {
            custom_map.insert(i as u32, aid);
        }
        let mut lineup = AvatarJson::to_lineup_info(&custom_map);
        lineup.plane_id = scene_info.plane_id;
        lineup.extra_lineup_type = ExtraLineupType::LineupChallenge.into();
        lineup.index = st_idx as u32;

        let mut lineup_buf = Vec::new();
        lineup.encode(&mut lineup_buf)?;
        tierce_info.push(0x7A); // tag 15: lineup_list
        prost::encoding::encode_varint(lineup_buf.len() as u64, &mut tierce_info);
        tierce_info.extend_from_slice(&lineup_buf);
    }

    tracing::info!("[CHALLENGE] StartTierce: Encoded {} lineups into tierce_info (is_single={})",
        all_lineups.len(), req.is_single_stage);

    let mut scene_buf = Vec::new();
    scene_info.encode(&mut scene_buf)?;

    // Modern 4.5.52 StartChallengeTierceScRsp (CmdID 8973: retcode: 12, challenge_tierce_info: 4, scene: 9)
    let mut body_8973 = Vec::new();
    body_8973.extend_from_slice(&[0x60, 0x00]); // tag 12: retcode = 0
    body_8973.push(0x22); // tag 4: challenge_tierce_info
    prost::encoding::encode_varint(tierce_info.len() as u64, &mut body_8973);
    body_8973.extend_from_slice(&tierce_info);
    body_8973.push(0x4A); // tag 9: scene
    prost::encoding::encode_varint(scene_buf.len() as u64, &mut body_8973);
    body_8973.extend_from_slice(&scene_buf);

    session.send_raw(NetPacket {
        cmd_type: 8973, // StartChallengeTierceScRsp
        head: Vec::new(),
        body: body_8973,
    }).await?;

    // Note: Do NOT send EnterSceneByServerScNotify (1427) because 8973 already contains SceneInfo!
    Ok(())
}

pub async fn handle_start_challenge(session: &mut PlayerSession, payload: &[u8]) -> Result<()> {
    let req = decode_start_challenge_req(payload);
    tracing::info!("handle_start_challenge: challenge_id={}", req.challenge_id);

    let mut chosen_avatars = if !req.first_avatars.is_empty() {
        req.first_avatars.clone()
    } else if let Some(json) = session.json_data.get() {
        json.lineups.values().copied().collect()
    } else {
        vec![1304, 1313, 1406, 1004]
    };

    // กันทีมเกิน 4 ตัว (client ส่งทั้ง avatar_lineup_first และ first_lineup ซ้ำกันได้
    // ทำให้ TeamManager IndexOutOfRange แล้ว crash ตอนโหลด arena)
    chosen_avatars.dedup();
    chosen_avatars.truncate(4);

    let base_id = map_challenge_stage_id(req.challenge_id);
    let (entrance, group, monster, event, buff) = if let Some(c) = CHALLENGE_DATA.challenges.get(&base_id) {
        (c.entrance, c.group1, c.monster1, c.event1, if req.buff_id != 0 { req.buff_id } else { c.buff })
    } else {
        (3014101, 5, 4033010, 30124011, 0)
    };

    // ใช้ arena จริงตาม challenge data (รอบก่อน crash เพราะทีมเกิน 4 ตัว — แก้แล้ว)
    // fallback กลับ 3000101 (arena มาตรฐาน) ถ้า arena ที่แมปไม่มีใน res
    let entrance = if GAME_RES.level_output_configs.contains_key(&entrance) {
        entrance
    } else {
        tracing::warn!("[CHALLENGE] Arena entrance {entrance} missing in res.json, fallback to 3000101");
        3000101
    };

    tracing::info!("[CHALLENGE] Start: challenge_id={} (mapped to {})", req.challenge_id, base_id);
    tracing::info!("[CHALLENGE] Arena: entrance={}, group={}, monster={}, event={}, buff={}", entrance, group, monster, event, buff);

    if let Some(json) = session.json_data.get_mut() {
        let mut custom_lineup = BTreeMap::new();
        for (i, &aid) in chosen_avatars.iter().enumerate() {
            custom_lineup.insert(i as u32, aid);
        }
        json.battle_config.custom_battle_lineup = Some(custom_lineup);
        json.battle_config.stage_id = event;

        let is_pf = (20000..30000).contains(&req.challenge_id);
        let is_as = req.challenge_id >= 30000;
        json.battle_config.cycle_count = if is_pf { 4 } else { 30 };
        json.battle_config.battle_type = if is_pf { BattleType::PF } else if is_as { BattleType::AS } else { BattleType::Default };

        if let Some(sb) = CHALLENGE_DATA.stages.get(&event) {
            json.battle_config.monsters = sb.monsters.iter().map(|wave| {
                wave.iter().map(|&mid| Monster {
                    level: sb.level,
                    monster_id: mid,
                    max_hp: 0,
                }).collect()
            }).collect();
        }

        if buff != 0 {
            json.battle_config.blessings = vec![BattleBuffJson {
                id: buff,
                level: 1,
                dynamic_key: None,
                dynamic_values: Vec::new(),
            }];
        }

        let _ = json.save_persistent().await;
    }

    let (scene_info, _motion) = load_challenge_scene(session, entrance, group, monster, event, &chosen_avatars).await?;

    let mut custom_map = BTreeMap::new();
    for (i, &aid) in chosen_avatars.iter().enumerate() {
        custom_map.insert(i as u32, aid);
    }
    let lineup_info = AvatarJson::to_lineup_info(&custom_map);

    let cur_challenge = CurChallenge {
        challenge_id: req.challenge_id,
        status: 1, // CHALLENGE_DOING
        round_count: 0,
        score_id: if (20000..30000).contains(&req.challenge_id) { 40000 } else { 0 },
        score_two: 0,
        extra_lineup_type: 1,
        ..Default::default()
    };

    let mut body = Vec::new();
    // StartChallengeScRsp 4.6.51 layout: retcode = 3, cur_challenge = 5, lineup_list = 6, scene = 10
    // tag 3: retcode = 0
    body.extend_from_slice(&[0x18, 0x00]);

    // tag 5: cur_challenge
    let mut chal_buf = Vec::new();
    cur_challenge.encode(&mut chal_buf)?;
    body.push(0x2A);
    prost::encoding::encode_varint(chal_buf.len() as u64, &mut body);
    body.extend_from_slice(&chal_buf);

    // tag 6: lineup_list
    let mut lineup_buf = Vec::new();
    lineup_info.encode(&mut lineup_buf)?;
    body.push(0x32);
    prost::encoding::encode_varint(lineup_buf.len() as u64, &mut body);
    body.extend_from_slice(&lineup_buf);

    // tag 10: scene
    let mut scene_buf = Vec::new();
    scene_info.encode(&mut scene_buf)?;
    body.push(0x52);
    prost::encoding::encode_varint(scene_buf.len() as u64, &mut body);
    body.extend_from_slice(&scene_buf);

    session.send_raw(NetPacket {
        cmd_type: 1787, // 4.6.51 CmdStartChallengeScRsp
        head: Vec::new(),
        body,
    }).await?;

    // Note: Do NOT send EnterSceneByServerScNotify (1427) because 1787 already contains SceneInfo!
    Ok(())
}

pub async fn handle_set_challenge_tierce_lineup(session: &PlayerSession, payload: &[u8]) -> Result<()> {
    let (challenge_id, stages) = decode_set_challenge_tierce_lineup_req(payload);
    if !stages.is_empty() {
        if let Ok(mut map) = TIERCE_LINEUPS.lock() {
            map.insert(challenge_id, stages);
        }
    }
    // 4.5.52 SetChallengeTierceLineupScRsp (CmdID 8999: tag 1 retcode = 0)
    session.send_raw(NetPacket {
        cmd_type: 8999,
        head: Vec::new(),
        body: vec![0x08, 0x00],
    }).await?;
    let _ = session.send_raw(NetPacket {
        cmd_type: 8995,
        head: Vec::new(),
        body: vec![0x60, 0x00], // legacy
    }).await;
    Ok(())
}

pub async fn handle_leave_challenge(session: &mut PlayerSession) -> Result<()> {
    if let Some(json) = session.json_data.get_mut() {
        json.battle_config.custom_battle_lineup = None;
        let _ = json.save_persistent().await;
    }
    // 4.5.52 LeaveChallengeScRsp (CmdID 1716: tag 4 retcode = 0)
    session.send_raw(NetPacket {
        cmd_type: 1716,
        head: Vec::new(),
        body: vec![0x20, 0x00], // tag 4: retcode = 0
    }).await?;
    let _ = session.send_raw(NetPacket {
        cmd_type: 1781,
        head: Vec::new(),
        body: vec![0x40, 0x00], // legacy tag 8: retcode = 0
    }).await;
    Ok(())
}

pub async fn handle_leave_challenge_tierce(session: &mut PlayerSession) -> Result<()> {
    if let Some(json) = session.json_data.get_mut() {
        json.battle_config.custom_battle_lineup = None;
        let _ = json.save_persistent().await;
    }
    // 4.5.52 LeaveChallengeTierceScRsp (CmdID 8997: tag 7 retcode = 0)
    session.send_raw(NetPacket {
        cmd_type: 8997,
        head: Vec::new(),
        body: vec![0x38, 0x00],
    }).await?;
    let _ = session.send_raw(NetPacket {
        cmd_type: 8982,
        head: Vec::new(),
        body: vec![0x68, 0x00], // legacy
    }).await;
    Ok(())
}

pub fn build_get_challenge_tierce_data_sc_rsp() -> Vec<u8> {
    let mut tierce_stages: Vec<(u32, Vec<u32>, u32)> = Vec::new();
    for (&id, data) in &CHALLENGE_DATA.tierce {
        let targets = if !data.targets.is_empty() {
            data.targets.clone()
        } else if id >= 30000 {
            vec![5001, 5002, 5003, 5000]
        } else if id >= 20000 {
            vec![4001, 4002, 4003, 4000]
        } else {
            vec![601, 602, 603, 600]
        };
        let score = if data.score > 0 {
            data.score
        } else if id >= 20000 && id < 30000 {
            40000
        } else {
            4000
        };
        tierce_stages.push((id, targets, score));
    }
    tierce_stages.sort_by_key(|v| v.0);

    let mut rsp = Vec::new();
    // 4.5.52 Tag 8: retcode = 0
    rsp.extend_from_slice(&[0x40, 0x00]);
    // Legacy Tag 12: retcode = 0
    rsp.extend_from_slice(&[0x60, 0x00]);

    for (challenge_id, targets, score) in tierce_stages {
        let mut tierce_data = Vec::new();

        // Tag 3: challenge_id
        tierce_data.push(0x18);
        prost::encoding::encode_varint(challenge_id as u64, &mut tierce_data);

        // Tag 6: is_passed = true
        tierce_data.extend_from_slice(&[0x30, 0x01]);

        // Tag 7: finished_target_list (packed varint)
        let mut targets_buf = Vec::new();
        for t in targets {
            prost::encoding::encode_varint(t as u64, &mut targets_buf);
        }
        tierce_data.push(0x3A);
        prost::encoding::encode_varint(targets_buf.len() as u64, &mut tierce_data);
        tierce_data.extend_from_slice(&targets_buf);

        // Tag 4: result_list (ChallengeTierceStageData: 3 stages: 0, 1, 2)
        for stage_idx in 0..3u32 {
            let mut result = Vec::new();
            // Tag 3: score_id
            result.push(0x18);
            prost::encoding::encode_varint(score as u64, &mut result);
            // Tag 9: stage_index
            result.push(0x48);
            prost::encoding::encode_varint(stage_idx as u64, &mut result);
            // Tag 10: end_status = 1 (BATTLE_END_WIN)
            result.extend_from_slice(&[0x50, 0x01]);

            tierce_data.push(0x22); // Tag 4 (len-delimited)
            prost::encoding::encode_varint(result.len() as u64, &mut tierce_data);
            tierce_data.extend_from_slice(&result);
        }

        // Tag 8: stage_info_list (ChallengeTierceStageInfo: 3 stages: 0, 1, 2)
        for stage_idx in 0..3u32 {
            let mut sinfo = Vec::new();
            // Tag 14: stage_index
            sinfo.push(0x70);
            prost::encoding::encode_varint(stage_idx as u64, &mut sinfo);

            tierce_data.push(0x42); // Tag 8 (len-delimited)
            prost::encoding::encode_varint(sinfo.len() as u64, &mut tierce_data);
            tierce_data.extend_from_slice(&sinfo);
        }

        // Tag 4: challenge_info_list (4.5.52)
        rsp.push(0x22);
        prost::encoding::encode_varint(tierce_data.len() as u64, &mut rsp);
        rsp.extend_from_slice(&tierce_data);

        // Tag 10: challenge_info_list (legacy)
        rsp.push(0x52);
        prost::encoding::encode_varint(tierce_data.len() as u64, &mut rsp);
        rsp.extend_from_slice(&tierce_data);
    }

    rsp
}

pub async fn handle_get_cur_challenge(session: &PlayerSession) -> Result<()> {
    session.send_raw(NetPacket {
        cmd_type: 1771,
        head: Vec::new(),
        body: vec![0x68, 0x00], // tag 13: retcode = 0
    }).await?;
    Ok(())
}

pub async fn handle_take_challenge_reward(session: &PlayerSession, payload: &[u8]) -> Result<()> {
    let mut group_id = 0u32;
    let mut buf = payload;
    while !buf.is_empty() {
        if let Ok(tag) = prost::encoding::decode_varint(&mut buf) {
            let fn_num = tag >> 3;
            let wt = tag & 0x7;
            if fn_num == 4 && wt == 0 {
                if let Ok(v) = prost::encoding::decode_varint(&mut buf) {
                    group_id = v as u32;
                }
                break;
            } else {
                skip_wire_field(wt as u32, &mut buf);
            }
        } else {
            break;
        }
    }

    let mut body = Vec::new();
    // tag 2: group_id
    body.push(0x10);
    prost::encoding::encode_varint(group_id as u64, &mut body);
    // tag 6: retcode = 0
    body.extend_from_slice(&[0x30, 0x00]);

    session.send_raw(NetPacket {
        cmd_type: 1704,
        head: Vec::new(),
        body,
    }).await?;
    Ok(())
}


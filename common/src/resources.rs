use serde::Deserialize;
use std::{collections::HashMap, fs, sync::LazyLock};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vector {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Deserialize, Copy, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[repr(u32)]
pub enum PropState {
    Closed = 0,
    Open = 1,
    Locked = 2,
    BridgeState1 = 3,
    BridgeState2 = 4,
    BridgeState3 = 5,
    BridgeState4 = 6,
    CheckPointDisable = 7,
    CheckPointEnable = 8,
    TriggerDisable = 9,
    TriggerEnable = 10,
    ChestLocked = 11,
    ChestClosed = 12,
    ChestUsed = 13,
    Elevator1 = 14,
    Elevator2 = 15,
    Elevator3 = 16,
    WaitActive = 17,
    EventClose = 18,
    EventOpen = 19,
    Hidden = 20,
    TeleportGate0 = 21,
    TeleportGate1 = 22,
    TeleportGate2 = 23,
    TeleportGate3 = 24,
    Destructed = 25,
    CustomState01 = 101,
    CustomState02 = 102,
    CustomState03 = 103,
    CustomState04 = 104,
    CustomState05 = 105,
    CustomState06 = 106,
    CustomState07 = 107,
    CustomState08 = 108,
    CustomState09 = 109,
}

#[derive(Deserialize, Copy, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[repr(u32)]
pub enum PlaneType {
    Unknown = 0,
    Maze = 2,
    Train = 3,
    Challenge = 4,
    Rogue = 5,
    Raid = 6,
    AetherDivide = 7,
    TrialActivity = 8,
    Town = 1,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneMonsterInfo {
    pub pos: Vector,
    pub rot: Vector,
    pub group_id: u32,
    pub inst_id: u32,
    pub monster_id: u32,
    pub event_id: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneNpcInfo {
    pub pos: Vector,
    pub rot: Vector,
    pub group_id: u32,
    pub inst_id: u32,
    pub npc_id: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenePropInfo {
    pub pos: Vector,
    pub rot: Vector,
    pub group_id: u32,
    pub inst_id: u32,
    pub prop_state: u32,
    pub prop_id: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeleportInfo {
    pub pos: Vector,
    pub rot: Vector,
    // pub group_id: u32,
    // pub inst_id: u32,
    // pub anchor_id: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneData {
    pub npcs: Vec<SceneNpcInfo>,
    pub props: Vec<ScenePropInfo>,
    pub monsters: Vec<SceneMonsterInfo>,
    pub teleports: HashMap<u32, TeleportInfo>,
    pub finished_sub_missions: Vec<u32>,
    pub finished_main_missions: Vec<u32>,
    pub chests: Vec<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelOutputConfig {
    pub is_entered_scene_info: bool,
    pub scenes: HashMap<u32, SceneData>,
    pub plane_type: u32,
    pub world_id: u32,
    pub sections: Vec<u32>,
    pub saved_values: HashMap<String, i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvatarConfig {
    pub weakness_buff_id: u32,
    // pub technique_buff_ids: Vec<u32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JsonConfig {
    /// `entryid` -> `P[planeId]_F[floorId]` -> `groupId`
    pub level_output_configs: HashMap<u32, HashMap<String, LevelOutputConfig>>,
    pub avatar_configs: HashMap<u32, AvatarConfig>,
    pub map_default_entrance_map: HashMap<u32, u32>,
    pub relic_avatar_recommend: HashMap<u32, Vec<u32>>,
    /// รายชื่อ skin id ที่ปลดล็อกทั้งหมด (แสดงในหน้าเลือกชุดของ client)
    #[serde(default)]
    pub skin_ids: Vec<u32>,
}

pub static GAME_RES: LazyLock<JsonConfig> = LazyLock::new(|| {
    serde_json::from_str::<JsonConfig>(&fs::read_to_string("res.json").unwrap()).unwrap()
});

/// teleport ทุกแมพจาก teleports.json — {floor_id: {teleport_id: info}}
/// ใช้ fallback เมื่อฉากใน res.json ไม่มี teleport (เช่นฉาก arena)
pub static TELEPORT_DB: LazyLock<HashMap<u32, HashMap<u32, TeleportInfo>>> = LazyLock::new(|| {
    std::fs::read_to_string("teleports.json")
        .ok()
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default()
});

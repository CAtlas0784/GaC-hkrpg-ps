use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::{monster::Monster, relic::SubAffix};

// BATTLE CONFIG
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BattleConfig {
    pub battle_type: BattleType,
    pub monsters: Vec<Vec<Monster>>,
    pub blessings: Vec<BattleBuffJson>,
    pub stage_id: u32,
    pub cycle_count: u32,
    pub path_resonance_id: u32,
    pub custom_stats: Vec<SubAffix>,
    #[serde(default)]
    pub scepters: Vec<RogueMagicScepter>,
    #[serde(default)]
    pub custom_battle_lineup: Option<BTreeMap<u32, u32>>,
}

impl Default for BattleConfig {
    fn default() -> Self {
        Self {
            battle_type: Default::default(),
            monsters: vec![vec![Monster {
                level: 60,
                monster_id: 3014022,
                max_hp: 0,
            }]],
            stage_id: 201012311,
            blessings: Default::default(),
            cycle_count: Default::default(),
            path_resonance_id: Default::default(),
            custom_stats: Default::default(),
            scepters: Default::default(),
            custom_battle_lineup: Default::default(),
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum BattleType {
    #[serde(alias = "DEFAULT")]
    #[default]
    Default = 0,
    #[serde(alias = "MOC")]
    Moc = 1,
    PF = 2,
    SU = 3,
    AS = 4,
    AA = 5,
}

// BATTLE BUFFS
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BattleBuffJson {
    pub level: u32,
    pub id: u32,
    pub dynamic_key: Option<DynamicKey>,
    #[serde(default)]
    pub dynamic_values: Vec<DynamicKey>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DynamicKey {
    pub key: String,
    pub value: u32,
}

#[allow(dead_code)]
impl BattleBuffJson {
    pub fn to_battle_buff_proto(&self) -> proto::BattleBuff {
        proto::BattleBuff {
            id: self.id,
            level: self.level,
            wave_flag: 0xffffffff,
            owner_id: 0xffffffff,
            dynamic_values: if let Some(dyn_key) = &self.dynamic_key {
                HashMap::from([(dyn_key.key.clone(), dyn_key.value as f32)])
            } else {
                Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default)]
#[repr(u32)]
pub enum RogueMagicComponentType {
    Passive = 3,
    #[default]
    Active = 4,
    Attach = 5,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RogueMagicScepter {
    pub level: u32,
    pub id: u32,
    pub components: Vec<RogueMagicComponent>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RogueMagicComponent {
    pub id: u32,
    pub level: u32,
    pub component_type: RogueMagicComponentType,
}

use std::collections::{BTreeMap, HashMap};

use proto::{
    Avatar, AvatarPathData, AvatarPathSkillTree, AvatarSkillTree, AvatarType, BattleAvatar,
    BattleBuff, ExtraLineupType, Gender, LineupAvatar, LineupInfo, MultiPathAvatarType, SpBarInfo,
};
use serde::{Deserialize, Serialize};

use crate::sr_tools::FreesrData;

use super::{lightcone::Lightcone, relic::Relic};

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum MultiPathAvatar {
    MalePyhsical = 8001,
    FemalePhysical = 8002,
    MalePreservation = 8003,
    FemalePreservation = 8004,
    MaleHarmony = 8005,
    FemaleHarmony = 8006,
    MaleRemembrance = 8007,
    FemaleRemembrance = 8008,
    MaleElation = 8009,
    FemaleElation = 8010,
    MarchHunt = 1224,
    MarchPreservation = 1001,
    #[default]
    Unk = 0,
}

impl From<u32> for MultiPathAvatar {
    fn from(value: u32) -> Self {
        match value {
            8001 => Self::MalePyhsical,
            8002 => Self::FemalePhysical,
            8003 => Self::MalePreservation,
            8004 => Self::FemalePreservation,
            8005 => Self::MaleHarmony,
            8006 => Self::FemaleHarmony,
            8007 => Self::MaleRemembrance,
            8008 => Self::FemaleRemembrance,
            8009 => Self::MaleElation,
            8010 => Self::FemaleElation,
            1224 => Self::MarchHunt,
            1001 => Self::MarchPreservation,
            _ => Self::Unk,
        }
    }
}

impl From<MultiPathAvatar> for u32 {
    fn from(value: MultiPathAvatar) -> Self {
        match value {
            MultiPathAvatar::MalePyhsical => 8001,
            MultiPathAvatar::FemalePhysical => 8002,
            MultiPathAvatar::MalePreservation => 8003,
            MultiPathAvatar::FemalePreservation => 8004,
            MultiPathAvatar::MaleHarmony => 8005,
            MultiPathAvatar::FemaleHarmony => 8006,
            MultiPathAvatar::MaleRemembrance => 8007,
            MultiPathAvatar::FemaleRemembrance => 8008,
            MultiPathAvatar::MaleElation => 8009,
            MultiPathAvatar::FemaleElation => 8010,
            MultiPathAvatar::MarchHunt => 1224,
            MultiPathAvatar::MarchPreservation => 1001,
            MultiPathAvatar::Unk => 8006,
        }
    }
}

impl MultiPathAvatar {
    #[allow(unused)]
    pub fn get_gender(&self) -> Gender {
        if (*self as u32) < 8000 {
            Gender::None
        } else if *self as u32 % 2 == 1 {
            Gender::Man
        } else {
            Gender::Woman
        }
    }

    pub fn get_type(&self) -> MultiPathAvatarType {
        match *self {
            MultiPathAvatar::MalePyhsical => MultiPathAvatarType::BoyWarriorType,
            MultiPathAvatar::FemalePhysical => MultiPathAvatarType::GirlWarriorType,
            MultiPathAvatar::MalePreservation => MultiPathAvatarType::BoyKnightType,
            MultiPathAvatar::FemalePreservation => MultiPathAvatarType::GirlKnightType,
            MultiPathAvatar::MaleHarmony => MultiPathAvatarType::BoyShamanType,
            MultiPathAvatar::FemaleHarmony => MultiPathAvatarType::GirlShamanType,
            MultiPathAvatar::MarchHunt => MultiPathAvatarType::Mar7thRogueType,
            MultiPathAvatar::MarchPreservation => MultiPathAvatarType::Mar7thKnightType,
            MultiPathAvatar::Unk => MultiPathAvatarType::None,
            MultiPathAvatar::MaleRemembrance => MultiPathAvatarType::BoyMemoryType,
            MultiPathAvatar::FemaleRemembrance => MultiPathAvatarType::GirlMemoryType,
            MultiPathAvatar::MaleElation => MultiPathAvatarType::BoyElationType,
            MultiPathAvatar::FemaleElation => MultiPathAvatarType::GirlElationType,
        }
    }

    pub fn is_mc(&self) -> bool {
        (*self as u32) > 8000
    }

    pub fn to_vec() -> Vec<MultiPathAvatar> {
        vec![
            Self::MalePyhsical,
            Self::FemalePhysical,
            Self::MalePreservation,
            Self::FemalePreservation,
            Self::MaleHarmony,
            Self::FemaleHarmony,
            Self::MaleRemembrance,
            Self::FemaleRemembrance,
            Self::MaleElation,
            Self::FemaleElation,
            Self::MarchHunt,
            Self::MarchPreservation,
        ]
    }

    pub fn is_main_character(avatar_id: u32) -> bool {
        (8001..=8010).contains(&avatar_id)
    }

    pub fn is_march_7th(avatar_id: u32) -> bool {
        avatar_id == 1001 || avatar_id == 1224
    }

    pub fn base_avatar_id(avatar_id: u32) -> u32 {
        if Self::is_main_character(avatar_id) {
            8001
        } else if Self::is_march_7th(avatar_id) {
            1001
        } else {
            avatar_id
        }
    }
}

// AVATAR
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AvatarJson {
    // #[serde(alias = "ownerUid")]
    // pub owner_uid: u32,
    #[serde(alias = "avatarId")]
    pub avatar_id: u32,
    pub data: AvatarData,
    pub level: u32,
    pub promotion: u32,
    #[serde(alias = "use_technique")]
    #[serde(alias = "useTechnique")]
    pub techniques: Vec<u32>,
    #[serde(alias = "spValue")]
    pub sp_value: Option<u32>,
    #[serde(alias = "spMax")]
    pub sp_max: Option<u32>,
    pub enhanced_id: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AvatarData {
    pub rank: u32,
    pub skills: HashMap<u32, u32>,
    #[serde(default)]
    pub skills_by_anchor_type: HashMap<u32, u32>,
}

impl AvatarJson {
    /// This should only be used to serialize "BaseAvatar". For example, only 8001 for MC and 1001 for March
    pub fn to_avatar_proto(
        &self,
        lightcone: Option<&Lightcone>,
        mc_id: u32,
        march_id: u32,
    ) -> Avatar {
        let base_avatar_id = MultiPathAvatar::base_avatar_id(self.avatar_id);
        let cur_multi_path_avatar_type = if MultiPathAvatar::is_main_character(self.avatar_id) {
            mc_id
        } else if MultiPathAvatar::is_march_7th(self.avatar_id) {
            march_id
        } else {
            base_avatar_id
        };

        Avatar {
            base_avatar_id,
            level: self.level,
            promotion: self.promotion,
            equipment_unique_id: lightcone.map(|v| v.get_unique_id()).unwrap_or_default(),
            first_met_timestamp: 1712924677,
            cur_multi_path_avatar_type,
            has_taken_promotion_reward_list: vec![1, 3, 5],
            is_marked: false,
            exp: 0,
        }
    }

    pub fn to_avatar_path_data_proto(
        &self,
        lightcone: Option<&Lightcone>,
        relics: Vec<&Relic>,
    ) -> AvatarPathData {
        AvatarPathData {
            avatar_id: self.avatar_id,
            rank: self.data.rank,
            equip_relic_list: relics
                .iter()
                .filter(|relic| relic.equip_avatar == self.avatar_id)
                .map(|&relic| relic.into())
                .collect(),
            avatar_path_skill_tree: self
                .data
                .skills_by_anchor_type
                .iter()
                .map(|(&anchor_type, &level)| AvatarPathSkillTree { point_id: anchor_type, level })
                .collect(),
            path_equipment_id: lightcone.map(|v| v.get_unique_id()).unwrap_or_default(),
            enhanced_id: self.enhanced_id.unwrap_or_default(),
            unlock_timestamp: 0,
            dressed_skin_id: 0,
            ..Default::default()
        }
    }

    pub fn to_battle_avatar_proto(
        &self,
        index: u32,
        lightcone: Option<&Lightcone>,
        relics: Vec<&Relic>,
    ) -> (BattleAvatar, Vec<BattleBuff>) {
        let battle_avatar = BattleAvatar {
            index,
            avatar_type: AvatarType::AvatarUpgradeAvailableType.into(),
            id: self.avatar_id,
            level: self.level,
            rank: self.data.rank,
            skilltree_list: self
                .data
                .skills
                .iter()
                .map(|v| AvatarSkillTree {
                    point_id: *v.0,
                    level: *v.1,
                })
                .collect::<Vec<_>>(),
            equipment_list: if let Some(lc) = lightcone {
                vec![lc.into()]
            } else {
                vec![]
            },
            hp: 10_000,
            promotion: self.promotion,
            relic_list: relics.iter().map(|v| (*v).into()).collect::<Vec<_>>(),
            world_level: 6,
            sp_bar: Some(SpBarInfo {
                cur_sp: self.sp_value.unwrap_or(10_000),
                max_sp: self.sp_max.unwrap_or(10_000),
            }),
            enhanced_id: self.enhanced_id.unwrap_or_default(),
            ..Default::default()
        };

        let mut battle_buff = Vec::<BattleBuff>::new();
        for buff_id in &self.techniques {
            battle_buff.push(BattleBuff {
                wave_flag: 0xffffffff,
                owner_id: index,
                level: 1,
                id: *buff_id,
                dynamic_values: HashMap::from([(String::from("SkillIndex"), 2.0)]),
                ..Default::default()
            });
        }

        (battle_avatar, battle_buff)
    }

    pub fn to_lineup_avatar_proto(&self, slot: u32) -> LineupAvatar {
        LineupAvatar {
            id: self.avatar_id,
            hp: 10_000,
            satiety: 100,
            avatar_type: AvatarType::AvatarFormalType.into(),
            sp_bar: Some(SpBarInfo {
                cur_sp: self.sp_value.unwrap_or(10_000),
                max_sp: self.sp_max.unwrap_or(10_000),
            }),
            slot,
        }
    }

    pub fn to_lineup_avatars(player: &FreesrData) -> Vec<LineupAvatar> {
        let avatar_ids = player
            .avatars
            .values()
            .map(|v| &v.avatar_id)
            .collect::<Vec<_>>();

        player
            .lineups
            .iter()
            .filter(|(slot, v)| **slot < 4 && v > &&0 && avatar_ids.contains(v))
            .map(|(slot, avatar_id)| {
                player
                    .avatars
                    .get(avatar_id)
                    .unwrap()
                    .to_lineup_avatar_proto(*slot)
            })
            .collect::<Vec<LineupAvatar>>()
    }

    pub fn to_lineup_info(lineups: &BTreeMap<u32, u32>) -> LineupInfo {
        let max_mp = if lineups.contains_key(&1408) { 8 } else { 5 };
        let mut lineup_info = LineupInfo {
            extra_lineup_type: ExtraLineupType::LineupNone.into(),
            name: "Squad 1".to_string(),
            mp: max_mp,
            max_mp,
            ..Default::default()
        };

        for id in lineups.values() {
            if *id == 0 {
                continue;
            }
            lineup_info.avatar_list.push(LineupAvatar {
                id: *id,
                hp: 10_000,
                satiety: 100,
                avatar_type: AvatarType::AvatarFormalType.into(),
                sp_bar: Some(SpBarInfo {
                    cur_sp: 10_000,
                    max_sp: 10_000,
                }),
                slot: lineup_info.avatar_list.len() as u32,
            });
        }

        lineup_info
    }
}

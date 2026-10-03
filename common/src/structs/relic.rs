use proto::{BattleRelic, EquipRelic, ItemType, RelicAffix};
use serde::{Deserialize, Serialize};

use crate::impl_from;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Relic {
    pub level: u32,
    #[serde(alias = "relicId")]
    pub relic_id: u32,
    // #[serde(alias = "relicSetId")]
    // pub relic_set_id: u32,
    #[serde(alias = "mainAffixId")]
    pub main_affix_id: u32,
    #[serde(alias = "subAffixes")]
    pub sub_affixes: Vec<SubAffix>,
    #[serde(alias = "internalUid")]
    pub internal_uid: u32,
    #[serde(alias = "equipAvatar")]
    pub equip_avatar: u32,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SubAffix {
    #[serde(alias = "subAffixId")]
    pub sub_affix_id: u32,
    pub count: u32,
    pub step: u32,
}

impl Relic {
    pub fn is_matching_slot(&self, slot: u32) -> bool {
        self.get_slot() == slot
    }

    pub fn get_slot(&self) -> u32 {
        self.relic_id % 10
    }

    pub fn get_unique_id(&self) -> u32 {
        super::get_item_unique_id(self.internal_uid, ItemType::ItemRelic)
    }
}

impl_from!(SubAffix, RelicAffix, |v| {
    RelicAffix {
        affix_id: v.sub_affix_id,
        cnt: v.count,
        step: v.step,
    }
});

impl_from!(Relic, BattleRelic, |value| {
    BattleRelic {
        id: value.relic_id,
        level: value.level,
        main_affix_id: value.main_affix_id,
        unique_id: value.get_unique_id(),
        sub_affix_list: value
            .sub_affixes
            .iter()
            .map(|v| v.into())
            .collect::<Vec<_>>(),
        ..Default::default()
    }
});

impl_from!(Relic, EquipRelic, |value| {
    EquipRelic {
        relic_slot: value.get_slot(),
        relic_unique_id: value.get_unique_id(),
    }
});

impl_from!(Relic, proto::Relic, |value| {
    proto::Relic {
        equip_avatar_id: value.equip_avatar,
        exp: 0,
        is_protected: false,
        level: value.level,
        main_affix_id: value.main_affix_id,
        tid: value.relic_id,
        unique_id: value.get_unique_id(),
        sub_affix_list: value
            .sub_affixes
            .iter()
            .map(|v| RelicAffix {
                affix_id: v.sub_affix_id,
                cnt: v.count,
                step: v.step,
            })
            .collect::<Vec<_>>(),
        ..Default::default()
    }
});

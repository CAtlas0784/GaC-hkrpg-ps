use proto::{BattleEquipment, Equipment, ItemType};
use serde::{Deserialize, Serialize};

use crate::impl_from;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Lightcone {
    pub level: u32,
    #[serde(alias = "itemId")]
    pub item_id: u32,
    #[serde(alias = "equipAvatar")]
    pub equip_avatar: u32,
    pub rank: u32,
    pub promotion: u32,
    #[serde(alias = "internalUid")]
    pub internal_uid: u32,
}

impl_from!(Lightcone, Equipment, |value| {
    Equipment {
        equip_avatar_id: value.equip_avatar,
        exp: 0,
        is_protected: false,
        level: value.level,
        promotion: value.promotion,
        rank: value.rank,
        tid: value.item_id,
        unique_id: value.get_unique_id(),
    }
});

impl_from!(Lightcone, BattleEquipment, |value| {
    BattleEquipment {
        id: value.item_id,
        level: value.level,
        promotion: value.promotion,
        rank: value.rank,
    }
});

impl Lightcone {
    pub fn get_unique_id(&self) -> u32 {
        super::get_item_unique_id(self.internal_uid, ItemType::ItemEquipment)
    }
}

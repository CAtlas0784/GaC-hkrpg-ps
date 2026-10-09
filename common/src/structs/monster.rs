use proto::{SceneMonster, SceneMonsterWave, SceneMonsterWaveParam};
use serde::{Deserialize, Serialize};

use crate::impl_from;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Monster {
    pub level: u32,
    #[serde(alias = "monsterId")]
    pub monster_id: u32,
    #[serde(default)]
    pub max_hp: u32,
}

impl_from!(Monster, SceneMonster, |value| {
    SceneMonster {
        monster_id: value.monster_id,
        max_hp: value.max_hp,
        cur_hp: value.max_hp,
        extra_info: None,
    }
});

impl Monster {
    pub fn to_scene_monster_wave(wave_id: u32, monsters: &[Self]) -> SceneMonsterWave {
        SceneMonsterWave {
            wave_id,
            wave_param: Some(SceneMonsterWaveParam {
                level: monsters.iter().map(|v| v.level).max().unwrap_or(95),
                ..Default::default()
            }),

            monster_list: monsters.iter().map(|v| v.into()).collect(),
            ..Default::default()
        }
    }

    pub fn to_scene_monster_waves(monsters: &[Vec<Self>]) -> Vec<SceneMonsterWave> {
        monsters
            .iter()
            .enumerate()
            .map(|(i, v)| Self::to_scene_monster_wave((i + 1) as u32, v))
            .collect::<_>()
    }
}

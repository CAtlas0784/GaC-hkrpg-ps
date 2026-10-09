use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::{
    avatar::MultiPathAvatar,
    scene::{Position, Scene},
};

// #[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq, Default)]
// #[repr(u32)]
// pub enum AllowedLanguages {
//     #[default]
//     En,
//     Jp,
//     Kr,
//     Cn,
// }

// impl FromStr for AllowedLanguages {
//     type Err = ();

//     fn from_str(input: &str) -> Result<AllowedLanguages, Self::Err> {
//         match input {
//             "en" => Ok(AllowedLanguages::En),
//             "kr" => Ok(AllowedLanguages::Kr),
//             "jp" => Ok(AllowedLanguages::Jp),
//             "cn" => Ok(AllowedLanguages::Cn),
//             _ => Ok(AllowedLanguages::En),
//         }
//     }
// }

#[derive(Debug, Serialize, Deserialize)]
pub struct Persistent {
    #[serde(default)]
    pub lineups: BTreeMap<u32, u32>,
    #[serde(default)]
    pub position: Position,
    #[serde(default)]
    pub scene: Scene,
    pub main_character: MultiPathAvatar,
    pub march_type: MultiPathAvatar,
    // pub game_language: AllowedLanguages,
    // pub voice_language: AllowedLanguages,
    #[serde(default = "default_true")]
    pub enable_sw_global: Option<bool>,
    #[serde(default = "default_true")]
    pub enable_castorice_global: Option<bool>,
    // avatar_id -> dressed skin_id
    #[serde(default)]
    pub dressed_skins: HashMap<u32, u32>,
}

fn default_true() -> Option<bool> {
    Some(true)
}

impl Default for Persistent {
    fn default() -> Self {
        Self {
            lineups: BTreeMap::from([(0, 1313), (1, 1006), (2, 8001), (3, 1405)]),
            position: Default::default(),
            main_character: MultiPathAvatar::FemaleRemembrance,
            scene: Default::default(),
            march_type: MultiPathAvatar::MarchHunt,
            // game_language: AllowedLanguages::En,
            // voice_language: AllowedLanguages::Jp,
            enable_sw_global: Some(true),
            enable_castorice_global: Some(true),
            dressed_skins: HashMap::new(),
        }
    }
}

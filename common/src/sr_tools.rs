use proto::{Avatar, AvatarPathData};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::structs::{
    // AllowedLanguages,
    avatar::{AvatarJson, MultiPathAvatar},
    battle::BattleConfig,
    lightcone::Lightcone,
    persistent::Persistent,
    relic::Relic,
    scene::{Position, Scene},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FreesrData {
    pub lightcones: Vec<Lightcone>,
    pub relics: Vec<Relic>,
    pub avatars: HashMap<u32, AvatarJson>,
    #[serde(default)]
    pub battle_config: BattleConfig,

    // Non freesr-data.json fields
    #[serde(skip_serializing, skip_deserializing)]
    pub lineups: BTreeMap<u32, u32>,
    #[serde(skip_serializing, skip_deserializing)]
    pub position: Position,
    #[serde(skip_serializing, skip_deserializing)]
    pub scene: Scene,
    #[serde(skip_serializing, skip_deserializing)]
    pub main_character: MultiPathAvatar,
    #[serde(skip_serializing, skip_deserializing)]
    pub march_type: MultiPathAvatar,
    #[serde(skip_serializing, skip_deserializing)]
    pub enable_sw_global: Option<bool>,
    #[serde(skip_serializing, skip_deserializing)]
    pub enable_castorice_global: Option<bool>,
    #[serde(skip_serializing, skip_deserializing)]
    pub dressed_skins: HashMap<u32, u32>,
}

impl FreesrData {
    pub fn get_avatar_proto(&self, avatar_id: u32, mc_id: u32, march_id: u32) -> Option<Avatar> {
        let avatar = self.avatars.get(&avatar_id)?;
        let lightcone = self.lightcones.iter().find(|l| l.equip_avatar == avatar_id);
        Some(avatar.to_avatar_proto(lightcone, mc_id, march_id))
    }

    pub fn get_avatar_path_data_proto(&self, avatar_id: u32) -> Option<AvatarPathData> {
        let avatar = self.avatars.get(&avatar_id)?;

        Some(
            avatar.to_avatar_path_data_proto(
                self.lightcones.iter().find(|v| v.equip_avatar == avatar_id),
                self.relics
                    .iter()
                    .filter(|relic| relic.equip_avatar == avatar_id)
                    .collect(),
                self.dressed_skins.get(&avatar_id).copied().unwrap_or(0),
            ),
        )
    }
}

impl FreesrData {
    pub async fn load() -> anyhow::Result<Self> {
        let mut freesr_data: FreesrData = tokio::fs::read_to_string("freesr-data.json")
            .await
            .map_err(|e| anyhow::anyhow!("failed to read freesr-data.json: {e}"))
            .and_then(|v| {
                serde_json::from_str::<FreesrData>(&v)
                    .map_err(|e| anyhow::anyhow!("freesr-data.json is broken: {e}, pls redownload"))
            })?;

        let persistent: Persistent = serde_json::from_str(
            &tokio::fs::read_to_string("persistent")
                .await
                .unwrap_or_default(),
        )
        .unwrap_or_default();

        freesr_data.lineups = persistent.lineups;
        freesr_data.position = persistent.position;
        freesr_data.scene = persistent.scene;
        freesr_data.main_character = persistent.main_character;
        freesr_data.march_type = persistent.march_type;
        freesr_data.enable_sw_global = persistent.enable_sw_global;
        freesr_data.enable_castorice_global = persistent.enable_castorice_global;
        freesr_data.dressed_skins = persistent.dressed_skins;
        // freesr_data.game_language = persistent.game_language;
        // freesr_data.voice_langauge = persistent.voice_language;

        // remove unequipped lightcones
        if freesr_data.lightcones.len() > 1500 {
            freesr_data.lightcones.retain(|v| v.equip_avatar != 0);
        }

        // remove unequipped relics
        if freesr_data.relics.len() > 3000 {
            freesr_data.relics.retain(|v| v.equip_avatar != 0);
        }

        freesr_data.verify_lineup().await;

        if freesr_data.march_type as u32 > 8000 {
            freesr_data.march_type = MultiPathAvatar::MarchHunt;
        }

        Ok(freesr_data)
    }

    pub async fn update(&mut self) -> anyhow::Result<()> {
        *self = Self::load().await?;
        Ok(())
    }

    // pub async fn refresh_persistent(&mut self) -> anyhow::Result<()> {
    //     let persistent: Persistent = serde_json::from_str(
    //         &tokio::fs::read_to_string("persistent")
    //             .await
    //             .unwrap_or_default(),
    //     )
    //     .unwrap_or_default();

    //     self.lineups = persistent.lineups;
    //     self.position = persistent.position;
    //     self.scene = persistent.scene;
    //     self.main_character = persistent.main_character;
    //     self.march_type = persistent.march_type;
    //     self.enable_sw_global = persistent.enable_sw_global;
    //     self.enable_castorice_global = persistent.enable_castorice_global;

    //     Ok(())
    // }

    async fn verify_lineup(&mut self) {
        if self.lineups.is_empty() {
            self.lineups = BTreeMap::<u32, u32>::from([(0, 8001), (1, 0), (2, 0), (3, 0)])
        } else if self.lineups.len() < 4 {
            for i in self.lineups.len()..4 {
                self.lineups.insert(i as u32, 0);
            }
        }
        self.save_persistent().await;
    }

    pub async fn save_persistent(&self) {
        if let Ok(data) = serde_json::to_string_pretty(&Persistent {
            lineups: self.lineups.clone(),
            main_character: self.main_character,
            position: self.position.clone(),
            scene: self.scene.clone(),
            march_type: self.march_type,
            enable_sw_global: self.enable_sw_global,
            enable_castorice_global: self.enable_castorice_global,
            dressed_skins: self.dressed_skins.clone(),
            // game_language: self.game_language,
            // voice_language: self.voice_langauge,
        }) {
            let _ = tokio::fs::write("persistent", data).await;
            tracing::info!("persistent saved");
        }
    }
}

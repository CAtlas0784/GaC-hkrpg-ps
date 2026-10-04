use common::{resources::GAME_RES, sr_tools::FreesrData};
use proto::{get_big_data_all_recommend_sc_rsp::RecommendType, *};

use crate::net::PlayerSession;

use super::BASE_AVATAR_IDS;

pub async fn on_get_bag_cs_req(
    session: &mut PlayerSession,
    _req: &GetBagCsReq,
    res: &mut GetBagScRsp,
) {
    let Some(player) = session.json_data.get() else {
        tracing::error!("data is not set!");
        return;
    };

    res.equipment_list = player.lightcones.iter().map(|v| v.into()).collect();
    res.relic_list = player.relics.iter().map(|v| v.into()).collect();
    res.material_list = vec![
        Material {
            tid: 101, // Normal Pass
            num: 999999,
            ..Default::default()
        },
        Material {
            tid: 102, // Special Pass
            num: 999999,
            ..Default::default()
        },
    ];
}

pub async fn on_get_archive_data_cs_req(
    _session: &mut PlayerSession,
    _: &GetArchiveDataCsReq,
    res: &mut GetArchiveDataScRsp,
) {
    res.retcode = 0;
    res.archive_data = None;
}

pub async fn on_dress_relic_avatar_cs_req(
    session: &mut PlayerSession,
    req: &DressRelicAvatarCsReq,
    _: &mut DressRelicAvatarScRsp,
) {
    let Some(player) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    if let Some(pkt) = equip_relic(player, req) {
        let _ = session.send(pkt).await;
    };
}

pub async fn on_take_off_relic_cs_req(
    session: &mut PlayerSession,
    req: &TakeOffRelicCsReq,
    _: &mut TakeOffRelicScRsp,
) {
    let Some(player) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    if let Some(pkt) = unequip_relic(player, req) {
        let _ = session.send(pkt).await;
    };
}

pub async fn on_dress_avatar_cs_req(
    session: &mut PlayerSession,
    req: &DressAvatarCsReq,
    _: &mut DressAvatarScRsp,
) {
    let Some(player) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    if let Some(pkt) = set_lightcone_equipper(player, req.avatar_id, req.equipment_unique_id) {
        let _ = session.send(pkt).await;
    };
}

pub async fn on_take_off_equipment_cs_req(
    session: &mut PlayerSession,
    req: &TakeOffEquipmentCsReq,
    _: &mut TakeOffEquipmentScRsp,
) {
    let Some(player) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    if let Some(pkt) = set_lightcone_equipper(player, req.avatar_id, 0) {
        let _ = session.send(pkt).await;
    };
}

pub async fn on_get_big_data_all_recommend_cs_req(
    _: &mut PlayerSession,
    req: &GetBigDataAllRecommendCsReq,
    res: &mut GetBigDataAllRecommendScRsp,
) {
    res.big_data_recommend_type = req.big_data_recommend_type;

    match req.big_data_recommend_type() {
        BigDataRecommendType::RelicAvatar => {
            res.recommend_type = Some(RecommendType::RelicAvatar(BigDataRecommendRelicAvatar {
                recommended_avatar_info_list: GAME_RES
                    .relic_avatar_recommend
                    .iter()
                    .map(|(set_id, avatar_list)| RecomendedAvatarInfo {
                        avatar_id_list: avatar_list.clone(),
                        recommend_avatar_id: avatar_list.first().copied().unwrap_or_default(),
                        relic_set_id: *set_id,
                    })
                    .collect(),
            }))
        }
        BigDataRecommendType::AvatarRelic => {
            res.recommend_type = Some(RecommendType::AvatarRelic(BigDataRecommendAvatarRelic {
                recomended_relic_info_list: BASE_AVATAR_IDS
                    .into_iter()
                    .map(|avatar_id| BigDataAvatarRelicRecommend {
                        avatar_id,
                        ..Default::default()
                    })
                    .collect(),
            }))
        }
        _ => {}
    }
}

pub async fn on_rank_up_avatar_cs_req(
    session: &mut PlayerSession,
    req: &RankUpAvatarCsReq,
    _: &mut RankUpAvatarScRsp,
) -> Option<()> {
    let player = session.json_data.get_mut()?;
    let avatar = player.avatars.get_mut(&req.avatar_id)?;

    avatar.data.rank = req.rank;

    let avatar_id = avatar.avatar_id;

    let mut ret = PlayerSyncScNotify::default();

    build_sync(
        player,
        &mut ret,
        vec![avatar_id],
        Vec::with_capacity(0),
        Vec::with_capacity(0),
    );

    let _ = session.send(ret).await;

    Some(())
}

// TODO: move these somewhere else?

fn set_lightcone_equipper(
    player: &mut FreesrData,
    target_avatar: u32,
    target_lightcone_uid: u32,
) -> Option<PlayerSyncScNotify> {
    let mut ret = PlayerSyncScNotify::default();

    let target_avatar = player.avatars.get(&target_avatar)?;

    let cur_avatar_lc_idx = player
        .lightcones
        .iter()
        .position(|l| l.equip_avatar == target_avatar.avatar_id);

    // undress
    if target_lightcone_uid == 0
        && let Some(cur_avatar_lc_idx) = cur_avatar_lc_idx
    {
        player.lightcones[cur_avatar_lc_idx].equip_avatar = 0;

        build_sync(
            player,
            &mut ret,
            vec![target_avatar.avatar_id],
            vec![cur_avatar_lc_idx],
            Vec::with_capacity(0),
        );

        return Some(ret);
    }

    let target_lightcone_idx = player
        .lightcones
        .iter()
        .position(|l| l.get_unique_id() == target_lightcone_uid)?;

    // jika avatar sekarang sedang pakai LC, kita tukar pemiliknya dengan pemilik dari LC target
    if let Some(cur_avatar_lc_idx) = cur_avatar_lc_idx {
        player.lightcones[cur_avatar_lc_idx].equip_avatar =
            player.lightcones[target_lightcone_idx].equip_avatar;
    }

    let avatars_sync = vec![
        player.lightcones[target_lightcone_idx].equip_avatar, // old
        target_avatar.avatar_id,                              // cur
    ];

    // set kepemilikan lightcone barunya ke avatar sekarang
    player.lightcones[target_lightcone_idx].equip_avatar = target_avatar.avatar_id;

    build_sync(
        player,
        &mut ret,
        avatars_sync,
        [Some(target_lightcone_idx), cur_avatar_lc_idx]
            .into_iter()
            .flatten()
            .collect(),
        Vec::with_capacity(0),
    );

    Some(ret)
}

fn equip_relic(player: &mut FreesrData, req: &DressRelicAvatarCsReq) -> Option<PlayerSyncScNotify> {
    let mut ret = PlayerSyncScNotify::default();

    let target_avatar = player.avatars.get(&req.avatar_id)?;

    let mut avatar_ids_to_sy = vec![];
    let mut relic_index_to_sync = vec![];

    for param in &req.switch_list {
        let Some(target_relic_idx) = player
            .relics
            .iter()
            .position(|v| v.get_unique_id() == param.relic_unique_id)
        else {
            continue;
        };

        let cur_avatar_relic_idx = player.relics.iter().position(|r| {
            r.equip_avatar == target_avatar.avatar_id && r.get_slot() == param.relic_type
        });

        // jika avatar sekarang sedang pakai LC, kita tukar pemiliknya dengan pemilik dari LC target
        if let Some(cur_avatar_relic_idx) = cur_avatar_relic_idx {
            avatar_ids_to_sy.push(player.relics[cur_avatar_relic_idx].equip_avatar);
            player.relics[cur_avatar_relic_idx].equip_avatar =
                player.relics[target_relic_idx].equip_avatar;

            relic_index_to_sync.push(cur_avatar_relic_idx);
        }

        // old owner
        avatar_ids_to_sy.push(player.relics[target_relic_idx].equip_avatar);

        // set kepemilikan relic barunya ke avatar sekarang
        player.relics[target_relic_idx].equip_avatar = target_avatar.avatar_id;

        // new owner
        avatar_ids_to_sy.push(player.relics[target_relic_idx].equip_avatar);

        relic_index_to_sync.push(target_relic_idx);
    }

    build_sync(
        player,
        &mut ret,
        avatar_ids_to_sy,
        Vec::with_capacity(0),
        relic_index_to_sync,
    );

    Some(ret)
}

fn unequip_relic(player: &mut FreesrData, req: &TakeOffRelicCsReq) -> Option<PlayerSyncScNotify> {
    let mut ret = PlayerSyncScNotify::default();

    let target_avatar = player.avatars.get(&req.avatar_id)?;

    let mut relic_index_to_sync = vec![];

    for slot in &req.relic_type_list {
        let relics = player
            .relics
            .iter()
            .enumerate()
            .filter(|(_, r)| r.equip_avatar == target_avatar.avatar_id && r.get_slot() == *slot)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        for relic_idx in &relics {
            player.relics[*relic_idx].equip_avatar = 0;
        }

        relic_index_to_sync.extend(relics);
    }

    build_sync(
        player,
        &mut ret,
        vec![req.avatar_id],
        Vec::with_capacity(0),
        relic_index_to_sync,
    );

    Some(ret)
}

fn build_sync(
    player: &mut FreesrData,
    ret: &mut PlayerSyncScNotify,
    avatar_ids: Vec<u32>,
    lightcone_indexes: Vec<usize>,
    relic_indexes: Vec<usize>,
) {
    let avatar_list = avatar_ids
        .iter()
        .filter_map(|id| {
            player.get_avatar_proto(*id, player.main_character as u32, player.march_type as u32)
        })
        .collect::<Vec<_>>();

    let avatar_path_data_info_list = avatar_ids
        .into_iter()
        .filter_map(|id| player.get_avatar_path_data_proto(id))
        .collect();

    ret.avatar_sync = Some(AvatarSync {
        avatar_list,
        avatar_path_data_info_list,
    });
    ret.relic_list = relic_indexes
        .into_iter()
        .map(|id| (&player.relics[id]).into())
        .collect();
    ret.equipment_list = lightcone_indexes
        .into_iter()
        .map(|id| (&player.lightcones[id]).into())
        .collect();
}

use common::structs::AvatarJson;
use scene_entity_info::Entity;
use scene_entity_refresh_info::RefreshType;

use super::*;

pub async fn on_get_all_lineup_data_cs_req(
    session: &mut PlayerSession,
    _body: &GetAllLineupDataCsReq,
    res: &mut GetAllLineupDataScRsp,
) {
    let Some(player) = session.json_data.get() else {
        tracing::error!("data is not set!");
        return;
    };

    res.lineup_list = vec![LineupInfo {
        extra_lineup_type: ExtraLineupType::LineupNone.into(),
        name: "Squad 1".to_string(),
        mp: 5,
        max_mp: 5,
        avatar_list: AvatarJson::to_lineup_avatars(player),
        ..Default::default()
    }];
}

pub async fn on_get_cur_lineup_data_cs_req(
    session: &mut PlayerSession,
    _body: &GetCurLineupDataCsReq,
    res: &mut GetCurLineupDataScRsp,
) {
    let Some(player) = session.json_data.get() else {
        tracing::error!("data is not set!");
        return;
    };

    let lineup = LineupInfo {
        extra_lineup_type: ExtraLineupType::LineupNone.into(),
        name: "Squad 1".to_string(),
        mp: 5,
        max_mp: 5,
        avatar_list: AvatarJson::to_lineup_avatars(player),
        is_virtual: false,
        plane_id: 0,
        ..Default::default()
    };

    res.lineup = Some(lineup)
}

pub async fn on_join_lineup_cs_req(
    session: &mut PlayerSession,
    body: &JoinLineupCsReq,
    _res: &mut JoinLineupScRsp,
) {
    let Some(player) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    let lineups = &mut player.lineups;
    lineups.insert(body.slot, body.base_avatar_id);
    player.save_persistent().await;
    refresh_lineup(session).await;
}

pub async fn on_replace_lineup_cs_req(
    session: &mut PlayerSession,
    req: &ReplaceLineupCsReq,
    _res: &mut ReplaceLineupScRsp,
) {
    let Some(player) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    let lineups = &mut player.lineups;
    for (slot, avatar_id) in &mut *lineups {
        if let Some(lineup) = req.lineup_slot_list.get(*slot as usize) {
            *avatar_id = lineup.id;
        } else {
            *avatar_id = 0;
        }
    }
    player.save_persistent().await;
    refresh_lineup(session).await;
}

pub async fn on_quit_lineup_cs_req(
    _session: &mut PlayerSession,
    _: &QuitLineupCsReq,
    _res: &mut QuitLineupScRsp,
) {
}

async fn refresh_lineup(session: &mut PlayerSession) {
    let Some(player) = session.json_data.get_mut() else {
        tracing::error!("data is not set!");
        return;
    };

    let lineup = LineupInfo {
        extra_lineup_type: ExtraLineupType::LineupNone.into(),
        name: "Squad 1".to_string(),
        avatar_list: AvatarJson::to_lineup_avatars(player),
        max_mp: 5,
        mp: 5,
        ..Default::default()
    };

    let new_entities = player
        .lineups
        .iter()
        .map(|(idx, v)| SceneEntityRefreshInfo {
            refresh_type: Some(RefreshType::AddEntity(SceneEntityInfo {
                entity: Some(Entity::Actor(SceneActorInfo {
                    avatar_type: AvatarType::AvatarFormalType.into(),
                    base_avatar_id: *v,
                    map_layer: 0,
                    uid: 25,
                })),
                entity_id: idx + 1,
                group_id: 0,
                inst_id: 0,
                ..Default::default()
            })),
            ..Default::default()
        })
        .collect();

    let floor_id = player.scene.floor_id;

    session
        .send(SceneGroupRefreshScNotify {
            group_refresh_info: vec![GroupRefreshInfo {
                group_id: 0,
                state: 0,
                group_refresh_type: SceneGroupRefreshType::Loaded.into(),
                refresh_entity: new_entities,
                ..Default::default()
            }],
            floor_id,
            dimension_id: 0,
        })
        .await
        .unwrap();

    session
        .send(SyncLineupNotify {
            lineup: Some(lineup),
            reason_list: vec![],
        })
        .await
        .unwrap();
}

pub async fn on_change_lineup_leader_cs_req(
    _session: &mut PlayerSession,
    body: &ChangeLineupLeaderCsReq,
    res: &mut ChangeLineupLeaderScRsp,
) {
    res.slot = body.slot;
}

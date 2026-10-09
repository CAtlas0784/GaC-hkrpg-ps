use proto::*;

use crate::{net::PlayerSession, util};

pub const PLAYER_UID: u32 = 1337;

pub async fn on_player_get_token_cs_req(
    _session: &mut PlayerSession,
    _body: &PlayerGetTokenCsReq,
    res: &mut PlayerGetTokenScRsp,
) {
    res.retcode = 0;
    res.msg = String::from("OK");
    res.uid = PLAYER_UID;
}

pub async fn on_player_login_cs_req(
    _session: &mut PlayerSession,
    body: &PlayerLoginCsReq,
    res: &mut PlayerLoginScRsp,
) {
    res.retcode = 0;
    res.cur_timezone = 8;
    res.login_random = body.login_random;
    res.server_timestamp_ms = util::cur_timestamp_ms();
    res.stamina = 240;
    res.basic_info = Some(PlayerBasicInfo {
        nickname: String::from("RobinSR"),
        level: 70,
        world_level: 6,
        stamina: 240,
        mcoin: 9999999,
        hcoin: 9999999,
        scoin: 9999999,
        ..Default::default()
    });
}

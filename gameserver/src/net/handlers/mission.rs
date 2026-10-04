use super::*;

pub async fn on_get_mission_status_cs_req(
    _session: &mut PlayerSession,
    _body: &GetMissionStatusCsReq,
    res: &mut GetMissionStatusScRsp,
) {
    res.retcode = 0;
}

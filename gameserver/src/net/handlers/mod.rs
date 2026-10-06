mod authentication;
mod avatar;
mod battle;
pub mod challenge;
mod chat;
mod gacha;
mod inventory;
mod lineup;
mod mail;
mod mission;
mod player;
mod scene;
mod tutorial;

use anyhow::Result;
use paste::paste;
use proto::*;

use super::PlayerSession;
use crate::net::NetPacket;

pub use authentication::*;
pub use avatar::*;
pub use battle::*;
pub use challenge::*;
pub use chat::*;
pub use gacha::*;
pub use inventory::*;
pub use lineup::*;
pub use mail::*;
pub use mission::*;
pub use player::*;
pub use scene::*;
pub use tutorial::*;

macro_rules! dummy {
    ($($cmd:ident),* $(,)*) => {
        paste! {
            impl PlayerSession {
                pub const fn should_send_dummy_rsp(cmd_id: u16) -> bool {
                    match cmd_id {
                        $(
                            x if x == <proto::[<$cmd CsReq>] as proto::CmdID>::CMD_ID => true,
                        )*
                        _ => false,
                    }
                }

                pub async fn send_dummy_response(&mut self, req_id: u16) -> Result<()> {
                    let cmd_type = match req_id {
                        $(
                            x if x == <proto::[<$cmd CsReq>] as proto::CmdID>::CMD_ID => {
                                <proto::[<$cmd ScRsp>] as proto::CmdID>::CMD_ID
                            },
                        )*
                        _ => return Err(anyhow::anyhow!("Invalid request id {req_id:?}")),
                    };

                    self.send_raw(NetPacket {
                        cmd_type,
                        head: Vec::new(),
                        body: Vec::new(),
                    }).await?;

                    Ok(())
                }
            }
        }
    };
}

dummy! {
    // SceneEntityMove,
    GetLevelRewardTakenList,
    // GetRogueScoreRewardInfo, // ?3.7.51
    // GetGachaInfo,
    QueryProductInfo,
    // GetQuestData,
    GetQuestRecord,
    // GetFriendListInfo,
    // GetFriendApplyListInfo,
    GetCurAssist,
    // GetRogueHandbookData, // ?3.7.51
    GetDailyActiveInfo,
    GetFightActivityData,
    // GetMultipleDropInfo, // ?3.7.51
    // GetPlayerReturnMultiDropInfo, // ?3.7.51
    // GetShareData, // ?3.7.51
    // GetTreasureDungeonActivityData, // ?3.7.51
    // PlayerReturnInfoQuery, // ?3.7.51
    // GetBag,
    // GetPlayerBoardData,
    // GetActivityScheduleConfig,
    // GetMissionData,
    // GetChallenge,
    // GetCurChallenge,
    // GetRogueInfo, // ?3.7.51
    GetExpeditionData,
    // GetRogueDialogueEventData,
    // GetJukeboxData,
    // DailyFirstMeetPam, // ?3.7.51
    // GetMuseumInfo, // ?3.7.51
    GetLoginActivity,
    GetRaidInfo,
    GetTrialActivityData,
    // GetBoxingClubInfo, // ?3.7.51
    GetNpcStatus,
    // TextJoinQuery, // ?3.7.51
    // GetSpringRecoverData, // Removed 2.7.51
    // GetChatFriendHistory,
    GetSecretKeyInfo,
    GetVideoVersionKey,
    // GetPhoneData,
    // PlayerLoginFinish,
    FinishTalkMission,
    GetRechargeGiftInfo,
    // RelicRecommend
    GetPreAvatarGrowthInfo,
    GetPreAvatarActivityList,
    // GetUnreleasedBlockInfo,
    GetFriendAssistList,
    GetAssistList,
    B51RacingGetData
}

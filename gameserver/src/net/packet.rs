use anyhow::Result;
use paste::paste;
use tracing::Instrument;

use proto::*;

use super::PlayerSession;
use super::handlers::*;

const HEAD_MAGIC: u32 = 0x9D74C714;
const TAIL_MAGIC: u32 = 0xD7A152C8;

#[derive(Debug)]
pub struct NetOperation {
    pub head: u32,
    pub param1: u32,
    pub param2: u32,
    pub data: u32,
    pub tail: u32,
}

#[derive(Debug)]
pub struct NetPacket {
    pub cmd_type: u16,
    pub head: Vec<u8>,
    pub body: Vec<u8>,
}

impl From<NetPacket> for Vec<u8> {
    fn from(value: NetPacket) -> Self {
        let mut out = Self::new();

        out.extend(HEAD_MAGIC.to_be_bytes());
        out.extend(value.cmd_type.to_be_bytes());
        out.extend((value.head.len() as u16).to_be_bytes());
        out.extend((value.body.len() as u32).to_be_bytes());
        out.extend(value.head);
        out.extend(value.body);
        out.extend(TAIL_MAGIC.to_be_bytes());
        out
    }
}

impl From<&[u8]> for NetPacket {
    fn from(value: &[u8]) -> Self {
        assert_eq!(
            u32::from_be_bytes(value[0..4].try_into().unwrap()),
            HEAD_MAGIC
        );

        let cmd_type = u16::from_be_bytes(value[4..6].try_into().unwrap());

        let head_length = usize::from(u16::from_be_bytes(value[6..8].try_into().unwrap()));

        let body_length = u32::from_be_bytes(value[8..12].try_into().unwrap()) as usize;

        let head_start = 12;
        let head_end = head_start + head_length;
        let head = value[head_start..head_end].to_vec();

        let body_start = head_end;
        let body_end = body_start + body_length;
        let body = value[body_start..body_end].to_vec();

        assert_eq!(
            u32::from_be_bytes(value[body_end..body_end + 4].try_into().unwrap()),
            TAIL_MAGIC
        );

        Self {
            cmd_type,
            head,
            body,
        }
    }
}

impl From<&[u8]> for NetOperation {
    fn from(value: &[u8]) -> Self {
        Self {
            head: u32::from_be_bytes(value[..4].try_into().unwrap()),
            param1: u32::from_be_bytes(value[4..8].try_into().unwrap()),
            param2: u32::from_be_bytes(value[8..12].try_into().unwrap()),
            data: u32::from_be_bytes(value[12..16].try_into().unwrap()),
            tail: u32::from_be_bytes(value[16..20].try_into().unwrap()),
        }
    }
}

impl From<NetOperation> for Vec<u8> {
    fn from(value: NetOperation) -> Self {
        let mut buf = Self::with_capacity(20);
        buf.extend(value.head.to_be_bytes());
        buf.extend(value.param1.to_be_bytes());
        buf.extend(value.param2.to_be_bytes());
        buf.extend(value.data.to_be_bytes());
        buf.extend(value.tail.to_be_bytes());

        buf
    }
}

macro_rules! trait_handler {
    ($($name:tt;)*) => {
        pub trait CommandHandler {
            $(
                paste! {
                    async fn [<on_$name:snake _cs_req>](session: &mut PlayerSession, request: &[<$name CsReq>]) -> Result<()> {
                        let mut response = proto::[<$name ScRsp>]::default();
                        let _ = [<on_$name:snake _cs_req>](session, request, &mut response).await;
                        session.send(response).await?;

                        Ok(())
                    }
                }
            )*

            async fn on_message(session: &mut PlayerSession, cmd_id: u16, payload: Vec<u8>) -> Result<()> {
                use ::prost::Message;
                tracing::info!("[PACKET] Received cmd_id: {cmd_id}");
                if PlayerSession::should_send_dummy_rsp(cmd_id) {
                    session.send_dummy_response(cmd_id).await?;
                    return Ok(());
                }


                match cmd_id {
                    $(
                        cmd_id if cmd_id == paste! { <proto::[<$name CsReq>] as proto::CmdID>::CMD_ID } => {
                            let body = paste! { proto::[<$name CsReq>]::decode(&mut &payload[..])? };
                            paste! {
                                Self::[<on_$name:snake _cs_req>](session, &body)
                                    .instrument(tracing::info_span!(stringify!([<on_$name:snake>]), cmd_id = cmd_id))
                                    .await
                            }
                        }
                    )*
                    1775 | 1711 => {
                        let mut buf = &payload[..];
                        let mut group_id = 100u32;
                        while !buf.is_empty() {
                            if let Ok(tag) = prost::encoding::decode_varint(&mut buf) {
                                let field_number = tag >> 3;
                                let wire_type = tag & 0x7;
                                if field_number == 12 && wire_type == 0 {
                                    if let Ok(val) = prost::encoding::decode_varint(&mut buf) {
                                        group_id = val as u32;
                                    }
                                    break;
                                } else if wire_type == 0 {
                                    let _ = prost::encoding::decode_varint(&mut buf);
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }

                        let mut body = Vec::new();
                        // Tag 9: group_id
                        body.push(0x48);
                        prost::encoding::encode_varint(group_id as u64, &mut body);
                        // Tag 15: retcode 0
                        body.extend_from_slice(&[0x78, 0x00]);

                        if (2000..3000).contains(&group_id) {
                            // Pure Fiction: Tag 8 = challenge_story (ChallengeStoryStatistics)
                            // EIKPHEMHIOH: score_id=tag 10 (80000), level=tag 11 (4)
                            let mut eik = Vec::new();
                            eik.push(0x50); // tag 10
                            prost::encoding::encode_varint(80000, &mut eik);
                            eik.push(0x58); // tag 11
                            prost::encoding::encode_varint(4, &mut eik);

                            // ChallengeStoryStatistics: PPBHLLOJNEK=tag 6, record_id=tag 7 (1)
                            let mut css = Vec::new();
                            css.push(0x32); // tag 6
                            prost::encoding::encode_varint(eik.len() as u64, &mut css);
                            css.extend_from_slice(&eik);
                            css.push(0x38); // tag 7
                            prost::encoding::encode_varint(1, &mut css);

                            body.push(0x42); // tag 8 (len-delimited)
                            prost::encoding::encode_varint(css.len() as u64, &mut body);
                            body.extend_from_slice(&css);
                        } else if (1000..2000).contains(&group_id) {
                            // Memory of Chaos: Tag 2 = challenge_default (ChallengeStatistics)
                            // ADKJKMKBFDC: round_count=tag 2 (20), level=tag 3 (12)
                            let mut adk = Vec::new();
                            adk.push(0x10); // tag 2
                            prost::encoding::encode_varint(20, &mut adk);
                            adk.push(0x18); // tag 3
                            prost::encoding::encode_varint(12, &mut adk);

                            // ChallengeStatistics: PPBHLLOJNEK=tag 6, record_id=tag 10 (1)
                            let mut cs = Vec::new();
                            cs.push(0x32); // tag 6
                            prost::encoding::encode_varint(adk.len() as u64, &mut cs);
                            cs.extend_from_slice(&adk);
                            cs.push(0x50); // tag 10
                            prost::encoding::encode_varint(1, &mut cs);

                            body.push(0x12); // tag 2 (len-delimited)
                            prost::encoding::encode_varint(cs.len() as u64, &mut body);
                            body.extend_from_slice(&cs);
                        } else if group_id >= 3000 {
                            // Apocalyptic Shadow: Tag 11 = challenge_boss (ChallengeBossStatistics)
                            // AANLJBLOOFO: score_id=tag 10 (8000), level=tag 3 (4)
                            let mut aan = Vec::new();
                            aan.push(0x50); // tag 10
                            prost::encoding::encode_varint(8000, &mut aan);
                            aan.push(0x18); // tag 3
                            prost::encoding::encode_varint(4, &mut aan);

                            // ChallengeBossStatistics: PPBHLLOJNEK=tag 1, record_id=tag 14 (1)
                            let mut cbs = Vec::new();
                            cbs.push(0x0A); // tag 1
                            prost::encoding::encode_varint(aan.len() as u64, &mut cbs);
                            cbs.extend_from_slice(&aan);
                            cbs.push(0x70); // tag 14
                            prost::encoding::encode_varint(1, &mut cbs);

                            body.push(0x5A); // tag 11 (len-delimited)
                            prost::encoding::encode_varint(cbs.len() as u64, &mut body);
                            body.extend_from_slice(&cbs);
                        }

                        // 4.6.51: 1708, legacy: 1736
                        session.send_raw(NetPacket {
                            cmd_type: 1708,
                            head: Vec::new(),
                            body: body.clone(),
                        }).await?;
                        let _ = session.send_raw(NetPacket {
                            cmd_type: 1736,
                            head: Vec::new(),
                            body,
                        }).await;
                        Ok(())
                    }
                    8995 | 8979 | 8981 => {
                        let body = challenge::build_get_challenge_tierce_data_sc_rsp();
                        // 4.6.51: 8996, legacy: 8994
                        session.send_raw(NetPacket {
                            cmd_type: 8996,
                            head: Vec::new(),
                            body: body.clone(),
                        }).await?;
                        let _ = session.send_raw(NetPacket {
                            cmd_type: 8994,
                            head: Vec::new(),
                            body,
                        }).await;
                        Ok(())
                    }
                    8909 => {
                        session.send_raw(NetPacket {
                            cmd_type: 8923,
                            head: Vec::new(),
                            body: vec![0x60, 0x00], // tag 12: retcode = 0
                        }).await?;
                        Ok(())
                    }
                    8924 | 8968 => {
                        session.send_raw(NetPacket {
                            cmd_type: 8920,
                            head: Vec::new(),
                            body: vec![0x10, 0x00], // tag 2: retcode = 0
                        }).await?;
                        Ok(())
                    }
                    8935 => {
                        session.send_raw(NetPacket { cmd_type: 8915, head: Vec::new(), body: vec![0x08, 0x00] }).await?;
                        Ok(())
                    }
                    8917 => {
                        session.send_raw(NetPacket { cmd_type: 8929, head: Vec::new(), body: vec![0x08, 0x00] }).await?;
                        Ok(())
                    }
                    8933 => {
                        session.send_raw(NetPacket { cmd_type: 8906, head: Vec::new(), body: vec![0x08, 0x00] }).await?;
                        Ok(())
                    }
                    8919 => {
                        session.send_raw(NetPacket { cmd_type: 8948, head: Vec::new(), body: vec![0x08, 0x00] }).await?;
                        Ok(())
                    }
                    8910 => {
                        session.send_raw(NetPacket { cmd_type: 8921, head: Vec::new(), body: vec![0x08, 0x00] }).await?;
                        Ok(())
                    }
                    8975 => {
                        session.send_raw(NetPacket { cmd_type: 8992, head: Vec::new(), body: vec![0x40, 0x00] }).await?;
                        Ok(())
                    }
                    8990 => {
                        session.send_raw(NetPacket { cmd_type: 8977, head: Vec::new(), body: vec![0x40, 0x00] }).await?;
                        Ok(())
                    }
                    8977 => {
                        session.send_raw(NetPacket { cmd_type: 8999, head: Vec::new(), body: vec![0x50, 0x00] }).await?;
                        Ok(())
                    }
                    2966 => {
                        session.send_raw(NetPacket { cmd_type: 2955, head: Vec::new(), body: vec![0x30, 0x00] }).await?;
                        Ok(())
                    }
                    2916 => {
                        session.send_raw(NetPacket { cmd_type: 2906, head: Vec::new(), body: vec![0x18, 0x00] }).await?;
                        Ok(())
                    }
                    2919 => {
                        session.send_raw(NetPacket { cmd_type: 2928, head: Vec::new(), body: vec![0x48, 0x00] }).await?;
                        Ok(())
                    }
                    740 => {
                        session.send_raw(NetPacket { cmd_type: 733, head: Vec::new(), body: vec![0x58, 0x00] }).await?;
                        Ok(())
                    }
                    1748 => {
                        session.send_raw(NetPacket { cmd_type: 1745, head: Vec::new(), body: vec![0x08, 0x00] }).await?;
                        Ok(())
                    }
                    1765 => {
                        session.send_raw(NetPacket { cmd_type: 1730, head: Vec::new(), body: vec![0x08, 0x00] }).await?;
                        Ok(())
                    }
                    1742 => {
                        session.send_raw(NetPacket { cmd_type: 1772, head: Vec::new(), body: vec![0x08, 0x00] }).await?;
                        Ok(())
                    }
                    1738 => {
                        session.send_raw(NetPacket { cmd_type: 1748, head: Vec::new(), body: Vec::new() }).await?;
                        Ok(())
                    }
                    1731 | 1793 => {
                        challenge::handle_start_challenge(session, &payload).await
                    }
                    8986 | 8983 | 8988 => {
                        challenge::handle_start_challenge_tierce(session, &payload).await
                    }
                    8978 => {
                        challenge::handle_set_challenge_tierce_lineup(session, &payload).await
                    }
                    1705 | 1760 | 1788 => {
                        challenge::handle_leave_challenge(session).await
                    }
                    8989 | 8998 => {
                        challenge::handle_leave_challenge_tierce(session).await
                    }
                    8993 | 8991 => {
                        // 4.6.51: 8981, 4.5.52: 8982 (tag 14 retcode = 0)
                        session.send_raw(NetPacket {
                            cmd_type: 8981,
                            head: Vec::new(),
                            body: vec![0x70, 0x00],
                        }).await?;
                        let _ = session.send_raw(NetPacket {
                            cmd_type: 8982,
                            head: Vec::new(),
                            body: vec![0x70, 0x00],
                        }).await;
                        Ok(())
                    }
                    8987 => {
                        // 4.5.52 ConfirmChallengeTierceStageSettleCsReq -> ConfirmChallengeTierceStageSettleScRsp (CmdID 8986: tag 7 retcode = 0)
                        session.send_raw(NetPacket {
                            cmd_type: 8986,
                            head: Vec::new(),
                            body: vec![0x38, 0x00],
                        }).await?;
                        Ok(())
                    }
                    1739 => {
                        challenge::handle_take_challenge_reward(session, &payload).await
                    }
                    188 => {
                        Self::on_get_cur_battle_info_cs_req(session, &proto::GetCurBattleInfoCsReq::default()).await?;
                        Ok(())
                    }
                    4130 => {
                        session.send_raw(NetPacket {
                            cmd_type: 4124,
                            head: Vec::new(),
                            body: vec![0x70, 0x00], // tag 14: retcode = 0 (GetUnreleasedBlockInfoScRsp)
                        }).await?;
                        Ok(())
                    }
                    8110 => {
                        session.send_raw(NetPacket {
                            cmd_type: 8116,
                            head: Vec::new(),
                            body: vec![0x38, 0x00], // tag 7: retcode = 0 (GetSwitchHandDataScRsp)
                        }).await?;
                        Ok(())
                    }
                    _ => {
                        if cmd_id != 7159 {
                            tracing::warn!("Unknown command ID: {cmd_id}");
                        }
                        Ok(())
                    },
                }
            }
        }
    };
}

trait_handler! {
    PlayerGetToken;
    PlayerLogin;
    GetMissionStatus;
    GetBasicInfo;
    GetAvatarData;
    GetAllLineupData;
    GetCurLineupData;
    GetCurSceneInfo;
    PlayerHeartBeat;
    SetAvatarEnhancedId;
    TakePromotionReward;
    SyncClientResVersion;

    // Entity move (dummy!)
    SceneEntityMove;

    // Inventory (dummy!)
    GetBag;
    GetArchiveData;
    DressAvatar;
    TakeOffEquipment;
    DressRelicAvatar;
    TakeOffRelic;
    RankUpAvatar;

    // Chat (dummy!)
    SendMsg;
    GetPrivateChatHistory;
    GetFriendListInfo;
    GetFriendLoginInfo;

    // In-game lineup
    JoinLineup;
    ChangeLineupLeader;
    ReplaceLineup;
    QuitLineup;

    // Battle
    StartCocoonStage;
    PveBattleResult;
    SceneCastSkill;
    QuickStartCocoonStage;
    SceneEnterStage;
    GetCurBattleInfo;

    // Teleport / Scene
    GetEnteredScene;
    GetSceneMapInfo;
    EnterScene;
    InteractProp;

    // Optional
    GetMail;
    GetGachaInfo;
    DoGacha;
    PlayerLoginFinish;
    GetBigDataAllRecommend;
    // SetClientPaused;

    // Challenge & Activity
    GetChallenge;
    GetCurChallenge;
    GetActivityScheduleConfig;
}

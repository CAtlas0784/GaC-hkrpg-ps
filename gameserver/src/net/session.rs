use std::{
    io::Error,
    net::SocketAddr,
    pin::Pin,
    sync::{Arc, OnceLock},
    task::{Context, Poll},
};

use anyhow::Result;
use common::sr_tools::FreesrData;
use mhy_kcp::Kcp;
use prost::Message;
use proto::{AvatarSync, CmdID, CmdPlayerType, PlayerSyncScNotify};
use tokio::{
    io::AsyncWrite,
    net::UdpSocket,
    sync::{Mutex, watch},
};

use crate::{net::handlers::BASE_AVATAR_IDS, util};

use super::{NetPacket, packet::CommandHandler};

struct RemoteEndPoint {
    socket: Arc<UdpSocket>,
    addr: SocketAddr,
}

pub struct PlayerSession {
    pub token: u32,
    kcp: Arc<Mutex<Kcp<RemoteEndPoint>>>,
    start_time: u64,
    pub shutdown_tx: watch::Sender<()>,
    pub shutdown_rx: watch::Receiver<()>,
    pub json_data: OnceLock<FreesrData>,
    pub next_scene_save: u64,
    /// entity_id -> (prop_id, inst_id, prop_state) ของ props ที่โหลดในฉากปัจจุบัน
    pub prop_entities: std::collections::HashMap<u32, (u32, u32, u32)>,
}

impl PlayerSession {
    pub fn new(socket: Arc<UdpSocket>, addr: SocketAddr, conv: u32, token: u32) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(());
        Self {
            token,
            kcp: Arc::new(Mutex::new(Kcp::new(
                conv,
                token,
                false,
                RemoteEndPoint { socket, addr },
            ))),
            start_time: util::cur_timestamp_secs(),
            json_data: OnceLock::new(),
            shutdown_rx,
            shutdown_tx,
            next_scene_save: 0,
            prop_entities: std::collections::HashMap::new(),
        }
    }

    pub async fn consume(&mut self, buffer: &[u8]) -> Result<()> {
        let mut kcp = self.kcp.lock().await;
        kcp.input(buffer)?;
        kcp.async_update(self.session_time() as u32).await?;
        kcp.async_flush().await?;

        let mut packets = Vec::new();
        let mut buf = [0; 24756];
        while let Ok(length) = kcp.recv(&mut buf) {
            packets.push(NetPacket::from(&buf[..length]));
        }

        drop(kcp);

        for packet in packets {
            if packet.cmd_type == CmdPlayerType::CmdPlayerLogoutCsReq as u16 {
                tracing::info!("Player logged out");
                let _ = self.shutdown_tx.send(());
                return Ok(());
            };
            Self::on_message(self, packet.cmd_type, packet.body).await?;
        }

        self.kcp
            .lock()
            .await
            .async_update(self.session_time() as u32)
            .await?;
        Ok(())
    }

    pub async fn send(&self, body: impl Message + CmdID) -> Result<()> {
        let mut buf = Vec::new();
        body.encode(&mut buf)?;
        crate::util::packet_log(&format!("[PACKET SEND] cmd_id: {} (len: {})", body.get_cmd_id(), buf.len()));
        tracing::info!("sent packet with CmdID: {}", body.get_cmd_id());

        let payload: Vec<u8> = NetPacket {
            cmd_type: body.get_cmd_id(),
            head: Vec::new(),
            body: buf,
        }
        .into();

        let mut kcp = self.kcp.lock().await;
        kcp.send(&payload)?;
        kcp.async_flush().await?;
        kcp.async_update(self.session_time() as u32).await?;

        Ok(())
    }

    pub async fn send_raw(&self, payload: NetPacket) -> Result<()> {
        crate::util::packet_log(&format!("[PACKET SEND RAW] cmd_id: {} (len: {})", payload.cmd_type, payload.body.len()));
        tracing::info!("sent raw packet with CmdID: {}", payload.cmd_type);
        let mut kcp = self.kcp.lock().await;
        let payload: Vec<u8> = payload.into();
        kcp.send(&payload)?;
        kcp.async_flush().await?;
        kcp.async_update(self.session_time() as u32).await?;

        Ok(())
    }

    pub async fn sync_player(&self) -> Result<()> {
        let Some(json) = self.json_data.get() else {
            tracing::error!("data is not init!");
            return Ok(());
        };

        // clear relics & lightcones
        self.send(PlayerSyncScNotify {
            del_relic_list: (1..=3000).collect(),
            del_equipment_list: (3001..=3500).collect(),
            ..Default::default()
        })
        .await?;

        // clear all avatars equip item
        self.send(PlayerSyncScNotify {
            avatar_sync: Some(AvatarSync {
                avatar_list: BASE_AVATAR_IDS
                    .into_iter()
                    .filter_map(|id| {
                        json.avatars.get(&id).map(|v| {
                            v.to_avatar_proto(
                                Option::None,
                                json.main_character as u32,
                                json.march_type as u32,
                            )
                        })
                    })
                    .collect(),
                avatar_path_data_info_list: json
                    .avatars
                    .values()
                    .map(|avatar| {
                        avatar.to_avatar_path_data_proto(
                            Option::None,
                            Vec::with_capacity(0),
                            json.dressed_skins
                                .get(&avatar.avatar_id)
                                .copied()
                                .unwrap_or(0),
                        )
                    })
                    .collect::<Vec<_>>(),
            }),

            ..Default::default()
        })
        .await?;

        // Sync new relics & lightcones
        self.send(PlayerSyncScNotify {
            relic_list: json.relics.iter().map(|v| v.into()).collect(),
            equipment_list: json.lightcones.iter().map(|v| v.into()).collect(),
            ..Default::default()
        })
        .await?;

        // Sync new avatars equip item
        self.send(PlayerSyncScNotify {
            avatar_sync: Some(AvatarSync {
                avatar_list: BASE_AVATAR_IDS
                    .into_iter()
                    .filter_map(|avatar_id| {
                        json.avatars.get(&avatar_id).map(|v| {
                            v.to_avatar_proto(
                                json.lightcones.iter().find(|v| v.equip_avatar == avatar_id),
                                json.main_character as u32,
                                json.march_type as u32,
                            )
                        })
                    })
                    .collect(),
                avatar_path_data_info_list: json
                    .avatars
                    .values()
                    .map(|avatar| {
                        avatar.to_avatar_path_data_proto(
                            json.lightcones
                                .iter()
                                .find(|v| v.equip_avatar == avatar.avatar_id),
                            json.relics
                                .iter()
                                .filter(|r| r.equip_avatar == avatar.avatar_id)
                                .collect(),
                            json.dressed_skins
                                .get(&avatar.avatar_id)
                                .copied()
                                .unwrap_or(0),
                        )
                    })
                    .collect::<Vec<_>>(),
            }),
            ..Default::default()
        })
        .await
    }

    fn session_time(&self) -> u64 {
        util::cur_timestamp_secs() - self.start_time
    }
}

// Auto implemented
impl CommandHandler for PlayerSession {}

impl AsyncWrite for RemoteEndPoint {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        self.socket.poll_send_to(cx, buf, self.addr)
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}

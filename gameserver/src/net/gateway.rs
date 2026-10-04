use std::{
    collections::HashMap,
    net::SocketAddr,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, SystemTime},
};

use anyhow::Result;

use common::sr_tools::FreesrData;
use rand::Rng;

use crate::net::PlayerSession;

use tokio::{
    net::UdpSocket,
    sync::{Mutex, RwLock, mpsc},
};

use crate::net::packet::NetOperation;

const MAX_PACKET_SIZE: usize = 1400;

pub struct Gateway {
    socket: Arc<UdpSocket>,
    id_counter: AtomicU32,
    sessions: Mutex<HashMap<u32, Arc<RwLock<PlayerSession>>>>,
}

impl Gateway {
    pub async fn new(host: &str, port: u16) -> Result<Self> {
        let socket = Arc::new(UdpSocket::bind(format!("{host}:{port}")).await?);

        Ok(Self {
            socket,
            id_counter: AtomicU32::new(0),
            sessions: Mutex::new(HashMap::new()),
        })
    }

    pub async fn listen(&mut self) -> Result<()> {
        tracing::info!(
            "KCP Gateway is listening at {}",
            self.socket.local_addr().unwrap()
        );

        let mut buf = [0; MAX_PACKET_SIZE];
        loop {
            let Ok((len, addr)) = self.socket.recv_from(&mut buf).await else {
                continue;
            };

            match len {
                20 => self.process_net_operation(buf[..len].into(), addr).await?,
                28.. => self.process_kcp_payload(buf[..len].into(), addr).await,
                _ => {
                    tracing::warn!("unk data len {len}")
                }
            }
        }
    }

    async fn process_net_operation(&mut self, op: NetOperation, addr: SocketAddr) -> Result<()> {
        match (op.head, op.tail) {
            (0xFF, 0xFFFFFFFF) => self.establish_kcp_session(op.data, addr).await?,
            (0x194, 0x19419494) => self.drop_kcp_session(op.param1, op.param2, addr).await,
            _ => tracing::warn!("Unknown magic pair received {:X}-{:X}", op.head, op.tail),
        }

        Ok(())
    }

    async fn establish_kcp_session(&mut self, data: u32, addr: SocketAddr) -> Result<()> {
        let (conv_id, session_token) = self.next_conv_pair();
        tracing::info!("New connection from addr: {addr} with conv_id: {conv_id}");
        crate::util::packet_log(&format!("[KCP] New connection from {addr} (conv_id: {conv_id}, token: {session_token})"));

        let session = Arc::new(RwLock::new(PlayerSession::new(
            self.socket.clone(),
            addr,
            conv_id,
            session_token,
        )));

        // Init the json to session
        match FreesrData::load().await {
            Ok(data) => {
                let _ = session.write().await.json_data.set(data);
                crate::util::packet_log("[KCP] FreesrData loaded successfully");
            }
            Err(e) => {
                crate::util::packet_log(&format!("[ERROR] Failed to load initial freesr-data.json: {e}"));
                tracing::error!("Failed to load initial freesr-data.json: {e}");
            }
        }

        let session_ref = session.clone();

        // FS watcher
        tokio::spawn(async move {
            let (tx, mut rx) = mpsc::channel(100);
            let mut debouncer =
                match notify_debouncer_mini::new_debouncer(Duration::from_millis(1000), move |ev| {
                    let _ = tx.blocking_send(ev);
                }) {
                    Ok(d) => d,
                    Err(e) => {
                        tracing::error!("debouncer init err: {e}");
                        return;
                    }
                };

            let path = Path::new("freesr-data.json");

            if let Err(e) = debouncer
                .watcher()
                .watch(path, notify::RecursiveMode::NonRecursive)
            {
                tracing::error!("failed to watch {path:?}: {e}");
                return;
            }

            tracing::info!("watching freesr-data.json changes");

            let mut shutdown_rx = session.read().await.shutdown_rx.clone();

            let mut last_modified = std::fs::metadata(path)
                .and_then(|meta| meta.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            loop {
                tokio::select! {
                    res = rx.recv() => {
                        let Some(res) = res else {
                            break;
                        };
                        match res {
                            Ok(events) => {
                                if events
                                    .iter()
                                    .any(|p| p.path.file_name() == path.file_name())
                                    && let Ok(metadata) = std::fs::metadata(path)
                                        && let Ok(modified) = metadata.modified()
                                            && modified > last_modified {
                                                last_modified = modified;

                                                let mut session = session.write().await;
                                                if let Some(json) = session.json_data.get_mut() {
                                                    if let Err(e) = json.update().await {
                                                        tracing::error!("Failed to update json: {e}");
                                                    } else {
                                                        let _ = session.sync_player().await;
                                                        tracing::info!("json updated");
                                                    }
                                                }
                                            }
                            }
                            Err(e) => tracing::error!("json watcher error: {e:?}"),
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        break;
                    }
                }
            }

            tracing::info!("unwatch freesr-data.json");
        });

        let mut sessions = self.sessions.lock().await;
        for session in sessions.values_mut() {
            let _ = session.write().await.shutdown_tx.send(());
        }
        sessions.clear();
        sessions.insert(conv_id, session_ref);

        self.socket
            .send_to(
                &Vec::from(NetOperation {
                    head: 0x145,
                    param1: conv_id,
                    param2: session_token,
                    data,
                    tail: 0x14514545,
                }),
                addr,
            )
            .await?;

        Ok(())
    }

    async fn drop_kcp_session(&mut self, conv_id: u32, token: u32, addr: SocketAddr) {
        tracing::info!("drop_kcp_session {conv_id} {token}");
        let mut sessions = self.sessions.lock().await;
        let Some(session) = sessions.get(&conv_id) else {
            tracing::warn!("drop_kcp_session failed, no session with conv_id {conv_id} was found");
            return;
        };
        let session = session.write().await;
        if session.token == token {
            let _ = session.shutdown_tx.send(());
            drop(session);
            sessions.remove(&conv_id);
            tracing::info!("Client from {addr} disconnected");
        }
    }

    async fn process_kcp_payload(&mut self, data: Box<[u8]>, addr: SocketAddr) {
        let conv_id = mhy_kcp::get_conv(&data);
        let mut sessions = self.sessions.lock().await;

        let Some(session) = sessions.get_mut(&conv_id).map(|s| s.clone()) else {
            tracing::warn!("Session with conv_id {conv_id} not found!");
            return;
        };

        tokio::spawn(async move {
            if let Err(err) = Box::pin(session.write().await.consume(&data)).await {
                crate::util::packet_log(&format!("[ERROR] Session consume error ({addr}): {err}"));
                tracing::error!("An error occurred while processing session ({addr}): {err}");
            }
        });
    }

    fn next_conv_pair(&mut self) -> (u32, u32) {
        (
            self.id_counter.fetch_add(1, Ordering::SeqCst) + 1,
            rand::rng().next_u32(),
        )
    }
}

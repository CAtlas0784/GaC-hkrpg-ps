use std::{
    io::{self, Error, Write},
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use anyhow::{Result, bail};
use mhy_kcp::Kcp;
use prost::Message;
use proto::*;
use tokio::{
    io::AsyncWrite,
    net::UdpSocket,
    sync::{mpsc, Mutex},
    time::{interval, sleep},
};

const HEAD_MAGIC: u32 = 0x9D74C714;
const TAIL_MAGIC: u32 = 0xD7A152C8;

#[derive(Debug, Clone)]
pub struct NetPacket {
    pub cmd_type: u16,
    pub head: Vec<u8>,
    pub body: Vec<u8>,
}

impl NetPacket {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend(HEAD_MAGIC.to_be_bytes());
        out.extend(self.cmd_type.to_be_bytes());
        out.extend((self.head.len() as u16).to_be_bytes());
        out.extend((self.body.len() as u32).to_be_bytes());
        out.extend(&self.head);
        out.extend(&self.body);
        out.extend(TAIL_MAGIC.to_be_bytes());
        out
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        if buf.len() < 16 {
            return None;
        }
        let head_magic = u32::from_be_bytes(buf[0..4].try_into().ok()?);
        if head_magic != HEAD_MAGIC {
            return None;
        }
        let cmd_type = u16::from_be_bytes(buf[4..6].try_into().ok()?);
        let head_length = u16::from_be_bytes(buf[6..8].try_into().ok()?) as usize;
        let body_length = u32::from_be_bytes(buf[8..12].try_into().ok()?) as usize;

        let head_start = 12;
        let head_end = head_start + head_length;
        let body_start = head_end;
        let body_end = body_start + body_length;

        if buf.len() < body_end + 4 {
            return None;
        }
        let tail_magic = u32::from_be_bytes(buf[body_end..body_end + 4].try_into().ok()?);
        if tail_magic != TAIL_MAGIC {
            return None;
        }

        Some(Self {
            cmd_type,
            head: buf[head_start..head_end].to_vec(),
            body: buf[body_start..body_end].to_vec(),
        })
    }
}

struct RemoteEndPoint {
    socket: Arc<UdpSocket>,
    addr: SocketAddr,
}

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

pub struct ProbeClient {
    kcp: Arc<Mutex<Kcp<RemoteEndPoint>>>,
    incoming_rx: mpsc::Receiver<NetPacket>,
    start_time: std::time::Instant,
}

impl ProbeClient {
    pub async fn connect(target: &str) -> Result<Self> {
        let server_addr: SocketAddr = target.parse()?;
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);

        println!("[*] Sending KCP handshake to {server_addr}...");
        // 20-byte handshake request
        let mut req = Vec::with_capacity(20);
        req.extend(0x000000FFu32.to_be_bytes());
        req.extend(0u32.to_be_bytes());
        req.extend(0u32.to_be_bytes());
        req.extend(0u32.to_be_bytes());
        req.extend(0xFFFFFFFFu32.to_be_bytes());

        socket.send_to(&req, server_addr).await?;

        // Wait for handshake response (timeout 3s)
        let mut buf = [0u8; 1500];
        let (len, _) = tokio::time::timeout(Duration::from_secs(3), socket.recv_from(&mut buf))
            .await??;

        if len != 20 {
            bail!("Unexpected handshake response length: {len}");
        }

        let head = u32::from_be_bytes(buf[0..4].try_into()?);
        let p1_be = u32::from_be_bytes(buf[4..8].try_into()?);
        let p2_be = u32::from_be_bytes(buf[8..12].try_into()?);
        let p2_le = u32::from_le_bytes(buf[8..12].try_into()?);
        let tail = u32::from_be_bytes(buf[16..20].try_into()?);

        // March7th sends enet (0) in param1 and conv_id (1) in param2 (LE/BE)
        let (conv, token) = if p1_be == 0 && (p2_be != 0 || p2_le != 0) {
            let c = if p2_le < 1000 { p2_le } else { p2_be };
            (c, c)
        } else {
            (p1_be, p2_be)
        };

        println!("[+] Handshake OK! head=0x{head:x}, conv={conv}, token={token}, tail=0x{tail:x}");

        let kcp = Arc::new(Mutex::new(Kcp::new(
            conv,
            token,
            false,
            RemoteEndPoint {
                socket: socket.clone(),
                addr: server_addr,
            },
        )));

        // Set nodelay mode and initial update
        {
            let mut k = kcp.lock().await;
            k.set_nodelay(true, 10, 2, true);
            k.async_update(0).await?;
        }

        let (tx, rx) = mpsc::channel(100);
        let kcp_clone = kcp.clone();
        let socket_clone = socket.clone();

        // Background driver task
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(10));
            let mut recv_buf = [0u8; 65535];
            let mut kcp_buf = [0u8; 65535];
            let start = std::time::Instant::now();

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let ms = start.elapsed().as_millis() as u32;
                        let mut k = kcp_clone.lock().await;
                        let _ = k.async_update(ms).await;
                        let _ = k.async_flush().await;
                    }
                    res = socket_clone.recv_from(&mut recv_buf) => {
                        if let Ok((len, _)) = res {
                            if len >= 28 {
                                let ms = start.elapsed().as_millis() as u32;
                                let mut k = kcp_clone.lock().await;
                                if k.input(&recv_buf[..len]).is_ok() {
                                    let _ = k.async_update(ms).await;
                                    let _ = k.async_flush().await;
                                    while let Ok(pack_len) = k.recv(&mut kcp_buf) {
                                        if let Some(packet) = NetPacket::decode(&kcp_buf[..pack_len]) {
                                            let _ = tx.send(packet).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            kcp,
            incoming_rx: rx,
            start_time: std::time::Instant::now(),
        })
    }

    pub async fn send_packet(&self, cmd_type: u16, body: &[u8]) -> Result<()> {
        let packet = NetPacket {
            cmd_type,
            head: Vec::new(),
            body: body.to_vec(),
        };
        let encoded = packet.encode();
        let ms = self.start_time.elapsed().as_millis() as u32;
        let mut k = self.kcp.lock().await;
        k.async_update(ms).await?;
        k.send(&encoded)?;
        k.async_flush().await?;
        println!(">>> [SENT] CmdID: {cmd_type} | Body length: {} bytes", body.len());
        Ok(())
    }

    pub async fn wait_for_packet(&mut self, timeout: Duration) -> Option<NetPacket> {
        tokio::time::timeout(timeout, self.incoming_rx.recv())
            .await
            .ok()
            .flatten()
    }
}

fn hex_preview(data: &[u8], max_len: usize) -> String {
    let slice = &data[..data.len().min(max_len)];
    slice.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let target = if args.len() > 1 && !args[1].starts_with('-') {
        &args[1]
    } else {
        "127.0.0.1:23301"
    };

    println!("============================================================");
    println!("🚀 HSR Private Server Protocol Probe CLI");
    println!("🎯 Target Server: {target}");
    println!("============================================================");

    let mut client = match ProbeClient::connect(target).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[-] Connection failed: {e}");
            eprintln!("    Make sure gameserver (March7th or Firefly) is running on {target}!");
            return Ok(());
        }
    };

    // Step 1: Login sequence
    println!("\n[*] Step 1: Performing login sequence...");
    
    // PlayerGetTokenCsReq (Cmd 25)
    let get_token_req = PlayerGetTokenCsReq {
        account_uid: "10001".to_string(),
        token: "test".to_string(),
        ..Default::default()
    };
    let mut buf = Vec::new();
    get_token_req.encode(&mut buf)?;
    client.send_packet(25, &buf).await?;

    if let Some(resp) = client.wait_for_packet(Duration::from_secs(3)).await {
        println!("<<< [RECV] CmdID: {} | Len: {} | Hex: {}", resp.cmd_type, resp.body.len(), hex_preview(&resp.body, 32));
    } else {
        println!("[!] No response to PlayerGetTokenCsReq (might already be authenticated or custom auth)");
    }

    // PlayerLoginCsReq (Cmd 24)
    let login_req = PlayerLoginCsReq::default();
    let mut buf = Vec::new();
    login_req.encode(&mut buf)?;
    client.send_packet(24, &buf).await?;

    if let Some(resp) = client.wait_for_packet(Duration::from_secs(3)).await {
        println!("<<< [RECV] CmdID: {} | Len: {} | Hex: {}", resp.cmd_type, resp.body.len(), hex_preview(&resp.body, 32));
    }

    // PlayerLoginFinishCsReq (Cmd 9)
    let finish_req = PlayerLoginFinishCsReq::default();
    let mut buf = Vec::new();
    finish_req.encode(&mut buf)?;
    client.send_packet(9, &buf).await?;

    if let Some(resp) = client.wait_for_packet(Duration::from_secs(3)).await {
        println!("<<< [RECV] CmdID: {} | Len: {} | Hex: {}", resp.cmd_type, resp.body.len(), hex_preview(&resp.body, 32));
    }

    println!("\n[+] Handshake & Login Sequence completed!");
    println!("============================================================");
    println!("Interactive Probe CLI - Type command number or name:");
    println!("  1. scene     - GetCurSceneInfoCsReq (Cmd 1459)");
    println!("  2. mission   - GetMissionStatusCsReq (Cmd 1259)");
    println!("  3. tutorial  - GetTutorialCsReq (Cmd 1659)");
    println!("  4. challenge - GetCurChallengeCsReq (Cmd 1759)");
    println!("  5. peak      - GetChallengePeakDataCsReq (Cmd 8942)");
    println!("  6. lineup    - GetCurLineupDataCsReq (Cmd 759)");
    println!("  7. auto      - Test all above and print summaries");
    println!("  c. <cmd_id>  - Send custom empty packet with CmdID");
    println!("  q. quit      - Exit probe");
    println!("============================================================");

    let stdin = io::stdin();
    loop {
        print!("PROBE> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() || line.trim().is_empty() {
            continue;
        }

        let cmd = line.trim().to_lowercase();
        match cmd.as_str() {
            "q" | "quit" | "exit" => {
                println!("[*] Exiting probe. Goodbye!");
                break;
            }
            "1" | "scene" => {
                let req = GetCurSceneInfoCsReq::default();
                let mut buf = Vec::new();
                req.encode(&mut buf)?;
                client.send_packet(1459, &buf).await?;
            }
            "2" | "mission" => {
                let req = GetMissionStatusCsReq::default();
                let mut buf = Vec::new();
                req.encode(&mut buf)?;
                client.send_packet(1259, &buf).await?;
            }
            "3" | "tutorial" => {
                let req = GetTutorialCsReq::default();
                let mut buf = Vec::new();
                req.encode(&mut buf)?;
                client.send_packet(1659, &buf).await?;
            }
            "4" | "challenge" => {
                let req = GetCurChallengeCsReq::default();
                let mut buf = Vec::new();
                req.encode(&mut buf)?;
                client.send_packet(1759, &buf).await?;
            }
            "5" | "peak" => {
                let req = GetChallengePeakDataCsReq::default();
                let mut buf = Vec::new();
                req.encode(&mut buf)?;
                client.send_packet(8942, &buf).await?;
            }
            "6" | "lineup" => {
                let req = GetCurLineupDataCsReq::default();
                let mut buf = Vec::new();
                req.encode(&mut buf)?;
                client.send_packet(759, &buf).await?;
            }
            "7" | "auto" => {
                println!("[*] Running automated test suite on standard packets...");
                let test_cases = [
                    (1459, "GetCurSceneInfoCsReq"),
                    (1259, "GetMissionStatusCsReq"),
                    (1659, "GetTutorialCsReq"),
                    (1759, "GetCurChallengeCsReq"),
                    (8942, "GetChallengePeakDataCsReq"),
                    (759, "GetCurLineupDataCsReq"),
                ];

                for (id, name) in test_cases {
                    println!("\n--- Testing {name} (Cmd {id}) ---");
                    client.send_packet(id, &[]).await?;
                    sleep(Duration::from_millis(500)).await;
                    while let Some(resp) = client.wait_for_packet(Duration::from_millis(500)).await {
                        println!("<<< [RECV] CmdID: {} | Len: {} bytes | Hex: {}", resp.cmd_type, resp.body.len(), hex_preview(&resp.body, 48));
                    }
                }
                continue;
            }
            custom => {
                if let Ok(id) = custom.parse::<u16>() {
                    println!("[*] Sending custom empty CsReq for CmdID: {id}...");
                    client.send_packet(id, &[]).await?;
                } else {
                    println!("[?] Unknown command: {cmd}. Type 1..7 or custom cmd ID.");
                    continue;
                }
            }
        }

        // Collect incoming packets for 1.5 seconds
        let deadline = std::time::Instant::now() + Duration::from_millis(1500);
        while std::time::Instant::now() < deadline {
            if let Some(resp) = client.wait_for_packet(Duration::from_millis(200)).await {
                println!("<<< [RECV] CmdID: {} | Len: {} bytes", resp.cmd_type, resp.body.len());
                println!("    Hex: {}", hex_preview(&resp.body, 64));
            }
        }
    }

    Ok(())
}

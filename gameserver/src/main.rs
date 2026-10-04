use anyhow::Result;

pub fn init_tracing() {
    #[cfg(target_os = "windows")]
    let _ = ansi_term::enable_ansi_support();

    let _ = env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("info"))
        .target(env_logger::Target::Stdout)
        .format_timestamp_secs()
        .try_init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    println!("============================================================");
    println!("  [GaC hkrpg ps] Gameserver (v4.5) - ONLINE");
    println!("  Listening on UDP port 23301 (KCP Gateway)");
    println!("  Ready for StarRail client connection...");
    println!("============================================================");
    gameserver::start_gameserver().await
}

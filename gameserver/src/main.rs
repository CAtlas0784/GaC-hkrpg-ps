use anyhow::Result;

use std::io::Write;
use std::sync::Mutex;

struct DualWriter {
    file: Mutex<std::fs::File>,
}

impl Write for DualWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = std::io::stdout().write_all(buf);
        if let Ok(mut f) = self.file.lock() {
            let _ = f.write_all(buf);
            let _ = f.flush();
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let _ = std::io::stdout().flush();
        if let Ok(mut f) = self.file.lock() {
            let _ = f.flush();
        }
        Ok(())
    }
}

pub fn init_tracing() {
    #[cfg(target_os = "windows")]
    let _ = ansi_term::enable_ansi_support();

    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("gameserver.log")
    {
        let writer = DualWriter {
            file: Mutex::new(file),
        };
        let _ = env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("info"))
            .target(env_logger::Target::Pipe(Box::new(writer)))
            .format_timestamp_secs()
            .try_init();
    } else {
        let _ = env_logger::Builder::from_env(env_logger::Env::new().default_filter_or("info"))
            .target(env_logger::Target::Stdout)
            .format_timestamp_secs()
            .try_init();
    }
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

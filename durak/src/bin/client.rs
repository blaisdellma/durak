use anyhow::Result;
use tracing::{info,debug,warn,Level};
use tracing_subscriber as ts;
use tracing_appender as ta;

use durak_players::*;

fn init_log(prefix: &str) -> Result<ta::non_blocking::WorkerGuard> {
    let log_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let (file, guard) = ta::non_blocking(ta::rolling::daily(log_dir,prefix));
    ts::fmt()
        .with_writer(file)
        .with_max_level(Level::DEBUG)
        .with_env_filter({
            ts::EnvFilter::from_default_env()
                .add_directive("durak=debug".parse()?)
                .add_directive("cursive=warn".parse()?)
        }).init();
    debug!("Log init successful");
    Ok(guard)
}

fn run_game_client() -> Result<()> {
    let _guard = init_log("client_log").map_err(|e| { warn!("Log init failed"); e })?;
    let mut player = NetClientDurakPlayer::new(CliPlayer::new(0))?;
    info!("Connected to game server");
    loop {
        match player.wait()? {
            1 => { break; },
            2 => { break; },
            _ => {},
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run_game_client() {
        eprintln!("ERROR: {}",e);
    }
}

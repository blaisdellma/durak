use anyhow::Result;
use rand::thread_rng;
use tracing::{info,debug,warn,error,Level};
use tracing_subscriber as ts;
use tracing_appender as ta;

use durak_core::prelude::*;
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

async fn run_game_server() -> Result<()> {
    let _guard = init_log("server_log").map_err(|e| { warn!("Log init failed"); e })?;
    let mut game = DurakGame::new();

    let mut server = DurakServer::new()?;
    for _ in 0..3 {
        server.wait_connection()?;
        info!("Client connected to server");
    }
    for player in server.get_players()? {
        game.add_player(Box::new(player)).await?;
    }

    game.init(&mut thread_rng()).map_err(|e| { error!("Game initialization error: {}",e); e })?;
    game.run_game().await.map_err(|e| { error!("Game error: {}",e); e })?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_game_server().await {
        eprintln!("ERROR: {}",e);
    }
}

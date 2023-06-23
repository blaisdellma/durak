use anyhow::{anyhow,Result};
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
    // game.add_player(Box::new(TUIDurakPlayer::new(1)),1)?;

    let mut server = DurakServer::new()?;
    for _ in 0..3 {
        server.wait_connection()?;
    }
    for player in server.get_players()? {
        game.add_player(Box::new(player)).await?;
    }

    game.init(&mut thread_rng()).map_err(|e| { error!("Game initialization error: {}",e); e })?;
    game.run_game().await.map_err(|e| { error!("Game error: {}",e); e })?;

    Ok(())
}

async fn run_game_client() -> Result<()> {
    let _guard = init_log("client_log").map_err(|e| { warn!("Log init failed"); e })?;
    let mut player = NetClientDurakPlayer::new(CliPlayer::new(0))?;
    info!("Connected to game server");
    loop {
        match player.wait().await? {
            1 => { break; },
            2 => { break; },
            _ => {},
        }
    }
    Ok(())
}

async fn run_game_test<T: DurakPlayer + 'static>(num_players: usize,player: T) -> Result<()> {
    let _guard = init_log("test_log").map_err(|e| { warn!("Log init failed"); e })?;
    let mut game = DurakGame::new();

    for _ in 0..num_players {
        game.add_player(Box::new(DummyDurakPlayer::new().with_wait(500))).await?;
    }
    game.add_player(Box::new(player)).await?;

    game.init(&mut thread_rng()).map_err(|e| { error!("Game initialization error: {}",e); e })?;
    game.run_game().await.map_err(|e| { error!("Game error: {}",e); e })?;

    Ok(())
}

#[tokio::main]
async fn main() {
    match match std::env::args().skip(1).next() {
        Some(arg) if arg == "server" => run_game_server().await,
        Some(arg) if arg == "client" => run_game_client().await,
        Some(arg) if arg == "test_cli" => run_game_test(2,CliPlayer::new(0)).await,
        Some(arg) if arg == "test_tui" => run_game_test(2,TuiPlayer::new()).await,
        _ => Err(anyhow!("Command option not recognized")),
    } {
        Ok(()) => {},
        Err(e) => eprintln!("ERROR: {}",e),
    }
}

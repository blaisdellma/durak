use rand::thread_rng;
use tracing::{info,debug,warn,error,Level};
use tracing_subscriber as ts;
use tracing_appender::{self as ta,non_blocking as tanb};

use durak::GameState;

mod tuidurakplayer;
use tuidurakplayer::TUIDurakPlayer;

fn init_log() -> Result<tanb::WorkerGuard,Box<dyn std::error::Error>> {
    let log_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let (file, guard) = ta::non_blocking(ta::rolling::daily(log_dir,"log"));
    ts::fmt()
        .with_writer(file)
        .with_max_level(Level::TRACE)
        .init();
    debug!("Log init successfull");
    Ok(guard)
}

fn test_game() -> Result<(),Box<dyn std::error::Error>> {
    let _guard = init_log().map_err(|e| { warn!("Log init failed"); e })?;
    println!("Welcome to DURAK!");
    info!("Welcome to DURAK!");

    let mut state = GameState::new();

    state.add_player(Box::new(TUIDurakPlayer::new(1)),1)?;
    state.add_player(Box::new(TUIDurakPlayer::new(2)),2)?;
    state.add_player(Box::new(TUIDurakPlayer::new(3)),3)?;

    state.init(&mut thread_rng()).map_err(|e| { error!("Game initialization error: {}",e); e })?;
    state.run_game().map_err(|e| { error!("Game error: {}",e); e })?;

    Ok(())
}

fn main() {
    match test_game() {
        Ok(()) => println!("All's good"),
        Err(e) => println!("ERROR: {}",e),
    }
}

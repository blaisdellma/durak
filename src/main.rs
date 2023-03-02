use rand::thread_rng;
use tracing::{info,debug,warn,error,Level};
use tracing_subscriber as ts;
use tracing_appender as ta;

use durak::{DurakGame,DurakResult,DurakPlayer};

mod tuidurakplayer;
use tuidurakplayer::*;

mod tui2durakplayer;
use tui2durakplayer::*;

mod dummydurakplayer;
use dummydurakplayer::*;

mod durak_net;
use durak_net::*;

fn init_log(prefix: &str) -> DurakResult<ta::non_blocking::WorkerGuard> {
    let log_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let (file, guard) = ta::non_blocking(ta::rolling::daily(log_dir,prefix));
    ts::fmt()
        .with_writer(file)
        .with_max_level(Level::TRACE)
        .init();
    debug!("Log init successfull");
    Ok(guard)
}

fn run_game_server() -> DurakResult<()> {
    let _guard = init_log("server_log").map_err(|e| { warn!("Log init failed"); e })?;
    let mut game = DurakGame::new();
    // game.add_player(Box::new(TUIDurakPlayer::new(1)),1)?;

    let mut server = DurakServer::new()?;
    for _ in 0..3 {
        server.wait_connection()?;
    }
    for player in server.get_players()? {
        game.add_player(Box::new(player))?;
    }

    game.init(&mut thread_rng()).map_err(|e| { error!("Game initialization error: {}",e); e })?;
    game.run_game().map_err(|e| { error!("Game error: {}",e); e })?;

    Ok(())
}

fn run_game_client() -> DurakResult<()> {
    let _guard = init_log("client_log").map_err(|e| { warn!("Log init failed"); e })?;
    let mut player = NetClientDurakPlayer::new(TUIDurakPlayer::new(0))?;
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

fn run_game_test<T: DurakPlayer + 'static>(num_players: usize,player: T) -> DurakResult<()> {
    let _guard = init_log("test_log").map_err(|e| { warn!("Log init failed"); e })?;
    let mut game = DurakGame::new();

    for _ in 0..num_players {
        game.add_player(Box::new(DummyDurakPlayer::new().with_wait(2000)))?;
    }
    game.add_player(Box::new(player))?;

    game.init(&mut thread_rng()).map_err(|e| { error!("Game initialization error: {}",e); e })?;
    game.run_game().map_err(|e| { error!("Game error: {}",e); e })?;

    Ok(())
}

fn main() {
    match match std::env::args().skip(1).next() {
        Some(arg) if arg == "server" => run_game_server(),
        Some(arg) if arg == "client" => run_game_client(),
        Some(arg) if arg == "test" => run_game_test(2,TUINewDurakPlayer::new()),
        _ => Err("Command option not recognized".into()),
    } {
        Ok(()) => println!("All's good"),
        Err(e) => println!("ERROR: {}",e),
    }
}
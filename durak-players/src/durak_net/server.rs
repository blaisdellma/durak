use std::net::TcpListener;

use anyhow::Result;

use tracing::info;

use crate::NetServerDurakPlayer;

pub struct DurakServer {
    listener: TcpListener,
    players: Vec<NetServerDurakPlayer>,
}

impl DurakServer {
    pub fn new() -> Result<Self> {
        Ok(DurakServer {
            listener: TcpListener::bind("127.0.0.1:8080")?,
            players: Vec::new(),
        })
    }

    pub fn wait_connection(&mut self) -> Result<()> {
        match self.listener.accept() {
            Ok((socket,addr)) => {
                info!("Connection at {}",addr);
                self.players.push(NetServerDurakPlayer::new(socket));
            },
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }

    pub fn get_players(self) -> Result<Vec<NetServerDurakPlayer>> {
        Ok(self.players)
    }
}

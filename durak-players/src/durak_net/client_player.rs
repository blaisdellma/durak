use std::net::TcpStream;
use std::io::{Write,BufWriter,Read,BufRead,BufReader};

// use tracing::{info};
use serde::{Serialize,Deserialize};

use durak_core::prelude::*;

pub struct NetClientDurakPlayer<T: DurakPlayer> {
    engine: T,
    stream: TcpStream,
}

impl<T: DurakPlayer> NetClientDurakPlayer<T> {
    fn process_query<A: for<'a> Deserialize<'a>,B: Serialize,F: Fn(&mut Self,&A)->DurakResult<B>>(&mut self, func: F) -> DurakResult<()> {
        let mut stream = BufReader::new(&mut self.stream);
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;
        let string = String::from_utf8(buf)?;
        let data = serde_json::from_str(&string)?;
        let ret = func(self,&data)?;
        let content = serde_json::to_string(&ret)?;
        let mut stream = BufWriter::new(&mut self.stream);
        stream.write(content.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

impl<T: DurakPlayer> NetClientDurakPlayer<T> {
    pub fn new(durak_player: T) -> DurakResult<Self> {
        Ok(NetClientDurakPlayer {
            engine: durak_player,
            stream: TcpStream::connect("127.0.0.1:8080")?,
        })
    }

    pub fn wait(&mut self) -> DurakResult<usize> {
        let mut stream = BufReader::new(&mut self.stream);
        let mut data = String::new();
        let _ = stream.read_line(&mut data)?;
        match data.get(0..1) {
            None => {},
            Some("A") => {
                self.process_query(|player,state| player.engine.attack(state))?;
            },
            Some("D") => {
                self.process_query(|player,state| player.engine.defend(state))?;
            },
            Some("P") => {
                self.process_query(|player,state| player.engine.pile_on(state))?;
            },
            Some("O") => {
                self.process_query(|player,state| player.engine.observe_move(state))?;
            },
            Some("I") => {
                self.process_query(|player,player_info| player.engine.get_id(player_info))?;
            },
            Some("W") => {
                self.engine.won()?;
                return Ok(1);
            },
            Some("L") => {
                self.engine.lost()?;
                return Ok(2);
            },
            Some("M") => {
                self.process_query(|player: &mut Self, msg: &String| player.engine.message(&msg))?;
                return Ok(3);
            },
            Some("E") => {
                self.process_query(|player: &mut Self, error_msg: &String| player.engine.error(&error_msg))?;
                return Ok(3);
            },
            _ => {},
        }
        Ok(0)
    }
}

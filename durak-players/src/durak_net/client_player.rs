use std::net::TcpStream;
use std::io::{Write,BufWriter,Read,BufRead,BufReader};

use anyhow::Result;

// use tracing::{info};
use serde::{Serialize,Deserialize};

use durak_core::prelude::*;

pub struct NetClientDurakPlayer<T: DurakPlayer> {
    engine: T,
    stream: TcpStream,
}

impl<T: DurakPlayer> NetClientDurakPlayer<T> {
    pub fn new(durak_player: T) -> Result<Self> {
        Ok(NetClientDurakPlayer {
            engine: durak_player,
            stream: TcpStream::connect("127.0.0.1:8080")?,
        })
    }

    fn stream_read<U: for<'a> Deserialize<'a>>(&mut self) -> Result<U> {
        let mut stream = BufReader::new(&mut self.stream);
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;
        let string = String::from_utf8(buf)?;
        let data = serde_json::from_str(&string)?;
        Ok(data)
    }

    fn stream_write<U: Serialize>(&mut self, data: U) -> Result<()> {
        let content = serde_json::to_string(&data)?;
        let mut stream = BufWriter::new(&mut self.stream);
        stream.write(content.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    // async fn process_query<A: for<'a> Deserialize<'a>,B: Serialize,F: Fn(&mut Self,&A)->Pin<Box<dyn Future<Output=Result<B>>>>>(&mut self, func: F) -> Result<()> {
    //     let mut stream = BufReader::new(&mut self.stream);
    //     let mut buf = Vec::new();
    //     stream.read_to_end(&mut buf)?;
    //     let string = String::from_utf8(buf)?;
    //     let data = serde_json::from_str(&string)?;
    //     let ret = func(self,&data).await?;
    //     let content = serde_json::to_string(&ret)?;
    //     let mut stream = BufWriter::new(&mut self.stream);
    //     stream.write(content.as_bytes())?;
    //     stream.flush()?;
    //     Ok(())
    // }

    pub async fn wait(&mut self) -> Result<usize> {
        let mut stream = BufReader::new(&mut self.stream);
        let mut data = String::new();
        let _ = stream.read_line(&mut data)?;
        match data.get(0..1) {
            None => {},
            Some("A") => {
                // self.process_query(|player,state| player.engine.attack(state)).await?;
                
                let state = self.stream_read()?;
                let ret = self.engine.attack(&state).await?;
                self.stream_write(ret)?;
        
            },
            Some("D") => {
                // self.process_query(|player,state| player.engine.defend(state)).await?;
                
                let state = self.stream_read()?;
                let ret = self.engine.defend(&state).await?;
                self.stream_write(ret)?;
        
            },
            Some("P") => {
                // self.process_query(|player,state| player.engine.pile_on(state)).await?;
                
                let state = self.stream_read()?;
                let ret = self.engine.pile_on(&state).await?;
                self.stream_write(ret)?;
        
            },
            Some("O") => {
                // self.process_query(|player,state| player.engine.observe_move(state)).await?;
                
                let state = self.stream_read()?;
                let ret = self.engine.observe_move(&state).await?;
                self.stream_write(ret)?;
        
            },
            Some("I") => {
                // self.process_query(|player,player_info| player.engine.get_id(player_info)).await?;
                
                let player_info = self.stream_read()?;
                let ret = self.engine.get_id(&player_info).await?;
                self.stream_write(ret)?;
        
            },
            Some("W") => {
                self.engine.won().await?;
                return Ok(1);
            },
            Some("L") => {
                self.engine.lost().await?;
                return Ok(2);
            },
            Some("M") => {
                // self.process_query(|player: &mut Self, msg: &String| player.engine.message(&msg)).await?;
                
                let msg: String = self.stream_read()?;
                let ret = self.engine.message(&msg).await?;
                self.stream_write(ret)?;
        
                return Ok(3);
            },
            Some("E") => {
                // self.process_query(|player: &mut Self, error_msg: &String| player.engine.error(&error_msg)).await?;
                
                let error_msg: String = self.stream_read()?;
                let ret = self.engine.error(&error_msg).await?;
                self.stream_write(ret)?;
        
                return Ok(3);
            },
            _ => {},
        }
        Ok(0)
    }
}

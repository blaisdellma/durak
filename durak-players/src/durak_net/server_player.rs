use std::net::TcpStream;
use std::io::{Write,BufWriter,BufRead,BufReader};

use anyhow::Result;
use async_trait::async_trait;

// use tracing::{info};
use serde::{Serialize,Deserialize};

use durak_core::prelude::*;

pub struct NetServerDurakPlayer {
    pub id: u64,
    stream: TcpStream,
}

impl NetServerDurakPlayer {
    pub fn new(stream: TcpStream) -> Self {
        NetServerDurakPlayer {
            id: 0,
            stream,
        }
    }

    fn query_client<A: Serialize, B: for<'b> Deserialize<'b>>(&mut self, sig: &str, data: &A) -> Result<B> {
        let json = serde_json::to_string(data)?;
        let content = json.as_bytes();
        let mut stream = BufWriter::new(&mut self.stream);
        stream.write(sig.as_bytes())?;
        stream.write(content)?;
        stream.flush()?;
        drop(stream);
        let mut stream = BufReader::new(&mut self.stream);
        let mut data = String::new();
        let _ = stream.read_line(&mut data)?;
        let ret: B = serde_json::from_str(&data)?;
        Ok(ret)
    }

}

#[async_trait]
impl DurakPlayer for NetServerDurakPlayer {
    async fn attack(&mut self, state: &ToPlayState) -> Result<Action> {
        self.query_client("A\n",state)
    }

    async fn defend(&mut self, state: &ToPlayState) -> Result<Action> {
        self.query_client("D\n",state)
    }

    async fn pile_on(&mut self, state: &ToPlayState) -> Result<Vec<Card>> {
        self.query_client("P\n",state)
    }

    async fn observe_move(&mut self, state: &ToPlayState) -> Result<()> {
        self.query_client("O\n",state)
    }

    async fn get_id(&mut self, player_info: &Vec<PlayerInfo>) -> Result<u64> {
        self.query_client("I\n",player_info)
    }

    async fn won(&mut self) -> Result<()> {
        let mut stream = BufWriter::new(&mut self.stream);
        stream.write("W\n".as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    async fn lost(&mut self) -> Result<()> {
        let mut stream = BufWriter::new(&mut self.stream);
        stream.write("L\n".as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    async fn message(&mut self, msg: &str) -> Result<()> {
        let mut stream = BufWriter::new(&mut self.stream);
        stream.write("M\n".as_bytes())?;
        stream.write(msg.as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    async fn error(&mut self, error: &str) -> Result<()> {
        let mut stream = BufWriter::new(&mut self.stream);
        stream.write("E\n".as_bytes())?;
        stream.write(error.as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

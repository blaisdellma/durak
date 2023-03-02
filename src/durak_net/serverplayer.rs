use std::net::TcpStream;
use std::io::{Write,BufWriter,BufRead,BufReader};

// use tracing::{info};
use serde::{Serialize,Deserialize};

use durak::{*,card::*,toplaystate::*};

pub struct NetServerDurakPlayer {
    pub id: u64,
    stream: TcpStream,
}

impl NetServerDurakPlayer {
    pub fn new(stream: TcpStream) -> Self {
        NetServerDurakPlayer {
            id: 0,
            stream: stream,
        }
    }

    fn query_client<A: Serialize, B: for<'b> Deserialize<'b>>(&mut self, sig: &str, data: &A) -> DurakResult<B> {
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

impl DurakPlayer for NetServerDurakPlayer {
    fn attack(&mut self, state: &ToPlayState) -> DurakResult<Option<Card>> {
        self.query_client("A\n",state)
    }

    fn defend(&mut self, state: &ToPlayState) -> DurakResult<Option<Card>> {
        self.query_client("D\n",state)
    }

    fn pile_on(&mut self, state: &ToPlayState) -> DurakResult<Vec<Card>> {
        self.query_client("P\n",state)
    }

    fn observe_move(&mut self, state: &ToPlayState) -> DurakResult<()> {
        self.query_client("O\n",state)
    }

    fn get_id(&mut self, player_info: &Vec<PlayerInfo>) -> DurakResult<u64> {
        self.query_client("I\n",player_info)
    }

    fn won(&mut self) -> DurakResult<()> {
        let mut stream = BufWriter::new(&mut self.stream);
        stream.write("W\n".as_bytes())?;
        stream.flush()?;
        Ok(())
    }

    fn lost(&mut self) -> DurakResult<()> {
        let mut stream = BufWriter::new(&mut self.stream);
        stream.write("L\n".as_bytes())?;
        stream.flush()?;
        Ok(())
    }
}

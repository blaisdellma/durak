use durak_core::prelude::*;

pub struct DummyDurakPlayer {
    id: u64,
    wait: u64,
}

impl DummyDurakPlayer {
    pub fn new() -> Self {
        Self { id: 1, wait: 0 }
    }

    pub fn with_wait(mut self, wait: u64) -> Self {
        self.wait = wait;
        self
    }
    
    fn wait(&self) {
        if self.wait > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.wait));
        }
    }
}

impl DurakPlayer for DummyDurakPlayer {
    fn attack(&mut self, state: &ToPlayState) -> DurakResult<Option<Card>> {
        self.wait();
        for &card in state.hand.iter() {
            if state.validate_attack(&Some(card)).is_ok() {
                return Ok(Some(card));
            }
        }
        Ok(None)
    }

    fn defend(&mut self, state: &ToPlayState) -> DurakResult<Option<Card>> {
        self.wait();
        for &card in state.hand.iter() {
            if state.validate_defense(&Some(card)).is_ok() {
                return Ok(Some(card));
            }
        }
        Ok(None)
    }

    fn pile_on(&mut self, _state: &ToPlayState) -> DurakResult<Vec<Card>> {
        self.wait();
        Ok(Vec::new())
    }

    fn get_id(&mut self, player_info: &Vec<PlayerInfo>) -> DurakResult<u64> {
        for info in player_info {
            if self.id <= info.id {
                self.id = info.id + 1;
            }
        }
        Ok(self.id)
    }
}

use durak_core::prelude::*;

use anyhow::Result;

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
    fn attack(&mut self, state: &ToPlayState) -> Result<Action> {
        self.wait();
        for &card in state.hand.iter() {
            if state.validate_attack(&Action::Play(card)).is_ok() {
                return Ok(Action::Play(card));
            }
        }
        Ok(Action::Pass)
    }

    fn defend(&mut self, state: &ToPlayState) -> Result<Action> {
        self.wait();
        for &card in state.hand.iter() {
            if state.validate_defense(&Action::Play(card)).is_ok() {
                return Ok(Action::Play(card));
            }
        }
        Ok(Action::Pass)
    }

    fn pile_on(&mut self, _state: &ToPlayState) -> Result<Vec<Card>> {
        self.wait();
        Ok(Vec::new())
    }

    fn get_id(&mut self, player_info: &Vec<PlayerInfo>) -> Result<u64> {
        for info in player_info {
            if self.id <= info.id {
                self.id = info.id + 1;
            }
        }
        Ok(self.id)
    }
}

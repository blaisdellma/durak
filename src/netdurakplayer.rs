use tracing::{warn};

use durak::{*,card::*};

// TODO

struct NetServerDurakPlayer {
}

struct NetClientDurakPlayer<T: DurakPlayer> {
    engine: T,
}

impl<T> NetClientDurakPlayer<T> {
    fn new(durak_player: T) -> Self {
        NetClientDurakPlayer { engine: durak_player }
    }
}

impl DurakPlayer for NetClientDurakPlayer {
    fn attack(&self, state: &ToPlayState) -> Action {
        None
    }

    fn defense(&self, state: &ToPlayState) -> Action {
        None
    }

    fn pile_on(&self, state: &ToPlayState) -> Vec<Card> {
        vec![]
    }

    fn won(&self) {
    }

    fn lost(&self) {
    }
}

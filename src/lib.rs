use std::borrow::Cow;
use rand::Rng;
use tracing::debug;
pub mod card;
use card::*;
pub mod toplaystate;
use toplaystate::*;

fn transfer_card(v_from: &mut Vec<Card>, v_to: &mut Vec<Card>, card: &Card) {
    let mut ind = 0;
    while ind < v_from.len() && v_from[ind] != *card { ind += 1};
    if ind < v_from.len() {
        v_to.push(v_from.swap_remove(ind));
    }
}

pub trait DurakPlayer: Send + Sync {
    fn attack(&self, state: &ToPlayState) -> Option<Card>;
    fn defend(&self, state: &ToPlayState) -> Option<Card>;
    fn pile_on(&self, state: &ToPlayState) -> Vec<Card>;
    fn lost(&self);
    fn won(&self);
}

struct Player {
    id: usize,
    hand: Vec<Card>,
    player_engine: Box<dyn DurakPlayer>,
}

#[derive(PartialEq)]
enum GameTurnType {
    Attack,
    Defense,
    PileOn,
    EndRound,
    GameEnd,
}

pub struct GameState {
    trump: Suit,
    players: Vec<Player>,
    draw_pile: Vec<Card>,
    attack_cards: Vec<Card>,
    defense_cards: Vec<Card>,
    discarded_cards: Vec<Card>,
    defender: usize,
    attacker: usize,
    to_play: usize,
    turn_type: GameTurnType,
}

impl GameState {

    pub fn new() -> Self {
        GameState {
            trump: Suit::Hearts,
            players: Vec::new(),
            draw_pile: Vec::new(),
            attack_cards: Vec::new(),
            defense_cards: Vec::new(),
            discarded_cards: Vec::new(),
            defender: 0usize,
            attacker: 0usize,
            to_play: 0usize,
            turn_type: GameTurnType::Attack,
        }
    }

    pub fn add_player(&mut self, engine: Box<dyn DurakPlayer>, id: usize) -> Result<(),Box<dyn std::error::Error>> {
        if self.players.iter().any(|player| player.id == id) { return Err("Duplicate player id".into()); }
        if self.players.len() >= 6 { return Err("Cannot add more than 6 players".into()); }
        self.players.push(Player {
            id: id,
            hand: Vec::new(),
            player_engine: engine,
        });
        debug!("Added player # {}", id);
        Ok(())
    }

    // fn get_player_by_id(&self, id: &usize) -> Option<&Player> {
    //     self.players.iter().filter(|p| p.id == *id).take(1).next()
    // }
    
    // fn get_in_play(&self) -> Vec<&Player> {
    //     self.players.iter().filter(|p| p.hand.len() != 0).collect()
    // }

    pub fn init<R: Rng>(&mut self, rng: &mut R) -> Result<(),Box<dyn std::error::Error>> {
        debug!("Initializing game");
        if self.players.len() < 2 {
            return Err(format!("Need at least two players to initialize game, only have ({})",self.players.len()).into());
        } else if self.players.len() > 6 {
            return Err("Can't have more than 6 players".into());
        }

        let mut deck = Deck::init(rng)?;
        self.trump = deck.get_trump()?;
        debug!("Trump suit is {}",self.trump);
        for player in &mut self.players {
            deck.deal_cards(6,&mut player.hand)?;
        }
        self.draw_pile.extend(deck.all_cards_left());

        for player in &self.players { debug!("Player # {} has cards: {}",player.id,hand_fmt(&player.hand)); }

        self.attacker = 0;
        self.defender = 1;

        self.to_play = self.attacker;
        self.turn_type = GameTurnType::Attack;

        Ok(())
    }

    // fn check_end(&self) -> bool {
    //     if self.draw_pile.len() > 0 { return false; }
    //     if self.get_in_play().len() > 1 { return false; }
    //     true
    // }

    // refills a players hand from the talon up to 6 cards
    fn refill_from_talon(&mut self, player_ind: usize) {
        while self.players[player_ind].hand.len() < 6 && self.draw_pile.len() > 0 {
            self.players[player_ind].hand.push(self.draw_pile.pop().unwrap());
        }
    }

    fn refill_players_hands(&mut self) {
        debug!("Refilling player's hands");
        self.refill_from_talon(self.attacker);
        let mut player_ind = (self.attacker + 2 ) % self.players.len();
        while player_ind != self.attacker {
            self.refill_from_talon(player_ind);
            player_ind = (player_ind + 1) % self.players.len();
        }
        self.refill_from_talon(self.defender);
    }

    pub fn run_game(mut self) -> Result<(),Box<dyn std::error::Error>> {
        while self.turn_type != GameTurnType::GameEnd {
            self.play_turn()?;
        }

        // notify players of win/lost status
        let handles : Vec<_> = self.players.into_iter().map(|player| {
            std::thread::spawn(move || {
                if player.hand.len() == 0 {
                    debug!("Player {} won ", player.id);
                    player.player_engine.won();
                } else {
                    debug!("Player {} lost ", player.id);
                    player.player_engine.lost();
                }
            })
        }).collect();
        for h in handles { h.join().unwrap(); }

        Ok(())
    }

    fn play_turn(&mut self) -> Result<(),Box<dyn std::error::Error>> {
        debug!("Taking turn");
        let to_play_state = gen_to_play_state(&self);
        for player in &self.players {
            debug!("Player # {} has cards: {}",player.id,player.hand.iter().map(|c| format!("{:>4}",format!("{}",c))).collect::<String>());
        }
        debug!("Player # {} is the attacker",self.players[self.attacker].id);
        debug!("Player # {} is the defender",self.players[self.defender].id);
        debug!("Player # {} is playing",self.players[self.to_play].id);

        match &self.turn_type {
            GameTurnType::Attack => {
                debug!("Attack turn");
                let attack = {
                    if self.players[self.to_play].hand.len() == 0 { 
                        debug!("Skipping turn because player has no cards left");
                        None
                    } else if self.players[self.defender].hand.len() == 0 {
                        debug!("Skipping turn because defender has no cards left");
                        None
                    } else {
                        debug!("Querying player for attack");
                        let attack = self.players[self.to_play].player_engine.attack(&to_play_state);
                        if attack.is_none() { debug!("Player has selected to pass"); }
                        attack
                    }
                };
                match attack {
                    Some(attack_card) => {
                        debug!("Player has selected {}",attack_card);
                        to_play_state.validate_attack(&attack)?;
                        transfer_card(&mut self.players[self.to_play].hand,&mut self.attack_cards,&attack_card);
                        self.to_play = self.defender;
                        self.turn_type = GameTurnType::Defense;
                    },
                    None => {
                        // bump to next player's turn skipping the defender
                        // if wrapped around to the attacker then the round is over
                        self.to_play = (self.to_play + 1) % self.players.len();
                        while self.to_play == self.defender || self.players[self.to_play].hand.len() == 0 {
                            if self.to_play == self.attacker { break; }
                            self.to_play = (self.to_play + 1) % self.players.len();
                        }
                        if self.to_play == self.attacker {
                            debug!("Ending round because all attackers passed");
                            // defender has priority for next round
                            self.to_play = self.defender;
                            self.turn_type = GameTurnType::EndRound;
                        }
                    },
                }
            },
            GameTurnType::Defense => {
                debug!("Defense turn");
                match self.players[self.to_play].player_engine.defend(&to_play_state) {
                    Some(defense_card) => {
                        debug!("Player has selected {}",defense_card);
                        to_play_state.validate_defense(&Some(defense_card))?;
                        transfer_card(&mut self.players[self.to_play].hand,&mut self.defense_cards,&defense_card);
                        if self.defense_cards.len() == 6 || self.players[self.to_play].hand.len() == 0 {
                            debug!("Ending round because attack has been successfully defended");
                            // defender has priority for next round
                            self.to_play = self.defender;
                            self.turn_type = GameTurnType::EndRound;
                        } else {
                            self.to_play = self.attacker;
                            self.turn_type = GameTurnType::Attack;
                        }
                    },
                    None => {
                        debug!("Player has selected to pass");
                        self.turn_type = GameTurnType::PileOn;
                    },
                }
            },
            GameTurnType::PileOn => {
                debug!("Pile on turn");
                for ind_pile in 0..self.players.len() {
                    if ind_pile == self.defender { continue; }
                    self.to_play = ind_pile;
                    let to_play_state = gen_to_play_state(&self);
                    let pile_on_cards = self.players[ind_pile].player_engine.pile_on(&to_play_state);
                    to_play_state.validate_pile_on(&pile_on_cards)?;
                    debug!("Player {} has piled on {}",self.players[ind_pile].id,hand_fmt(&pile_on_cards));

                    for card in pile_on_cards {
                        transfer_card(&mut self.players[ind_pile].hand,&mut self.attack_cards,&card);
                    }
                }
                self.to_play = (self.defender + 1) % self.players.len();
                self.turn_type = GameTurnType::EndRound;
            },
            GameTurnType::EndRound => {
                if self.to_play == self.defender {
                    // successful defense
                    self.discarded_cards.append(&mut self.attack_cards);
                    self.discarded_cards.append(&mut self.defense_cards);
                } else {
                    // unsuccessful defense
                    self.players[self.defender].hand.append(&mut self.attack_cards);
                    self.players[self.defender].hand.append(&mut self.defense_cards);
                }
                self.refill_players_hands();
                if self.players.iter().map(|player| { 
                    match player.hand.len() {
                        0 => 0,
                        _ => 1,
                    }
                }).fold(0, |a,b| a + b) <= 1 {
                    self.turn_type = GameTurnType::GameEnd;
                    return Ok(());
                }
                while self.players[self.to_play].hand.len() == 0 {
                    self.to_play = (self.to_play + 1) % self.players.len();
                }
                self.attacker = self.to_play;
                self.defender = (self.attacker + 1) % self.players.len();
                while self.players[self.defender].hand.len() == 0 {
                    self.defender = (self.defender + 1) % self.players.len();
                }
                self.turn_type = GameTurnType::Attack;
            },
            GameTurnType::GameEnd => {},
        }
        Ok(())
    }
}

fn gen_to_play_state<'a>(state: &'a GameState) -> ToPlayState<'a> {
    ToPlayState {
        attack_cards: Cow::Borrowed(&state.attack_cards),
        defense_cards: Cow::Borrowed(&state.defense_cards),
        hand: Cow::Borrowed(&state.players[state.to_play].hand),
        trump: state.trump,
        player_hand_sizes: state.players.iter()
            .map(|p| p.hand.len()).collect(),
        attacker: state.attacker,
        defender: state.defender,
        to_play: state.to_play,
    }
}

//! The core game engine.

use std::borrow::Cow;

use anyhow::{bail,Result};
use async_trait::async_trait;
use rand::Rng;
use tracing::{debug,error};
use serde::{Serialize,Deserialize};

use crate::prelude::*;
use crate::card::transfer_card;

/// Defines the actions available to a player on attack and defense turns.
#[allow(missing_docs)]
#[derive(PartialEq,Serialize,Deserialize)]
pub enum Action {
    Play(Card),
    Pass,
}

/// Used to signify if the player is ready for another game.
#[allow(missing_docs)]
#[derive(PartialEq,Serialize,Deserialize)]
pub enum Ready {
    Yes,
    No,
}

/// Trait defining player behavior.
/// Implement this when making a player client.
#[async_trait]
pub trait DurakPlayer: Send + Sync {
    /// Plays an attack turn.
    async fn attack(&mut self, state: &ToPlayState) -> Result<Action>;

    /// Plays a defense turn.
    async fn defend(&mut self, state: &ToPlayState) -> Result<Action>;

    /// Plays a pile on turn.
    async fn pile_on(&mut self, state: &ToPlayState) -> Result<Vec<Card>>;

    /// Not playing a turn, but is sent whenever another player plays a turn to update player
    /// client game state.
    async fn observe_move(&mut self, state: &ToPlayState) -> Result<()> {
        _ = state;
        Ok(())
    }

    /// Returns a unique ID for the player.
    /// [`PlayerInfo`] contains all the other player's IDs. No duplicates allowed.
    async fn get_id(&mut self, player_info: &Vec<PlayerInfo>) -> Result<u64>;

    /// A notification that the player has lost the game.
    async fn lost(&mut self) -> Result<Ready> {
        Ok(Ready::Yes)
    }

    /// A notification that the player has won the game.
    /// Or rather, just not lost the game.
    async fn won(&mut self) -> Result<Ready> {
        Ok(Ready::Yes)
    }

    /// Any non-error notification to the player from the game engine
    async fn message(&mut self, msg: &str) -> Result<()> {
        _ = msg;
        Ok(())
    }

    /// A notification that there has been some error and the game engine is shutting down.
    async fn error(&mut self, error: &str) -> Result<()> {
        _ = error;
        Ok(())
    }
}

struct Player {
    id: u64,
    hand: Vec<Card>,
}

#[derive(PartialEq)]
enum GameTurnType {
    Attack,
    Defense,
    PileOn,
    EndRound,
    GameEnd,
}

struct GameState {
    trump: Suit,
    players: Vec<Player>,
    attackers: Vec<usize>, // indices for attackers for current round
    attackers_passed: Vec<usize>, // indices for attackers who have passed since last attack
    draw_pile: Vec<Card>,
    attack_cards: Vec<Card>,
    defense_cards: Vec<Card>,
    discarded_cards: Vec<Card>,
    defender: usize,
    last_attacker: usize, // the last attacker (used for reference during defense turns)
    to_play: usize, // whoever's turn it currently is
    turn_type: GameTurnType,
}

/// The durak game engine.
pub struct DurakGame {
    state: GameState,
    engines: Vec<Box<dyn DurakPlayer>>,
}

/// The results of a game of Durak
pub struct DurakGameResult {
    winner: Option<(Box<dyn DurakPlayer>,Ready)>,
    losers: Vec<(Box<dyn DurakPlayer>,Ready)>,
}

impl DurakGame {
    /// Create a new game.
    pub fn new() -> Self {
        DurakGame {
            state: GameState::new(),
            engines: Vec::new(),
        }
    }

    /// Add a player to the game. Will call [`DurakPlayer::get_id()`] so make sure player client is
    /// initialized first.
    pub async fn add_player(&mut self, mut engine: Box<dyn DurakPlayer>) -> Result<()> {
        let id = engine.get_id(&get_player_info(&self.state)).await?;
        self.state.add_player(id)?;
        self.engines.push(engine);
        debug!("Added player # {}", id);
        Ok(())
    }

    /// Initialize the game. Deals cards to players and decides what the trump suit is based on
    /// RNG.
    pub fn init<R: Rng>(&mut self, rng: &mut R) -> Result<()> {
        self.state.init(rng)
    }

    /// Start the game.
    pub async fn run_game(mut self) -> Result<DurakGameResult> {
        let mut result = DurakGameResult {
            winner: None,
            losers: vec![],
        };
        let mut set = tokio::task::JoinSet::new();
        match self.game_loop().await {
            Ok(()) => {
                // notify players of win/lost status
                for (player, mut engine) in std::iter::zip(self.state.players,self.engines) {
                    if player.hand.len() == 0 {
                        debug!("Player {} won ", player.id);
                        set.spawn( async move {
                            engine.won().await.map(|ready| (player,engine,ready))
                        });
                    } else {
                        debug!("Player {} lost ", player.id);
                        set.spawn( async move {
                            engine.lost().await.map(|ready| (player,engine,ready))
                        });
                    }
                }
                while let Some(res) = set.join_next().await {
                    match res {
                        Ok(Ok((player,engine,ready))) => {
                            if player.hand.len() == 0 {
                                result.winner = Some((engine,ready));
                            } else {
                                result.losers.push((engine,ready));
                            }
                        },
                        Ok(Err(e)) => { error!("Error: {}", e); },
                        Err(e) => { error!("Error: {}", e); },
                    }
                }
                Ok(result)
            },
            Err(e) => {
                for (player, mut engine) in std::iter::zip(self.state.players,self.engines) {
                    let err_str = format!("{}",e); 
                    set.spawn( async move {
                        engine.error(&err_str).await.map(|_| (player,engine,Ready::No))
                    });
                }
                while let Some(res) = set.join_next().await {
                    match res {
                        Ok(_) => (),
                        Err(e) => { error!("Error: {}", e); },
                    }
                }
                Err(e)
            },
        }
    }

    async fn game_loop(&mut self) -> Result<()> {
        while self.state.turn_type != GameTurnType::GameEnd {
            self.state.play_turn(&mut self.engines).await?;
            for (i,engine) in self.engines.iter_mut().enumerate() {
                let to_play_state = gen_to_play_state_w_hand(&self.state,i);
                engine.observe_move(&to_play_state).await?;
            }
        }
        Ok(())
    }
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            trump: Suit::Hearts,
            players: Vec::new(),
            attackers: Vec::new(),
            attackers_passed: Vec::new(),
            draw_pile: Vec::with_capacity(36),
            attack_cards: Vec::new(),
            defense_cards: Vec::new(),
            discarded_cards: Vec::new(),
            defender: 0usize,
            // attacker: 0usize,
            last_attacker: 0usize,
            to_play: 0usize,
            turn_type: GameTurnType::Attack,
        }
    }

    pub fn add_player(&mut self, id: u64) -> Result<()> {
        if self.players.iter().any(|player| player.id == id) { bail!("Duplicate player id"); }
        if self.players.len() >= 6 { bail!("Cannot add more than 6 players"); }
        self.players.push(Player {
            id,
            hand: Vec::new(),
        });
        Ok(())
    }

    pub fn init<R: Rng>(&mut self, rng: &mut R) -> Result<()> {
        debug!("Initializing game");
        if self.players.len() < 2 {
            bail!("Need at least two players to initialize game, only have ({})",self.players.len());
        } else if self.players.len() > 6 {
            bail!("Can't have more than 6 players");
        }

        // shuffle deck
        let mut in_order_cards = (0..36).map(|i| Card::try_from(i)).collect::<Result<Vec<Card>>>()?;
        for _ in 0..36 {
            let index = rng.gen_range(0..in_order_cards.len());
            self.draw_pile.push(in_order_cards.swap_remove(index));
        }

        // deal cards
        for _ in 0..6 {
            for hand in self.players.iter_mut().map(|p| &mut p.hand) {
                hand.push(self.draw_pile.pop().unwrap());
            }
        }

        // determine trump suit
        self.trump = self.draw_pile[0].suit;
        debug!("Trump suit is {}",self.trump);

        for player in &self.players { debug!("Player # {} has cards: {}",player.id,hand_fmt(&player.hand)); }

        self.to_play = 0;
        self.defender = 1;
        self.turn_type = GameTurnType::Attack;

        self.attackers_passed.clear();
        self.attackers = (0..self.players.len()).filter(|&ind| ind != self.defender).collect();

        Ok(())
    }

    // refills a players hand from the talon up to 6 cards
    fn refill_from_talon(&mut self, player_ind: usize) {
        while self.players[player_ind].hand.len() < 6 && self.draw_pile.len() > 0 {
            self.players[player_ind].hand.push(self.draw_pile.pop().unwrap());
        }
        sort_cards(&mut self.players[player_ind].hand,self.trump);
    }

    fn refill_players_hands(&mut self) {
        debug!("Refilling player's hands");
        let attackers = self.attackers.clone();
        for &ind in &attackers {
            self.refill_from_talon(ind);
        }
        self.refill_from_talon(self.defender);
    }


    async fn play_turn(&mut self, engines: &mut Vec<Box<dyn DurakPlayer>>) -> Result<()> {
        debug!("Taking turn");
        let to_play_state = gen_to_play_state(&self);
        for player in &self.players {
            debug!("Player # {} has cards: {}",player.id,player.hand.iter().map(|c| format!("{:>4}",format!("{}",c))).collect::<String>());
        }
        debug!("Player # {} is the attacker",self.players[self.attackers[0]].id);
        debug!("Player # {} is the defender",self.players[self.defender].id);
        debug!("Player # {} is playing",self.players[self.to_play].id);

        match &self.turn_type {
            GameTurnType::Attack => {
                debug!("Attack turn");
                let attack = {
                    if self.players[self.to_play].hand.len() == 0 { 
                        debug!("Skipping turn because player has no cards left");
                        Action::Pass
                    } else if self.players[self.defender].hand.len() == 0 {
                        debug!("Skipping turn because defender has no cards left");
                        Action::Pass
                    } else {
                        debug!("Querying player for attack");
                        let attack = engines[self.to_play].attack(&to_play_state).await?;
                        if attack == Action::Pass { debug!("Player has selected to pass"); }
                        attack
                    }
                };
                match attack {
                    Action::Play(attack_card) => {
                        debug!("Player has selected {}",attack_card);
                        to_play_state.validate_attack(&attack)?;
                        transfer_card(&mut self.players[self.to_play].hand,&mut self.attack_cards,&attack_card);
                        self.attackers_passed.clear();
                        self.last_attacker = self.to_play;
                        self.to_play = self.defender;
                        self.turn_type = GameTurnType::Defense;
                    },
                    Action::Pass => {
                        // bump to next attacker's turn, skipping those that have passed since last
                        // attack move
                        self.attackers_passed.push(self.to_play);
                        match self.attackers.iter().filter(|ind| !self.attackers_passed.contains(ind) && self.players[**ind].hand.len() != 0).next() {
                            Some(&ind) => self.to_play = ind,
                            None => {
                                debug!("Ending round because all attackers passed");
                                self.to_play = self.defender;
                                self.turn_type = GameTurnType::EndRound;
                            }
                        }
                    },
                }
            },
            GameTurnType::Defense => {
                debug!("Defense turn");
                match engines[self.to_play].defend(&to_play_state).await? {
                    Action::Play(defense_card) => {
                        debug!("Player has selected {}",defense_card);
                        to_play_state.validate_defense(&Action::Play(defense_card))?;
                        transfer_card(&mut self.players[self.to_play].hand,&mut self.defense_cards,&defense_card);
                        if self.defense_cards.len() == 6 || self.players[self.to_play].hand.len() == 0 {
                            debug!("Ending round because attack has been successfully defended");
                            // defender has priority for next round
                            self.to_play = self.defender;
                            self.turn_type = GameTurnType::EndRound;
                        } else {
                            // last attacker has dibs on attacking next
                            self.to_play = self.last_attacker;
                            self.turn_type = GameTurnType::Attack;
                        }
                    },
                    Action::Pass => {
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
                    let pile_on_cards = engines[ind_pile].pile_on(&to_play_state).await?;
                    to_play_state.validate_pile_on(&pile_on_cards)?;
                    debug!("Player {} has piled on {}",self.players[ind_pile].id,hand_fmt(&pile_on_cards));

                    for card in pile_on_cards {
                        transfer_card(&mut self.players[ind_pile].hand,&mut self.attack_cards,&card);
                    }
                }
                // defender is not the first attacker for next round
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

                // select players with cards left
                self.attackers = (0..self.players.len()).filter(|ind| self.players[*ind].hand.len() != 0).collect();
                // find next in order
                while !self.attackers.contains(&self.to_play) {
                    self.to_play = (self.to_play + 1) % self.players.len();
                }
                // rotate so they're first in line
                let offset = self.attackers.iter().enumerate().filter(|(_,ind)| **ind == self.to_play).next().unwrap().0;
                self.attackers.rotate_left(offset);
                // second in line is defender
                self.defender = self.attackers[1];
                self.attackers.remove(1);
                self.turn_type = GameTurnType::Attack;
            },
            GameTurnType::GameEnd => {},
        }
        Ok(())
    }
}

fn gen_to_play_state(state: &GameState) -> ToPlayState {
    gen_to_play_state_w_hand(state,state.to_play)
}

fn gen_to_play_state_w_hand(state: &GameState, hand_ind: usize) -> ToPlayState {
    ToPlayState {
        attack_cards: Cow::Borrowed(&state.attack_cards),
        defense_cards: Cow::Borrowed(&state.defense_cards),
        hand: Cow::Borrowed(&state.players[hand_ind].hand),
        trump: state.trump,
        player_info: get_player_info(state),
        last_attacker: state.last_attacker,
        defender: state.defender,
        to_play: state.to_play,
    }
}

fn get_player_info(state: &GameState) -> Vec<PlayerInfo> {
        state.players
            .iter()
            .map(|player| { 
                PlayerInfo {
                    id: player.id,
                    hand_len: player.hand.len()
                }
            }).collect()
}

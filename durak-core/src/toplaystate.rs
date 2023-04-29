//! Limited game state and player information made available to players on their turn.

use std::borrow::Cow;

use serde::{Serialize,Deserialize};
use thiserror::Error;

use crate::prelude::*;

/// A struct containing the information about a player in the game that is available to the other
/// players in the game.
#[derive(Serialize,Deserialize,Copy,Clone)]
pub struct PlayerInfo {
    /// Players unique ID.
    pub id: u64,
    /// Number of cards in player's hand.
    pub hand_len: usize,
}

/// A struct containing the limited game state information available to players.
#[derive(Serialize,Deserialize,Clone)]
pub struct ToPlayState<'a> {
    /// Trump suit.
    pub trump: Suit,

    /// Played attack cards.
    pub attack_cards: Cow<'a,Vec<Card>>,

    /// Played defense cards.
    pub defense_cards: Cow<'a,Vec<Card>>,

    /// Player's current hand.
    pub hand: Cow<'a,Vec<Card>>,

    /// All player's info, including this player.
    pub player_info: Vec<PlayerInfo>,

    /// Index to `player_info` for whoever attacked last.
    pub last_attacker: usize,

    /// Index to `player_info` for defender for this round.
    pub defender: usize,

    /// Index to `player_info` for player whose turn it currently is. Will be this player unless
    /// passed to [`DurakPlayer::observe_move()`].
    pub to_play: usize,
}

// checks if defense beats attack
fn beats_card(defense: &Card, attack: &Card, trump: &Suit) -> bool {
    if defense.suit == *trump {
        if attack.suit == *trump {
            if attack.rank >= defense.rank { return false; }
        }
    } else {
        if attack.suit == *trump { return false; }
        if attack.suit != defense.suit { return false; }
        if attack.rank >= defense.rank { return false; }
    }
    true
}

/// An error type for player move validation
#[derive(Error,Debug)]
#[allow(missing_docs)]
pub enum ValidationError {
    #[error("This is the wrong turn type for the current player")]
    WrongTurnType,
    #[error("Card {0} is not in player's hand")]
    CardNotInHand(Card),
    #[error("Invalid attack move: card rank {:?} has not been played yet",.0.rank)]
    InvalidAttack(Card),
    #[error("Invalid defense move: defense card {0} does not beat attack card {1}")]
    InvalidDefense(Card,Card),
}

// validates moves
// used as error causing assertions in game code
// players can call before returning their move
// to ensure that only valid moves are submitted
impl ToPlayState<'_> {

    /// Converts all internal Cow's to Owned variant.
    pub fn to_static(&self) -> ToPlayState<'static> {
        ToPlayState {
            trump: self.trump,
            attack_cards: Cow::Owned(self.attack_cards.clone().into_owned()),
            defense_cards: Cow::Owned(self.defense_cards.clone().into_owned()),
            hand: Cow::Owned(self.hand.clone().into_owned()),
            player_info: self.player_info.clone(),
            last_attacker: self.last_attacker,
            defender: self.defender,
            to_play: self.to_play,
        }
    }

    /// Validates an attack move
    pub fn validate_attack(&self, action: &Action) -> Result<(),ValidationError> {
        if self.to_play == self.defender { return Err(ValidationError::WrongTurnType); }
        match action {
            Action::Play(attack_card) => {
                if !self.hand.contains(&attack_card) {
                    return Err(ValidationError::CardNotInHand(*attack_card));
                }
                if self.attack_cards.len() == 0 { return Ok(()); }
                for card in self.attack_cards.iter() {
                    if card.rank == attack_card.rank { return Ok(()); }
                }
                for card in self.defense_cards.iter() {
                    if card.rank == attack_card.rank { return Ok(()); }
                }
                return Err(ValidationError::InvalidAttack(*attack_card));
            },
            Action::Pass => {
            }
        }
        Ok(())
    }

    /// Validates a defend move
    pub fn validate_defense(&self, action: &Action) -> Result<(), ValidationError> {
        if self.to_play != self.defender { return Err(ValidationError::WrongTurnType); }
        match action {
            Action::Play(defense_card) => {
                if !self.hand.contains(&defense_card) {
                    return Err(ValidationError::CardNotInHand(*defense_card));
                }
                if let Some(attack_card) = self.attack_cards.last() {
                    if !beats_card(defense_card,attack_card,&self.trump) { 
                        return Err(ValidationError::InvalidDefense(*defense_card,*attack_card));
                    }
                } else {
                    return Err(ValidationError::WrongTurnType);
                }
            },
            Action::Pass => {
            },
        }
        Ok(())
    }

    /// Validates a pile on
    pub fn validate_pile_on(&self, cards: &[Card]) -> Result<(), ValidationError> {
        if self.to_play == self.defender { return Err(ValidationError::WrongTurnType); }
        for pile_on_card in cards {
            self.validate_pile_on_single(pile_on_card)?;
        }
        Ok(())
    }

    /// Validates a single card for pile on. Does not validate turn type.
    pub fn validate_pile_on_single(&self, pile_on_card: &Card) -> Result<(), ValidationError> {
        if !self.hand.contains(&pile_on_card) {
            return Err(ValidationError::CardNotInHand(*pile_on_card));
        }
        for card in self.attack_cards.iter() {
            if card.rank == pile_on_card.rank {
                return Ok(());
            }
        }
        for card in self.defense_cards.iter() {
            if card.rank == pile_on_card.rank {
                return Ok(());
            }
        }
        return Err(ValidationError::InvalidAttack(*pile_on_card));
    }
}

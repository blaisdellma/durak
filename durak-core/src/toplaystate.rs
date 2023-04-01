//! Limited game state and player information made available to players on their turn.

use std::borrow::Cow;

use anyhow::{anyhow,bail,Result};
use serde::{Serialize,Deserialize};

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
    pub fn validate_attack(&self, action: &Action) -> Result<()> {
        if self.to_play == self.defender { bail!("Attack Invalid: Defender's turn"); }
        match action {
            Action::Play(attack_card) => {
                if !self.hand.contains(&attack_card) {
                    bail!("Attack Invalid: Card not in player's hand");
                }
                if self.attack_cards.len() == 0 { return Ok(()); }
                for card in self.attack_cards.iter() {
                    if card.rank == attack_card.rank { return Ok(()); }
                }
                for card in self.defense_cards.iter() {
                    if card.rank == attack_card.rank { return Ok(()); }
                }
                bail!("Attack Invalid: Card rank not in play");
            },
            Action::Pass => {
            }
        }
        Ok(())
    }

    /// Validates a defend move
    pub fn validate_defense(&self, action: &Action) -> Result<()> {
        if self.to_play != self.defender { bail!("Defense Invalid: Not defender's turn"); }
        match action {
            Action::Play(defense_card) => {
                if !self.hand.contains(&defense_card) {
                    bail!("Defense Invalid: Card not in player's hand");
                }
                let attack_card = self.attack_cards.last().ok_or_else(|| anyhow!("Defense Invalid: No corresponding attack card"))?;
                if !beats_card(defense_card,attack_card,&self.trump) { 
                    bail!("Defense Invalid: Defense card not sufficient for attack");
                }
            },
            Action::Pass => {
            },
        }
        Ok(())
    }

    /// Validates a pile on
    pub fn validate_pile_on(&self, cards: &[Card]) -> Result<()> {
        if self.to_play == self.defender { bail!("Pile On Invalid: Not attackers' turn"); }
        for pile_on_card in cards {
            if !self.hand.contains(&pile_on_card) {
                bail!("Pile On Invalid: Card not in player's hand");
            }
            let mut notfound = true;
            for card in self.attack_cards.iter() {
                if card.rank == pile_on_card.rank {
                    notfound = false;
                    break;
                }
            }
            for card in self.defense_cards.iter() {
                if card.rank == pile_on_card.rank {
                    notfound = false;
                    break;
                }
            }
            if notfound { bail!("Pile On Invalid: Card rank not in play"); }
        }
        Ok(())
    }
}

use std::borrow::Cow;
use serde::{Serialize,Deserialize};
use crate::*;

// limited game state available to players on their turn
#[derive(Serialize,Deserialize)]
pub struct ToPlayState<'a> {
    pub trump: Suit,
    pub attack_cards: Cow<'a,Vec<Card>>,
    pub defense_cards: Cow<'a,Vec<Card>>,
    pub hand: Cow<'a,Vec<Card>>,
    pub player_hand_sizes: Vec<usize>,
    pub attacker: usize,
    pub defender: usize,
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
    pub fn validate_attack(&self, action: &Option<Card>) -> Result<(),Box<dyn std::error::Error>> {
        if self.to_play == self.defender { return Err("Attack Invalid: Defender's turn".into()); }
        match action {
            Some(attack_card) => {
                if !self.hand.contains(&attack_card) {
                    return Err("Attack Invalid: Card not in player's hand".into());
                }
                if self.attack_cards.len() == 0 { return Ok(()); }
                for card in self.attack_cards.iter() {
                    if card.rank == attack_card.rank { return Ok(()); }
                }
                for card in self.defense_cards.iter() {
                    if card.rank == attack_card.rank { return Ok(()); }
                }
                return Err("Attack Invalid: Card rank not in play".into());
            },
            None => {
            }
        }
        Ok(())
    }

    pub fn validate_defense(&self, action: &Option<Card>) -> Result<(),Box<dyn std::error::Error>> {
        if self.to_play != self.defender { return Err("Defense Invalid: Not defender's turn".into()); }
        match action {
            Some(defense_card) => {
                if !self.hand.contains(&defense_card) {
                    return Err("Defense Invalid: Card not in player's hand".into());
                }
                let attack_card = self.attack_cards.last().ok_or_else(|| "Defense Invalid: No corresponding attack card")?;
                if !beats_card(defense_card,attack_card,&self.trump) { 
                    return Err("Defense Invalid: Defense card not sufficient for attack".into());
                }
            },
            None => {
            },
        }
        Ok(())
    }

    pub fn validate_pile_on(&self, cards: &[Card]) -> Result<(),Box<dyn std::error::Error>> {
        if self.to_play == self.defender { return Err("Pile On Invalid: Not attackers' turn".into()); }
        for pile_on_card in cards {
            if !self.hand.contains(&pile_on_card) {
                return Err("Pile On Invalid: Card not in player's hand".into());
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
            if notfound { return Err("Pile On Invalid: Card rank not in play".into()); }
        }
        Ok(())
    }
}

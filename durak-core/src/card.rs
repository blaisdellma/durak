#![allow(dead_code)]
use serde::{Serialize,Deserialize};
use std::fmt;
use rand::Rng;

use crate::*;

#[derive(PartialEq,Copy,Clone,Serialize,Deserialize)]
pub enum Suit {
    Spades = 0,
    Diamonds = 1,
    Hearts = 2,
    Clubs = 3,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Suit::Spades => write!(f,"♠"),
            Suit::Diamonds => write!(f,"♦"),
            Suit::Hearts => write!(f,"♥"),
            Suit::Clubs => write!(f,"♣"),
        }
    }
}

impl TryFrom<usize> for Suit {
    type Error = &'static str;
    fn try_from(value: usize) -> Result<Self,Self::Error> {
        match value {
            0 => Ok(Suit::Spades),
            1 => Ok(Suit::Diamonds),
            2 => Ok(Suit::Hearts),
            3 => Ok(Suit::Clubs),
            _ => Err("Value out of range"),
        }
    }
}

#[derive(PartialEq,Copy,Clone,PartialOrd,Serialize,Deserialize)]
pub enum Rank {
    Ace = 9,
    King = 8,
    Queen = 7,
    Jack = 6,
    Ten = 5,
    Nine = 4,
    Eight = 3,
    Seven = 2,
    Six = 1,
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rank::Ace => write!(f,"A"),
            Rank::King => write!(f,"K"),
            Rank::Queen => write!(f,"Q"),
            Rank::Jack => write!(f,"J"),
            Rank::Ten => write!(f,"10"),
            Rank::Nine => write!(f,"9"),
            Rank::Eight => write!(f,"8"),
            Rank::Seven => write!(f,"7"),
            Rank::Six => write!(f,"6"),
        }
    }
}

impl TryFrom<usize> for Rank {
    type Error = &'static str;
    fn try_from(value: usize) -> Result<Self,Self::Error> {
        match value {
            1 => Ok(Rank::Six),
            2 => Ok(Rank::Seven),
            3 => Ok(Rank::Eight),
            4 => Ok(Rank::Nine),
            5 => Ok(Rank::Ten),
            6 => Ok(Rank::Jack),
            7 => Ok(Rank::Queen),
            8 => Ok(Rank::King),
            9 => Ok(Rank::Ace),
            _ => Err("Value out of range"),
        }
    }
}

#[derive(PartialEq,Copy,Clone,Serialize,Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}{}",self.rank,self.suit)
    }
}

impl TryFrom<usize> for Card {
    type Error = &'static str;
    fn try_from(value: usize) -> Result<Self,Self::Error> {
        if value < 36 {
            let r = value%9 + 1;
            let s = value/9;
            Ok(Card {
                rank: Rank::try_from(r)?,
                suit: Suit::try_from(s)?,
            })
        } else {
            Err("Value out of range")
        }
    }
}

impl TryFrom<Card> for usize {
    type Error = &'static str;
    fn try_from(card: Card) -> Result<Self,Self::Error> {
        Ok(card.rank as usize + card.suit as usize * 9 - 1)
    }
}

pub fn hand_fmt(hand: &Vec<Card>) -> String {
    hand.iter().map(|c| format!("{:>4}",format!("{}",c))).collect::<String>()
}

pub struct Deck<'a, R: Rng> {
    cards: Vec<Card>,
    rng: &'a mut R,
}

impl<'a, R: Rng> Deck<'a, R> {
    pub fn init(rng: &'a mut R) -> DurakResult<Self> {
        Ok(Deck {
            cards: (0..36).map(|i| Card::try_from(i)).collect::<Result<Vec<Card>,_>>()?,
            rng: rng,
        })
    }

    pub fn get_trump(&mut self) -> DurakResult<Suit> {
        Ok(match self.rng.gen_range(0..4usize) {
            0 => Suit::Hearts,
            1 => Suit::Clubs,
            2 => Suit::Diamonds,
            3 => Suit::Spades,
            _ => {
                return Err("RNG error".into());
            },
        })
    }

    pub fn deal_cards(&mut self, ncards: usize, hand: &mut Vec<Card>, trump: Suit) -> DurakResult<()> {
        if ncards > self.cards.len() { return Err("Not enough cards in deck".into()); }
        for _ in 0..ncards {
            let k : usize = self.rng.gen_range(0..self.cards.len());
            hand.push(self.cards[k]);
            self.cards.remove(k);
        }
        sort_cards(hand,trump);
        Ok(())
    }

    pub fn all_cards_left(self) -> Vec<Card> {
        self.cards
    }
}

pub fn transfer_card(v_from: &mut Vec<Card>, v_to: &mut Vec<Card>, card: &Card) {
    let mut ind = 0;
    while ind < v_from.len() && v_from[ind] != *card { ind += 1};
    if ind < v_from.len() {
        v_to.push(v_from.remove(ind));
    }
}

pub fn sort_cards(cards: &mut Vec<Card>, trump: Suit) {
    cards.sort_by_key(|&card| {
        let val = usize::try_from(card).unwrap();
        match card.suit {
            suit if suit == trump => val + 100,
            _ => val,
        }
    });
}

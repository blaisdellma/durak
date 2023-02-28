use std::io::Write;
use tracing::{warn};

use durak::{*,card::*,toplaystate::*};

fn print_cards(cards: &[Card], trump: Suit) {
    for c in cards {
        if c.suit == trump {
            print!("\x1b[31m{:>5}\x1b[0m",format!("{}",c));
        } else {
            print!("{:>5}",format!("{}",c));
        }
    }
}

fn print_card_stack(state: &ToPlayState) {
    let s = 5;

    for i in 0..state.player_hand_sizes.len() {
        if i == state.to_play { print!("\x1b[31m"); }
        print!("{}{:>s$}",format!("  {}   ",i+1),"");
        if i == state.to_play { print!("\x1b[0m"); }
    }
    println!("");

    for i in 0..state.player_hand_sizes.len() {
        if i == state.to_play { print!("\x1b[31m"); }
        print!("{}{:>s$}","┌──┐  ","");
        if i == state.to_play { print!("\x1b[0m"); }
    }
    println!("");

    for i in 0..state.player_hand_sizes.len() {
        if i == state.to_play { print!("\x1b[31m"); }
        print!("{}{:>s$}","│┌─┴┐ ","");
        if i == state.to_play { print!("\x1b[0m"); }
    }
    println!("");

    for i in 0..state.player_hand_sizes.len() {
        if i == state.to_play { print!("\x1b[31m"); }
        print!("{}{:>s$}","└┤┌─┴┐","");
        if i == state.to_play { print!("\x1b[0m"); }
    }
    println!("");

    for (i,n) in (&state.player_hand_sizes).iter().enumerate() {
        if i == state.to_play {
            print!("\x1b[31m");
            print!("{}{:>s$}",format!(" └┤{:>2}│",state.hand.len()),"");
            print!("\x1b[0m");
        } else {
            print!("{}{:>s$}",format!(" └┤{:>2}│",n),"");
        }
    }
    println!("");

    for i in 0..state.player_hand_sizes.len() {
        if i == state.to_play { print!("\x1b[31m"); }
        print!("{}{:>s$}","  └──┘","");
        if i == state.to_play { print!("\x1b[0m"); }
    }
    println!("");
}

pub struct TUIDurakPlayer {
    id: usize,
}

impl TUIDurakPlayer {
    pub fn new(id: usize) -> Self {
        TUIDurakPlayer { id }
    }

    fn display_game_state(&self, state: &ToPlayState) {

        print_card_stack(state);

        println!("");
        print!("A:  "); print_cards(state.attack_cards,state.trump); println!("");
        println!("");
        print!("D:  "); print_cards(state.defense_cards,state.trump); println!("");
        println!("");

        print_cards(state.hand,state.trump);

        println!("");
        for x in 0..state.hand.len() {
            print!("{:>5}", x+1);
        }
        print!("{:>5}", 0);
        println!("");
    }

    fn get_input(&self) -> Result<usize,Box<dyn std::error::Error>> {
        print!("Your move:  ");
        std::io::stdout().flush()?;

        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        let buf = buf.trim_end_matches(char::is_whitespace);
        buf.parse().map_err(|e| format!("{}",e).into())
    }
}

impl DurakPlayer for TUIDurakPlayer {
    fn attack(&self, state: &ToPlayState) -> Option<Card> {
        println!("Player ID: {}", self.id);
        println!("You are attacking");
        self.display_game_state(state);

        loop {
            match self.get_input() {
                Err(e) => { warn!("Input error: {}",e); },
                Ok(x) if x == 0 => { return None; },
                Ok(x) if x > state.hand.len() => { warn!("Input out of range"); },
                Ok(x) => {
                    match state.validate_attack(&Some(state.hand[x-1])) {
                        Ok(_) => return Some(state.hand[x-1]),
                        Err(_) => { warn!("Disallowed attack card"); },
                    }
                },
            }
        }

    }

    fn defend(&self, state: &ToPlayState) -> Option<Card> {
        println!("Player ID: {}", self.id);
        println!("You are defending");
        self.display_game_state(state);

        loop {
            match self.get_input() {
                Err(e) => { warn!("Input error: {}",e); },
                Ok(x) if x == 0 => { return None; },
                Ok(x) if x > state.hand.len() => { continue; }
                Ok(x) if state.validate_defense(&Some(state.hand[x-1])).is_ok() => { return Some(state.hand[x-1]); },
                _ => continue
            }
        }
    }

    fn pile_on(&self, state: &ToPlayState) -> Vec<Card> {
        println!("Player ID: {}", self.id);
        println!("You are piling on");
        self.display_game_state(state);
        let mut inds = std::collections::HashSet::new();
        loop {
            for i in 0..state.hand.len() {
                if inds.contains(&(i+1)) {
                    print!("{:>5}","^");
                } else {
                    print!("{:>5}","");
                }
            }
            println!("");
            match self.get_input() {
                Err(e) => { warn!("Input error: {}", e); },
                Ok(x) if x == 0 => {
                    let output: Vec<Card> = inds.iter().map(|x| state.hand[x - 1]).collect();
                    match state.validate_pile_on(&output) {
                        Ok(_) => return output,
                        Err(e) => { warn!("Validation error: {}", e); },
                    }
                },
                Ok(x) if x > state.hand.len() => { continue; }
                Ok(x) => {
                    if inds.contains(&x) {
                        inds.remove(&x);
                    } else {
                        inds.insert(x);
                    }
                },
            }
        }
    }

    fn won(&self) {
        println!("Congratulations, Player #{}\nYOU WON!!!", self.id);
    }

    fn lost(&self) {
        println!("I'm sorry, Player #{}\nYou lost.", self.id);
    }
}


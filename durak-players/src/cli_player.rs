use std::io::Write;

use anyhow::{anyhow,Result};
use async_trait::async_trait;
use tracing::warn;

use durak_core::prelude::*;

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

    let bold_to_play = |id, func: &dyn Fn() -> ()| {
        if id == state.player_info[state.to_play].id { print!("\x1b[31m"); }
        func();
        if id == state.player_info[state.to_play].id { print!("\x1b[0m"); }
    };

    for info in state.player_info.iter() {
        bold_to_play(info.id, &|| {
            print!("{}{:>s$}",format!("  {}   ",info.id),"");
        });
    }
    println!("");

    for info in state.player_info.iter() {
        bold_to_play(info.id, &|| {
            print!("{}{:>s$}","┌──┐  ","");
        });
    }
    println!("");

    for info in state.player_info.iter() {
        bold_to_play(info.id, &|| {
            print!("{}{:>s$}","│┌─┴┐ ","");
        });
    }
    println!("");

    for info in state.player_info.iter() {
        bold_to_play(info.id, &|| {
            print!("{}{:>s$}","└┤┌─┴┐","");
        });
    }
    println!("");

    for info in state.player_info.iter() {
        bold_to_play(info.id, &|| {
            print!("{}{:>s$}",format!(" └┤{:>2}│",info.hand_len),"");
        });
    }
    println!("");

    for info in state.player_info.iter() {
        bold_to_play(info.id, &|| {
            print!("{}{:>s$}","  └──┘","");
        });
    }
    println!("");
}

pub struct CliPlayer {
    id: u64,
}

impl CliPlayer {
    pub fn new(id: u64) -> Self {
        CliPlayer { id }
    }

    fn display_game_state(&self, state: &ToPlayState) {

        print_card_stack(state);

        println!("");
        print!("A:  "); print_cards(&state.attack_cards,state.trump); println!("");
        println!("");
        print!("D:  "); print_cards(&state.defense_cards,state.trump); println!("");
        println!("");

        print_cards(&state.hand,state.trump);

        println!("");
        for x in 0..state.hand.len() {
            print!("{:>5}", x+1);
        }
        print!("{:>5}", 0);
        println!("");
    }

    fn get_input<T: std::str::FromStr<Err=std::num::ParseIntError>>(&self) -> Result<T> {
        print!("Your move:  ");
        std::io::stdout().flush()?;

        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        let buf = buf.trim_end_matches(char::is_whitespace);
        buf.parse().map_err(|e| anyhow!("{:?}",e))
    }
}

#[async_trait]
impl DurakPlayer for CliPlayer {
    async fn attack(&mut self, state: &ToPlayState) -> Result<Action> {
        println!("Player ID: {}", self.id);
        println!("You are attacking");
        self.display_game_state(state);

        loop {
            match self.get_input::<usize>() {
                Err(e) => { warn!("Input error: {}",e); },
                Ok(x) if x == 0 => { return Ok(Action::Pass); },
                Ok(x) if x > state.hand.len() => { warn!("Input out of range"); },
                Ok(x) => {
                    match state.validate_attack(&Action::Play(state.hand[x-1])) {
                        Ok(_) => return Ok(Action::Play(state.hand[x-1])),
                        Err(_) => { warn!("Disallowed attack card"); },
                    }
                },
            }
        }

    }

    async fn defend(&mut self, state: &ToPlayState) -> Result<Action> {
        println!("Player ID: {}", self.id);
        println!("You are defending");
        self.display_game_state(state);

        loop {
            match self.get_input::<usize>() {
                Err(e) => { warn!("Input error: {}",e); },
                Ok(x) if x == 0 => { return Ok(Action::Pass); },
                Ok(x) if x > state.hand.len() => { continue; }
                Ok(x) if state.validate_defense(&Action::Play(state.hand[x-1])).is_ok() => { return Ok(Action::Play(state.hand[x-1])); },
                _ => continue
            }
        }
    }

    async fn pile_on(&mut self, state: &ToPlayState) -> Result<Vec<Card>> {
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
            match self.get_input::<usize>() {
                Err(e) => { warn!("Input error: {}", e); },
                Ok(x) if x == 0 => {
                    let output: Vec<Card> = inds.iter().map(|x| state.hand[x - 1]).collect();
                    match state.validate_pile_on(&output) {
                        Ok(_) => return Ok(output),
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

    async fn observe_move(&mut self, state: &ToPlayState) -> Result<()> {
        println!("Player ID: {}", self.id);
        self.display_game_state(state);
        Ok(())
    }

    async fn won(&mut self) -> Result<Ready> {
        println!("Congratulations, Player #{}\nYOU WON!!!", self.id);
        Ok(Ready::Yes)
    }

    async fn lost(&mut self) -> Result<Ready> {
        println!("I'm sorry, Player #{}\nYou lost.", self.id);
        Ok(Ready::Yes)
    }

    async fn message(&mut self, msg: &str) -> Result<()> {
        println!("Message from game engine: {}",msg);
        Ok(())
    }

    async fn error(&mut self, error: &str) -> Result<()> {
        println!("I'm sorry, there was an error.");
        println!("Error: {}",error);
        println!("The game is over now.");
        Ok(())
    }

    async fn get_id(&mut self,player_info: &Vec<PlayerInfo>) -> Result<u64> {
        println!("Player List:");
        for info in player_info {
            println!("Player {}",info.id);
        }
        loop {
            match self.get_input() {
                Err(e) => { warn!("Input error: {}",e); },
                Ok(x) => {
                    if !player_info.iter().map(|info| info.id).collect::<Vec<_>>().contains(&x) {
                        self.id = x;
                        break;
                    } else {
                        println!("That ID is already taken.");
                    }
                },
            }
        }
        Ok(self.id)
    }
}


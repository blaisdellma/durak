use tracing::{warn};
use cursive::theme::{Style,ColorStyle,ColorType,PaletteColor};
use cursive::traits::{Resizable,Nameable};
use cursive::views::{DummyView,TextView,LinearLayout,Dialog,PaddedView,ResizedView,Button,EditView};

use durak_core::*;

pub struct TUINewDurakPlayer {
    id: u64,
}

impl TUINewDurakPlayer {
    pub fn new() -> Self {
        TUINewDurakPlayer { id: 0 }
    }

    fn display_oppo(&self, n: usize) -> ResizedView<TextView> {
        ResizedView::with_fixed_size((6,5), {
            TextView::new(format!("{}\n{}\n{}\n{}{:>2}{}\n{}",
            "┌──┐  ",
            "│┌─┴┐ ",
            "└┤┌─┴┐",
            " └┤",n,"│",
            "  └──┘"))
        })
    }

    fn display_oppo_hands(&self, state: &ToPlayState) -> LinearLayout {
        let mut layout = LinearLayout::horizontal();
        for info in &state.player_info {
            layout.add_child(PaddedView::lrtb(3,3,0,1,LinearLayout::vertical()
                             .child(self.display_oppo(info.hand_len))
                             .child(TextView::new({
                                 if info.id == self.id {
                                     format!("You")
                                 } else {
                                     format!("Player # {}", info.id)
                                 }
                             }))
                             ));
        }
        layout
    }

    fn display_card_list(&self, label: &str, list: &Vec<Card>, trump: Suit) -> LinearLayout {
        let mut layout = LinearLayout::horizontal();
        layout.add_child(TextView::new(label));
        for card in list.iter() {
            if card.suit == trump {
                layout.add_child(TextView::new(format!(" {:>4} ",card)).style({
                        let mut style = Style::none();
                        style.color = ColorStyle::front(ColorType::Palette(PaletteColor::Highlight));
                        style
                }));
            } else {
                layout.add_child(TextView::new(format!(" {:>4} ",card)));
            }
        }
        layout
    }

    fn display_state(&self, state: &ToPlayState) -> LinearLayout {
        let mut layout = LinearLayout::vertical();
        layout.add_child(self.display_oppo_hands(state));
        layout.add_child(self.display_card_list("A: ",&state.attack_cards, state.trump));
        layout.add_child(self.display_card_list("D: ",&state.defense_cards, state.trump));
        layout
    }

    fn display_hand_choice(&self, state: &ToPlayState) -> Dialog {
        let mut dialog = Dialog::new().title("Your move:");
        for &card in state.hand.iter() {
            dialog.add_button(format!("{}",card), move |s| {
                s.set_user_data(DurakResult::<Option<Card>>::Ok(Some(card)));
                s.quit();
            });
        }
        dialog.add_button("Pass", move |s| {
            s.set_user_data(DurakResult::<Option<Card>>::Ok(None));
            s.quit();
        });
        dialog
    }

    fn display_hand_info(&self, state: &ToPlayState) -> Dialog {
        let mut dialog = {
            if state.player_info[state.to_play].id == self.id {
                Dialog::new().title("Your turn")
            } else {
                Dialog::new().title(format!("Player {} turn",state.player_info[state.to_play].id))
            }
        };
        for &card in state.hand.iter() {
            dialog.add_button(format!("{}",card), move |s| {
                s.set_user_data(DurakResult::<Option<Card>>::Ok(Some(card)));
                s.quit();
            });
        }
        dialog.add_button("Pass", move |s| {
            s.set_user_data(DurakResult::<Option<Card>>::Ok(None));
            s.quit();
        });
        for button in dialog.buttons_mut() {
            button.disable();
        }
        dialog
    }

    fn play_single_card<F: Fn(&Option<Card>)->DurakResult<()>>(&self, state: &ToPlayState, validate: F) -> DurakResult<Option<Card>> {
        loop {
            let mut siv = cursive::default();
            siv.set_user_data(DurakResult::<Option<Card>>::Ok(None));
            siv.add_global_callback('q', |s| {
                s.set_user_data(DurakResult::<Option<Card>>::Err("Player quit.".into()));
                s.quit();
            });

            siv.add_layer(LinearLayout::vertical()
                          .child(self.display_state(state))
                          .child(DummyView)
                          .child(TextView::new("Your Turn:"))
                          .child(self.display_hand_choice(state)));

            siv.run();

            match siv.take_user_data::<DurakResult<Option<Card>>>() {
                Some(Ok(Some(ret))) => {
                    match validate(&Some(ret)) {
                        Ok(_) => return Ok(Some(ret)),
                        Err(_) => warn!("Disallowed card"),
                    }
                },
                Some(Ok(None)) => return Ok(None),
                Some(Err(e)) => return Err(e),
                None => {},
            }
        }
    }
}

impl DurakPlayer for TUINewDurakPlayer {
    fn attack(&mut self, state: &ToPlayState) -> DurakResult<Option<Card>> {
        self.play_single_card(state,|x| state.validate_attack(x))
    }

    fn defend(&mut self, state: &ToPlayState) -> DurakResult<Option<Card>> {
        self.play_single_card(state,|x| state.validate_defense(x))
    }

    fn pile_on(&mut self, _state: &ToPlayState) -> DurakResult<Vec<Card>> {
        Ok(Vec::new())
        // println!("Player ID: {}", self.id);
        // println!("You are piling on");
        // self.display_game_state(state);
        // let mut inds = std::collections::HashSet::new();
        // loop {
        //     for i in 0..state.hand.len() {
        //         if inds.contains(&(i+1)) {
        //             print!("{:>5}","^");
        //         } else {
        //             print!("{:>5}","");
        //         }
        //     }
        //     println!("");
        //     match self.get_input() {
        //         Err(e) => { warn!("Input error: {}", e); },
        //         Ok(x) if x == 0 => {
        //             let output: Vec<Card> = inds.iter().map(|x| state.hand[x - 1]).collect();
        //             match state.validate_pile_on(&output) {
        //                 Ok(_) => return Ok(output),
        //                 Err(e) => { warn!("Validation error: {}", e); },
        //             }
        //         },
        //         Ok(x) if x > state.hand.len() => { continue; }
        //         Ok(x) => {
        //             if inds.contains(&x) {
        //                 inds.remove(&x);
        //             } else {
        //                 inds.insert(x);
        //             }
        //         },
        //     }
        // }
    }

    fn observe_move(&mut self, state: &ToPlayState) -> DurakResult<()> {
        let mut siv = cursive::default();
        siv.add_global_callback('q', |s| {
            s.set_user_data(DurakResult::<()>::Err("Player quit.".into()));
            s.quit();
        });

        siv.add_layer(LinearLayout::vertical()
                      .child(self.display_state(state))
                      .child(DummyView)
                      .child(self.display_hand_info(state))
                      .child(Button::new("Continue", |s| s.quit()).with_name("button")));

        siv.focus_name("button").unwrap();
        siv.run();
        match siv.take_user_data() {
            Some(x) => x,
            None => Ok(())
        }
    }

    fn won(&mut self) -> DurakResult<()> {
        let mut siv = cursive::default();
        siv.add_global_callback('q', |s| {
            s.quit();
        });

        siv.add_layer(PaddedView::lrtb(5,5,2,2,LinearLayout::vertical()
                      .child(TextView::new("Congratulations!"))
                      .child(TextView::new("   YOU WON!!!"))
                      .child(DummyView)
                      .child(Button::new("Exit", |s| s.quit()))));

        siv.run();
        println!("Congratulations, Player #{}\nYOU WON!!!", self.id);
        Ok(())
    }

    fn lost(&mut self) -> DurakResult<()> {
        let mut siv = cursive::default();
        siv.add_global_callback('q', |s| {
            s.quit();
        });

        siv.add_layer(PaddedView::lrtb(5,5,2,2,LinearLayout::vertical()
                      .child(TextView::new(" Sorry."))
                      .child(TextView::new("You lost."))
                      .child(DummyView)
                      .child(Button::new("Exit", |s| s.quit()))));

        siv.run();
        println!("I'm sorry, Player #{}\nYou lost.", self.id);
        Ok(())
    }

    fn get_id(&mut self, player_info: &Vec<PlayerInfo>) -> DurakResult<u64> {
        loop {
            let mut siv = cursive::default();
            siv.add_global_callback('q', |s| {
                s.quit();
            });
            siv.add_layer(PaddedView::lrtb(5,5,2,2, {
                let mut layout = LinearLayout::vertical();
                layout.add_child(TextView::new("Player List:"));
                for info in player_info {
                    layout.add_child(TextView::new(format!("Player {}",info.id)));
                }
                layout.add_child(DummyView);
                layout.add_child(TextView::new("Enter Player ID"));
                layout.add_child(Dialog::new().content(EditView::new().on_submit(|s, id_buf| {
                    s.set_user_data(id_buf.to_owned());
                    s.quit();
                }).fixed_width(20).with_name("id")).button("Submit", |s| {
                    let id_buf = s.call_on_name("id", | view: &mut EditView | {
                        view.get_content();
                    }).unwrap();
                    s.set_user_data(id_buf.to_owned());
                    s.quit();
                }));
                layout
            }));
            siv.run();
            match siv.take_user_data::<String>() {
                Some(id_buf) => {
                    match id_buf.parse::<u64>() {
                        Ok(id) => {
                            let mut notfound = true;
                            for info in player_info {
                                if info.id == id {
                                    notfound = false;
                                    break;
                                }
                            }
                            if notfound {
                                self.id = id;
                                return Ok(self.id);
                            }
                        },
                        Err(_) => {},
                    }
                },
                None => {},
            }
        }
    }
}


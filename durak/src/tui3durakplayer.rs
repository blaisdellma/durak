use std::any::Any;

use tracing::{debug,error};

use crossbeam_channel::unbounded;

use cursive::{Cursive,CbSink,CursiveRunnable};
use cursive::reexports::crossbeam_channel::{Sender,Receiver,bounded};
use cursive::theme::{Style,ColorStyle,ColorType,PaletteColor};
use cursive::utils::markup::StyledString;
use cursive::traits::{Resizable,Nameable};
use cursive::views::{HideableView,DummyView,TextView,LinearLayout,Dialog,PaddedView,ResizedView,EditView,DialogFocus};

use durak_core::{*,card::*,toplaystate::*};

pub struct TUISuperNewDurakPlayer {
    id: u64,
    tui: CbSink,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl TUISuperNewDurakPlayer {
    pub fn new() -> Self {
        let (sender, receiver) = bounded::<CbSink>(0);
        let handle = std::thread::spawn(|| {
            let mut siv = cursive::default();
            setup(&mut siv);
            siv.add_layer({
                HideableView::new({
                    TextView::new("Press <Enter> to start game")
                }).with_name("start")
            });
            siv.add_global_callback(cursive::event::Key::Enter, move |s: &mut Cursive| {
                s.clear_global_callbacks(cursive::event::Key::Enter);
                s.pop_layer();
                // s.call_on_name("start", |view: &mut HideableView<TextView>| view.hide());
                // s.call_on_name("main", |view: &mut HideableView<LinearLayout>| view.unhide());
                s.add_global_callback('q', |ss| {
                    ss.quit();
                });
                sender.send(s.cb_sink().clone()).unwrap();
            });
            siv.run();
        });
        let tui = receiver.recv().unwrap();
        TUISuperNewDurakPlayer {
            id: 0,
            tui: tui,
            handle: Some(handle),
        }
    }

    fn test_recv<T>(&mut self, receiver: Receiver<T>) -> DurakResult<T> {
        debug!("test recv");
        let thing = match receiver.recv() {
            Ok(x) => x,
            Err(e) => {
                error!("receiver error");
                self.tui.send_timeout(Box::new(move |s| {
                    s.add_layer(PaddedView::lrtb(10,10,4,4,TextView::new("Exiting due to recv error")));
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                    s.pop_layer();
                    s.quit();
                }),std::time::Duration::from_millis(10000)).unwrap();
                eprintln!("ERROR!");
                return Err(format!("Receiver error: {}" ,e).into());
            },
        };
        Ok(thing)
    }

    fn end(&mut self) -> DurakResult<()> {
        self.tui.send(Box::new(|s: &mut Cursive| {
            s.quit();
        })).unwrap();
        match self.handle.take() {
            Some(h) => h.join().map_err(|_| "Join Error".into()),
            None => Ok(()),
        }
    }

}

impl DurakPlayer for TUISuperNewDurakPlayer {
    fn attack(&mut self, state: &ToPlayState) -> DurakResult<Option<Card>> {
        let (sender,receiver) = bounded::<Option<Card>>(0);
        let id = self.id;
        let static_state = state.to_static();
        self.tui.send(Box::new(move |s| {
            update_game_state(s,&static_state,id,sender);
        })).unwrap();
        loop {
            debug!("loop");
            match self.test_recv(receiver.clone()) {
                Ok(Some(card)) => {
                    debug!("Received card");
                    match state.validate_attack(&Some(card)) {
                        Ok(_) => return Ok(Some(card)),
                        Err(_) => {},
                    }
                },
                Ok(None) => { return Ok(None); },
                Err(e) => { return Err(e); },
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    fn defend(&mut self, state: &ToPlayState) -> DurakResult<Option<Card>> {
        let (sender,receiver) = bounded::<Option<Card>>(0);
        let id = self.id;
        let static_state = state.to_static();
        self.tui.send(Box::new(move |s| {
            update_game_state(s,&static_state,id,sender);
        })).unwrap();
        loop {
            match self.test_recv(receiver.clone()) {
                Ok(Some(card)) => {
                    match state.validate_defense(&Some(card)) {
                        Ok(_) => return Ok(Some(card)),
                        Err(_) => {},
                    }
                },
                Ok(None) => { return Ok(None); },
                Err(e) => { return Err(e); },
            }
        }
    }

    fn pile_on(&mut self, _state: &ToPlayState) -> DurakResult<Vec<Card>> {
        Ok(Vec::new())
    }

    fn observe_move(&mut self, state: &ToPlayState) -> DurakResult<()> {
        let (sender,_receiver) = unbounded::<()>();
        let id = self.id;
        let static_state = state.to_static();
        self.tui.send(Box::new(move |s| {
            update_game_state(s,&static_state,id,sender);
        })).unwrap();
        Ok(())
    }

    fn get_id(&mut self, _player_info: &Vec<PlayerInfo>) -> DurakResult<u64> {
        let player_info = _player_info.clone();
        let (sender,receiver) = bounded::<u64>(0);
        self.tui.send(Box::new(move |s| {
            s.call_on_name("get_id", move |hideable: &mut HideableView<PaddedView<LinearLayout>>| {
                let layout = hideable.get_inner_mut().get_inner_mut();
                layout.clear();
                layout.add_child(TextView::new("Player List:"));
                for info in &player_info {
                    layout.add_child(TextView::new(format!("Player {}",info.id)));
                }
                layout.add_child(DummyView);
                layout.add_child(TextView::new("Enter Player ID"));
                layout.add_child(Dialog::new().content(EditView::new().on_submit(move |s, id_buf| {
                    match id_buf.parse::<u64>() {
                        Ok(id) => {
                            if !player_info.iter().any(|info| info.id == id) {
                                sender.clone().send(id).unwrap();
                                s.call_on_name("get_id", |hideable: &mut HideableView<PaddedView<LinearLayout>>| {
                                    hideable.hide();
                                    hideable.get_inner_mut().get_inner_mut().clear();
                                });
                                s.pop_layer();
                            }
                        },
                        Err(_) => {},
                    }
                }).fixed_width(20).with_name("id")));
                hideable.unhide();
            });
            s.focus_name("id").unwrap();
        })).unwrap();

        let id = self.test_recv(receiver)?;
        self.id = id;
        Ok(id)
    }

    fn won(&mut self) -> DurakResult<()> {
        let (sender,receiver) = bounded::<()>(0);
        self.tui.send(Box::new(|s: &mut Cursive| {
            s.call_on_name("main", | hideable: &mut HideableView<LinearLayout> | {
                hideable.hide();
            });
            s.call_on_name("won", | hideable: &mut HideableView<PaddedView<LinearLayout>> | {
                hideable.unhide();
            });
            s.add_global_callback(cursive::event::Key::Enter, move |_s: &mut Cursive| {
                sender.send(()).unwrap();
            });
        })).unwrap();
        self.test_recv(receiver)?;
        self.end()?;
        println!("Congratulations, Player #{}\nYOU WON!!!", self.id);
        Ok(())
    }

    fn lost(&mut self) -> DurakResult<()> {
        let (sender,receiver) = bounded::<()>(0);
        self.tui.send(Box::new(|s: &mut Cursive| {
            s.call_on_name("main", | hideable: &mut HideableView<LinearLayout> | {
                hideable.hide();
            });
            s.call_on_name("lost", | hideable: &mut HideableView<PaddedView<LinearLayout>> | {
                hideable.unhide();
            });
            s.add_global_callback(cursive::event::Key::Enter, move |_s: &mut Cursive| {
                sender.send(()).unwrap();
            });
        })).unwrap();
        self.test_recv(receiver)?;
        self.end()?;
        println!("I'm sorry, Player #{}\nYou lost.", self.id);
        Ok(())
    }
}

fn update_game_state<T: Any>(siv: &mut Cursive, state: &ToPlayState, id: u64, sender: Sender<T>) {
    siv.call_on_name("player_info", |layout: &mut LinearLayout| {
        layout.clear();
        for info in state.player_info.iter() {
            layout.add_child(create_player_info(info,id));
        }
    });
    siv.call_on_name("attack_cards", |layout: &mut LinearLayout| {
        layout.clear();
        layout.add_child(TextView::new("A: "));
        for &card in state.attack_cards.iter() {
            layout.add_child(create_card_view(card,state.trump));
        }
    });
    siv.call_on_name("trump_msg", |text: &mut TextView| {
        text.set_content(format!("Trump Suit: {}",state.trump));
    });
    siv.call_on_name("defense_cards", |layout: &mut LinearLayout| {
        layout.clear();
        layout.add_child(TextView::new("D: "));
        for &card in state.defense_cards.iter() {
            layout.add_child(create_card_view(card,state.trump));
        }
    });
    siv.call_on_name("hand_dialog", |dialog: &mut Dialog| {
        dialog.clear_buttons();
        if state.player_info[state.to_play].id == id {
            dialog.set_title("Your Turn");
        } else {
            dialog.set_title(format!("Player {} turn",state.player_info[state.to_play].id));
        }
        for &card in state.hand.iter() {
            let sender2 = sender.clone();
            dialog.add_button(create_card_label(card,state.trump), move |_s| {
                let sender2 = sender2.clone();
                match (&sender2 as &dyn Any).downcast_ref::<Sender<Option<Card>>>() {
                    Some(x) => x.send(Some(card)).unwrap(),
                    None => {},
                }
            });
        }
        dialog.add_button("Pass", move |_s| {
            let sender = sender.clone();
            match (&sender as &dyn Any).downcast_ref::<Sender<Option<Card>>>() {
                Some(x) => x.send(None).unwrap(),
                None => {},
            }
        });
    });
    siv.call_on_name("main", |view: &mut HideableView<LinearLayout>| view.unhide());
    siv.focus_name("hand_dialog").unwrap();
    siv.call_on_name("hand_dialog", |dialog: &mut Dialog| {
        dialog.set_focus(DialogFocus::Button(0));
    });
}

fn setup(siv: &mut CursiveRunnable) {
    setup_msg(siv,vec!["Congratulations!","YOU WON!!!"],"won");
    setup_msg(siv,vec!["Sorry","You lost"],"lost");
    setup_scaffold(siv);
    setup_id(siv);
}

fn setup_msg(siv: &mut CursiveRunnable, messages: Vec<&str>, name: &str) {
    siv.add_layer(HideableView::new(PaddedView::lrtb(5,5,2,2, {
        let mut layout = LinearLayout::vertical();
        for msg in messages {
            layout.add_child(TextView::new(msg));
        }
        layout
    })).hidden().with_name(name));
}

fn setup_scaffold(siv: &mut CursiveRunnable) {
    let hand_dialog = Dialog::new().with_name("hand_dialog");
    let attack_cards = LinearLayout::horizontal().with_name("attack_cards");
    let defense_cards = LinearLayout::horizontal().with_name("defense_cards");
    let trump_msg = TextView::new("").with_name("trump_msg");
    let player_info = LinearLayout::horizontal().with_name("player_info");

    siv.add_layer(HideableView::new({
        LinearLayout::vertical()
            .child(player_info)
            .child(trump_msg)
            .child(attack_cards)
            .child(defense_cards)
            .child(hand_dialog)
    }).hidden().with_name("main"));
}

fn setup_id(siv: &mut CursiveRunnable) {
    siv.add_layer(HideableView::new({
        PaddedView::lrtb(5,5,2,2,{
            LinearLayout::vertical()
        })
    }).hidden().with_name("get_id"));
}

fn create_player_info(info: &PlayerInfo, id: u64) -> PaddedView<LinearLayout> {
    let label = TextView::new({
        if info.id == id {
            format!("You")
        } else {
            format!("Player # {}", info.id)
        }
    });
    PaddedView::lrtb(3,3,0,1,LinearLayout::vertical()
                     .child(display_oppo(info.hand_len))
                     .child(label))
}

fn create_card_view(card: Card, trump: Suit) -> TextView {
    // // if card.suit == trump {
    // //     TextView::new(format!(" {:>4} ",card)).style({
    // //             let mut style = Style::none();
    // //             style.color = ColorStyle::front(ColorType::Palette(PaletteColor::Highlight));
    // //             style
    // //     })
    // // } else {
    // //     TextView::new(format!(" {:>4} ",card))
    // // }
    // TextView::new(format!(" {:>4} ",card))
    TextView::new(create_card_label(card,trump))
}

fn create_card_label(card: Card, trump: Suit) -> StyledString {
    StyledString::styled(format!(" {:>4} ",card),{
        let mut style = Style::none();
        if card.suit == trump {
            style.color = ColorStyle::front(ColorType::Palette(PaletteColor::Highlight));
        } else {
        }
        style
    })
}

fn display_oppo(n: usize) -> ResizedView<TextView> {
    ResizedView::with_fixed_size((6,5), {
        TextView::new(format!("{}\n{}\n{}\n{}{:>2}{}\n{}",
        "┌──┐  ",
        "│┌─┴┐ ",
        "└┤┌─┴┐",
        " └┤",n,"│",
        "  └──┘"))
    })
}

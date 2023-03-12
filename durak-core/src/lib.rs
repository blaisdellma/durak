pub type DurakResult<T> = Result<T,Box<dyn std::error::Error>>;

pub mod game;
pub mod card;
pub mod toplaystate;

pub use game::{DurakPlayer, DurakGame};
pub use card::{Card, Suit, Rank};
use card::{transfer_card, hand_fmt, sort_cards, Deck};
pub use toplaystate::{PlayerInfo, ToPlayState};

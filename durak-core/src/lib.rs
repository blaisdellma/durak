//! 
//! [Durak] is a card game. The name comes from the Russian
//! word for fool. There are lots of variations to durak and this only (currently) an
//! implementation of the most basic game. This crate provides a struct ([`DurakGame`]) which
//! implements the game engine and a trait ([`DurakPlayer`]) which serves as the interface for
//! players. For some implementations of [`DurakPlayer`] see the associated durak crate.
//!
//! [Durak]: https://en.wikipedia.org/wiki/Durak
//! [`DurakPlayer`]: game/trait.durakplayer.html
//! [`DurakGame`]: game/struct.durakgame.html
//!

#![warn(missing_docs)]

/// A basic result type used throughout this crate
pub type DurakResult<T> = Result<T,Box<dyn std::error::Error>>;

pub mod game;
pub mod card;
pub mod toplaystate;
pub mod prelude;


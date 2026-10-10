//! Coffee Maker: a tiny 2-word parser game, ported from Ada.
//!
//! This version keeps the original's design: one `Thing` enum covering
//! rooms, objects, verbs and sentinels, and global tables indexed by it.

mod game;
mod parser;
mod tables;
mod text;
mod thing;

use std::time::Duration;

pub use game::Game;

/// Paces the output, as the original did, so text doesn't arrive all at
/// once.
pub trait Pacer {
    fn pause(&mut self, d: Duration);
}

/// Really sleeps. Used when playing.
pub struct Sleep;

impl Pacer for Sleep {
    fn pause(&mut self, d: Duration) {
        std::thread::sleep(d);
    }
}

/// Doesn't pause at all. Used by tests.
pub struct NoPause;

impl Pacer for NoPause {
    fn pause(&mut self, _: Duration) {}
}

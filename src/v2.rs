//! Version 2: with generics. Read `v1.rs` first.
//!
//! In v1 the FSM mechanism (`state`, `dispatch`, `tran`) is written inside
//! `Keyboard`, so a second machine would have to copy it. Here the mechanism
//! moves to `fsm.rs` and is written once for any machine, like the C header
//! in Listing 1. `keyboard.rs` then only contains the keyboard itself.

pub mod fsm;
pub mod keyboard;

use fsm::Machine;
use keyboard::{KeyEvent, Keyboard};

pub fn run() {
    let mut keyboard = Keyboard::new();

    // `initial` ignores its event, so it does not matter which one we pass.
    keyboard.init(&KeyEvent::ShiftReleased);

    let events = [
        KeyEvent::AnyKey('h'),
        KeyEvent::AnyKey('i'),
        KeyEvent::ShiftDepressed,
        KeyEvent::AnyKey('r'),
        KeyEvent::AnyKey('u'),
        KeyEvent::ShiftReleased,
        KeyEvent::AnyKey('s'),
        KeyEvent::AnyKey('t'),
    ];

    for event in &events {
        keyboard.dispatch(event);
    }

    println!("typed: {}", keyboard.typed);
}

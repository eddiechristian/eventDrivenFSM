//! The keyboard, built on `fsm.rs`. Same machine as v1:
//!
//! ```text
//!        (initial)
//!            |
//!            v        ShiftDepressed
//!      +-----------+ ---------------> +-----------+
//!      |  default  |                  |  shifted  |
//!      +-----------+ <--------------- +-----------+
//!        AnyKey:       ShiftReleased    AnyKey:
//!        lower-case                     upper-case
//! ```
//!
//! Compared with v1, `dispatch` and `tran` are gone from this file: they come
//! from the `Machine` trait. The state functions are unchanged.

use super::fsm::{Fsm, Machine};

/// The things that can happen to the keyboard.
pub enum KeyEvent {
    ShiftDepressed,
    ShiftReleased,
    AnyKey(char),
}

/// C: `struct Keyboard { Fsm super_; ... }`
pub struct Keyboard {
    fsm: Fsm<Keyboard, KeyEvent>,
    /// Everything typed so far (ordinary data the machine carries along).
    pub typed: String,
}

/// Plug the keyboard into the reusable mechanism.
impl Machine for Keyboard {
    type Event = KeyEvent;

    fn fsm(&mut self) -> &mut Fsm<Keyboard, KeyEvent> {
        &mut self.fsm
    }
}

impl Keyboard {
    pub fn new() -> Keyboard {
        Keyboard {
            fsm: Fsm::new(Keyboard::initial),
            typed: String::new(),
        }
    }

    // ---- the states: one function per state -------------------------------

    /// Not a real state: it only exists to get us into `default`.
    fn initial(&mut self, _event: &KeyEvent) {
        println!("[initial] keyboard initialized -> default");
        self.tran(Keyboard::default);
    }

    fn default(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::ShiftDepressed => {
                println!("[default] shift depressed -> shifted");
                self.tran(Keyboard::shifted);
            }
            KeyEvent::AnyKey(key) => {
                let key = key.to_ascii_lowercase();
                println!("[default] key '{key}'");
                self.typed.push(key);
            }
            KeyEvent::ShiftReleased => {} // ignored in this state
        }
    }

    fn shifted(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::ShiftReleased => {
                println!("[shifted] shift released -> default");
                self.tran(Keyboard::default);
            }
            KeyEvent::AnyKey(key) => {
                let key = key.to_ascii_uppercase();
                println!("[shifted] key '{key}'");
                self.typed.push(key);
            }
            KeyEvent::ShiftDepressed => {} // ignored in this state
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_changes_how_a_key_is_handled() {
        let mut keyboard = Keyboard::new();
        keyboard.init(&KeyEvent::ShiftReleased);

        keyboard.dispatch(&KeyEvent::AnyKey('a'));
        keyboard.dispatch(&KeyEvent::ShiftDepressed);
        keyboard.dispatch(&KeyEvent::AnyKey('b'));
        keyboard.dispatch(&KeyEvent::ShiftReleased);
        keyboard.dispatch(&KeyEvent::AnyKey('c'));

        assert_eq!(keyboard.typed, "aBc");
    }
}

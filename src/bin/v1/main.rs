//! Version 1: no generics. Everything in one file, read it top to bottom.
//!
//! The one idea from Listing 1:
//!
//!   The current state is a FUNCTION POINTER.
//!   - dispatch an event  = call the function the pointer holds
//!   - make a transition  = store a different function in the pointer
//!
//! The machine: a keyboard with a Shift key.
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

// ---------------------------------------------------------------------------
// Part 1: what Listing 1 defines
// ---------------------------------------------------------------------------

/// The things that can happen to the keyboard.
///
/// C: `struct Event { Signal sig; }`
pub enum Event {
    ShiftDepressed,
    ShiftReleased,
    AnyKey(char),
}

/// A state is a function: "here is the keyboard and an event, react to it".
///
/// C: `typedef void (*State)(Fsm *, Event const *);`
type State = fn(&mut Keyboard, &Event);

/// The state machine. `state` is the only field the FSM mechanism needs.
///
/// C: `struct Fsm { State state__; }`
pub struct Keyboard {
    state: State,
    /// Everything typed so far (ordinary data the machine carries along).
    pub typed: String,
}

impl Keyboard {
    /// Start out pointing at the `initial` function. Nothing runs yet.
    ///
    /// C: `FsmCtor_(me, init)`
    pub fn new() -> Keyboard {
        Keyboard {
            state: Keyboard::initial,
            typed: String::new(),
        }
    }

    /// Run the `initial` function once, which moves us into `default`.
    ///
    /// C: `FsmInit(me, e)` - in the C listing it is the same code as dispatch.
    pub fn init(&mut self, event: &Event) {
        self.dispatch(event);
    }

    /// Give an event to the current state.
    ///
    /// C: `FsmDispatch(me, e)`
    pub fn dispatch(&mut self, event: &Event) {
        let handler = self.state; // which function are we pointing at?
        handler(self, event); //     call it
    }

    /// Change state: point at a different function.
    ///
    /// C: `FsmTran_(me, target)`
    fn tran(&mut self, target: State) {
        self.state = target;
    }
}

// ---------------------------------------------------------------------------
// Part 2: the states - one function per state
// ---------------------------------------------------------------------------

impl Keyboard {
    /// Not a real state: it only exists to get us into `default`.
    fn initial(&mut self, _event: &Event) {
        println!("[initial] keyboard initialized -> default");
        self.tran(Keyboard::default);
    }

    fn default(&mut self, event: &Event) {
        match event {
            Event::ShiftDepressed => {
                println!("[default] shift depressed -> shifted");
                self.tran(Keyboard::shifted);
            }
            Event::AnyKey(key) => {
                let key = key.to_ascii_lowercase();
                println!("[default] key '{key}'");
                self.typed.push(key);
            }
            Event::ShiftReleased => {} // ignored in this state
        }
    }

    fn shifted(&mut self, event: &Event) {
        match event {
            Event::ShiftReleased => {
                println!("[shifted] shift released -> default");
                self.tran(Keyboard::default);
            }
            Event::AnyKey(key) => {
                let key = key.to_ascii_uppercase();
                println!("[shifted] key '{key}'");
                self.typed.push(key);
            }
            Event::ShiftDepressed => {} // ignored in this state
        }
    }
}

// ---------------------------------------------------------------------------
// Part 3: using it
// ---------------------------------------------------------------------------

fn main() {
    let mut keyboard = Keyboard::new();

    // `initial` ignores its event, so it does not matter which one we pass.
    keyboard.init(&Event::ShiftReleased);

    let events = [
        Event::AnyKey('h'),
        Event::AnyKey('i'),
        Event::ShiftDepressed,
        Event::AnyKey('r'),
        Event::AnyKey('u'),
        Event::ShiftReleased,
        Event::AnyKey('s'),
        Event::AnyKey('t'),
    ];

    // The same call every time. What happens depends on the current state.
    for event in &events {
        keyboard.dispatch(event);
    }

    println!("typed: {}", keyboard.typed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_changes_how_a_key_is_handled() {
        let mut keyboard = Keyboard::new();
        keyboard.init(&Event::ShiftReleased);

        keyboard.dispatch(&Event::AnyKey('a'));
        keyboard.dispatch(&Event::ShiftDepressed);
        keyboard.dispatch(&Event::AnyKey('b'));
        keyboard.dispatch(&Event::ShiftReleased);
        keyboard.dispatch(&Event::AnyKey('c'));

        assert_eq!(keyboard.typed, "aBc");
    }
}

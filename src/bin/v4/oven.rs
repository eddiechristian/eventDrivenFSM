//! The microwave oven, built on the shared mechanism in `src/hsm.rs`.
//!
//! ```text
//! DoorOpen                      entry: lamp on     exit: lamp off
//! DoorClosed                    starts in Idle
//!     Idle
//!     Cooking                   entry: heater on   exit: heater off
//!     Paused
//! ```
//!
//! `DoorClosed` is the parent of `Idle`, `Cooking` and `Paused`. It handles
//! `DoorOpened` and `AddTime` once, for all three.
//!
//! The oven tells the mechanism about its states in four places: the `State`
//! enum, and the `parent`, `initial` and `handle` functions.

use event_driven_fsm::hsm::{Machine, Reply, Signal};

/// The things that can happen to the oven.
pub enum Event {
    DoorOpened,
    DoorClosed,
    /// Start button.
    Start,
    /// Stop button.
    Stop,
    /// Add this many seconds.
    AddTime(u32),
    /// One second has passed.
    Tick,
}

/// The names of the states. `DoorClosed` is the parent of the last three.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    DoorOpen,
    DoorClosed,
    Idle,
    Cooking,
    Paused,
}

pub struct Oven {
    state: State,
    /// Seconds of cooking left.
    pub remaining: u32,
    /// What the oven did to its hardware, in order.
    pub log: Vec<&'static str>,
    /// The child of `DoorClosed` that was last active. Closing the door
    /// returns here.
    history: State,
}

/// Plug the oven into the reusable mechanism.
impl Machine for Oven {
    type Event = Event;
    type State = State;

    fn state(&self) -> State {
        self.state
    }

    fn set_state(&mut self, state: State) {
        self.state = state;
    }

    /// Who is inside whom.
    fn parent(state: State) -> Option<State> {
        match state {
            State::Idle | State::Cooking | State::Paused => Some(State::DoorClosed),
            State::DoorOpen | State::DoorClosed => None,
        }
    }

    /// Where a parent starts.
    fn initial(state: State) -> Option<State> {
        match state {
            State::DoorClosed => Some(State::Idle),
            _ => None,
        }
    }

    /// Which function belongs to which state.
    fn handle(&mut self, state: State, signal: &Signal<Event>) -> Reply<State> {
        match state {
            State::DoorOpen => self.door_open(signal),
            State::DoorClosed => self.door_closed(signal),
            State::Idle => self.idle(signal),
            State::Cooking => self.cooking(signal),
            State::Paused => self.paused(signal),
        }
    }
}

impl Oven {
    /// A new oven: door closed, idle, no time set. Call `init` before use.
    pub fn new() -> Oven {
        Oven {
            state: State::DoorClosed,
            remaining: 0,
            log: Vec::new(),
            history: State::Idle,
        }
    }

    /// Do something to the hardware: here, just record and print it.
    fn act(&mut self, action: &'static str) {
        println!("    {action}");
        self.log.push(action);
    }

    // ---- the state functions: one per state --------------------------------

    fn door_open(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Entry => {
                self.act("lamp on");
                Reply::Handled
            }
            Signal::Exit => {
                self.act("lamp off");
                Reply::Handled
            }
            // Back to whatever the oven was doing before the door opened.
            Signal::Event(Event::DoorClosed) => Reply::Tran(self.history),
            _ => Reply::Unhandled,
        }
    }

    /// The parent. Its two rules apply in `Idle`, `Cooking` and `Paused`.
    fn door_closed(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Event(Event::DoorOpened) => Reply::Tran(State::DoorOpen),
            Signal::Event(Event::AddTime(seconds)) => {
                self.remaining += seconds;
                Reply::Handled
            }
            _ => Reply::Unhandled,
        }
    }

    fn idle(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Entry => {
                self.history = State::Idle;
                Reply::Handled
            }
            Signal::Event(Event::Start) if self.remaining > 0 => Reply::Tran(State::Cooking),
            _ => Reply::Unhandled,
        }
    }

    fn cooking(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Entry => {
                self.history = State::Cooking;
                self.act("heater on");
                Reply::Handled
            }
            Signal::Exit => {
                self.act("heater off");
                Reply::Handled
            }
            Signal::Event(Event::Tick) if self.remaining > 1 => {
                self.remaining -= 1;
                Reply::Handled
            }
            Signal::Event(Event::Tick) => {
                self.remaining = 0;
                self.act("beep");
                Reply::Tran(State::Idle)
            }
            Signal::Event(Event::Stop) => Reply::Tran(State::Paused),
            _ => Reply::Unhandled,
        }
    }

    fn paused(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Entry => {
                self.history = State::Paused;
                Reply::Handled
            }
            Signal::Event(Event::Start) => Reply::Tran(State::Cooking),
            Signal::Event(Event::Stop) => {
                self.remaining = 0;
                Reply::Tran(State::Idle)
            }
            _ => Reply::Unhandled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Event::*;
    use super::*;

    /// Feed `events` to a new oven and return it.
    fn run(events: impl IntoIterator<Item = Event>) -> Oven {
        let mut oven = Oven::new();
        oven.init();
        for event in events {
            oven.dispatch(event);
        }
        oven
    }

    /// Check the log and the final state after `events`.
    fn check(events: impl IntoIterator<Item = Event>, log: &[&str], state: State) {
        let oven = run(events);
        assert_eq!(oven.log, log);
        assert_eq!(oven.state(), state);
    }

    #[test]
    fn full_cook() {
        check(
            [AddTime(2), Start, Tick, Tick],
            &["heater on", "beep", "heater off"],
            State::Idle,
        );
    }

    #[test]
    fn start_does_nothing_with_no_time_set() {
        check([Start], &[], State::Idle);
    }

    #[test]
    fn door_opened_is_inherited_by_cooking() {
        check(
            [AddTime(5), Start, DoorOpened],
            &["heater on", "heater off", "lamp on"],
            State::DoorOpen,
        );
    }

    #[test]
    fn door_opened_is_inherited_by_paused() {
        check(
            [AddTime(5), Start, Stop, DoorOpened],
            &["heater on", "heater off", "lamp on"],
            State::DoorOpen,
        );
    }

    #[test]
    fn door_opened_is_inherited_by_idle() {
        check([DoorOpened, DoorClosed], &["lamp on", "lamp off"], State::Idle);
    }

    #[test]
    fn add_time_is_ignored_while_the_door_is_open() {
        let oven = run([DoorOpened, AddTime(5), DoorClosed, Start]);
        assert_eq!(oven.log, ["lamp on", "lamp off"]);
        assert_eq!(oven.state(), State::Idle);
        assert_eq!(oven.remaining, 0);
    }

    #[test]
    fn second_stop_clears_the_time() {
        let oven = run([AddTime(5), Start, Stop, Stop]);
        assert_eq!(oven.log, ["heater on", "heater off"]);
        assert_eq!(oven.state(), State::Idle);
        assert_eq!(oven.remaining, 0);
    }

    #[test]
    fn closing_the_door_resumes_cooking() {
        let oven = run([AddTime(5), Start, DoorOpened, DoorClosed]);
        assert_eq!(
            oven.log,
            ["heater on", "heater off", "lamp on", "lamp off", "heater on"]
        );
        assert_eq!(oven.state(), State::Cooking);
        assert_eq!(oven.remaining, 5);
    }

    #[test]
    fn closing_the_door_returns_to_paused() {
        let oven = run([AddTime(5), Start, Tick, Stop, DoorOpened, DoorClosed]);
        assert_eq!(oven.log, ["heater on", "heater off", "lamp on", "lamp off"]);
        assert_eq!(oven.state(), State::Paused);
        assert_eq!(oven.remaining, 4);
    }

    #[test]
    fn a_cooking_oven_is_also_in_door_closed() {
        let oven = run([AddTime(5), Start]);
        assert!(oven.is_in(State::Cooking));
        assert!(oven.is_in(State::DoorClosed));
        assert!(!oven.is_in(State::DoorOpen));
    }
}

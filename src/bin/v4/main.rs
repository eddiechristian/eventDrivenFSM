//! Version 4: hierarchical state machines (states inside states), with
//! generics. Read `../v2/main.rs` first.
//!
//! In v2 every state stands alone, so a rule shared by several states has to
//! be repeated in each one. Here a state can sit inside a parent state, and
//! an event the child does not handle is passed up to the parent.
//!
//! `src/hsm.rs` is the mechanism, written once for any machine and kept in
//! the library so other exercises can use it. `oven.rs` is a microwave oven
//! built on it. The design is described in `plan.md` under "Microwave oven
//! in detail", design B.

pub mod oven;

use event_driven_fsm::hsm::Machine;
use oven::{Event, Oven, State};

fn main() {
    let mut oven = Oven::new();
    oven.init();

    let events = [
        ("AddTime(3)", Event::AddTime(3)),
        ("Start", Event::Start),
        ("Tick", Event::Tick),
        ("DoorOpened", Event::DoorOpened), // handled by the parent, DoorClosed
        ("DoorClosed", Event::DoorClosed), // history: back to Cooking
        ("Tick", Event::Tick),
        ("Stop", Event::Stop),
        ("Start", Event::Start),
        ("Tick", Event::Tick), // time is up
    ];

    for (name, event) in events {
        println!("{name}");
        oven.dispatch(event);
        println!(
            "  -> {:?} (door closed: {}, {}s left)",
            oven.state(),
            oven.is_in(State::DoorClosed),
            oven.remaining
        );
    }
}

//! Version 5: a digital watch. Read `../v4/main.rs` first.
//!
//! The second machine on the hierarchical mechanism. Nothing in `src/hsm.rs`
//! was changed to make it work, which is the point of a reusable mechanism.
//!
//! The watch nests three levels deep and has a child state that overrides
//! its parent. See `watch.rs` for the states, and `plan.md` under "Digital
//! watch in detail" for the design.

pub mod watch;

use event_driven_fsm::hsm::Machine;
use watch::{Event, State, Watch};

fn main() {
    let mut watch = Watch::new();
    println!("init");
    watch.init();

    let events = [
        ("Tick", Event::Tick), // handled two levels up, by Watch
        ("Tick", Event::Tick),
        ("Mode", Event::Mode),
        ("Set", Event::Set), // handled one level up, by Displaying
        ("Tick", Event::Tick), // overridden by Setting: the clock stands still
        ("Adjust", Event::Adjust),
        ("Adjust", Event::Adjust),
        ("Set", Event::Set), // hours -> minutes: still inside Setting
        ("Adjust", Event::Adjust),
        ("Set", Event::Set), // back to the display shown before
        ("Tick", Event::Tick),
    ];

    for (name, event) in events {
        println!("{name}");
        watch.dispatch(event);
        println!(
            "  -> {:?} (setting: {}) {}",
            watch.state(),
            watch.is_in(State::Setting),
            watch.time()
        );
    }
}

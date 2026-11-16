//! The digital watch, built on the shared mechanism in `src/hsm.rs`.
//!
//! ```text
//! Watch                         Tick: the clock advances
//!     Displaying                starts in ShowingTime      Set -> Setting
//!         ShowingTime           entry: show time           Mode -> ShowingDay
//!         ShowingDay            entry: show day            Mode -> ShowingTime
//!     Setting                   starts in SettingHours     Mode -> Displaying
//!                               entry: blink on            Tick: nothing
//!                               exit: blink off, seconds = 0
//!         SettingHours          entry: select hours        Adjust: hours + 1
//!                                                          Set -> SettingMinutes
//!         SettingMinutes        entry: select minutes      Adjust: minutes + 1
//!                                                          Set -> Displaying
//! ```
//!
//! What this shows that the oven in v4 did not:
//!
//! - **Three levels.** `Tick` in `ShowingDay` goes unhandled twice before
//!   `Watch` takes it.
//! - **A child overrides its parent.** `Setting` handles `Tick` itself and
//!   does nothing, so the clock stands still while it is being set.
//! - **A parent with entry and exit actions.** `blink on` runs once when
//!   `Setting` is entered, not again when moving from hours to minutes.

use event_driven_fsm::hsm::{Machine, Reply, Signal};

/// The things that can happen to the watch.
pub enum Event {
    /// Mode button.
    Mode,
    /// Set button.
    Set,
    /// Adjust button.
    Adjust,
    /// One second has passed.
    Tick,
}

/// The names of the states, three levels deep.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State {
    Watch,
    Displaying,
    ShowingTime,
    ShowingDay,
    Setting,
    SettingHours,
    SettingMinutes,
}

pub const DAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

pub struct Watch {
    state: State,
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    /// Index into `DAYS`.
    pub day: usize,
    /// What the watch did to its display, in order.
    pub log: Vec<&'static str>,
    /// The child of `Displaying` that was last active. Leaving `Setting`
    /// returns here.
    last_display: State,
}

/// Plug the watch into the reusable mechanism.
impl Machine for Watch {
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
            State::Watch => None,
            State::Displaying | State::Setting => Some(State::Watch),
            State::ShowingTime | State::ShowingDay => Some(State::Displaying),
            State::SettingHours | State::SettingMinutes => Some(State::Setting),
        }
    }

    /// Where a parent starts.
    fn initial(state: State) -> Option<State> {
        match state {
            State::Watch => Some(State::Displaying),
            State::Displaying => Some(State::ShowingTime),
            State::Setting => Some(State::SettingHours),
            _ => None,
        }
    }

    /// Which function belongs to which state.
    fn handle(&mut self, state: State, signal: &Signal<Event>) -> Reply<State> {
        match state {
            State::Watch => self.watch(signal),
            State::Displaying => self.displaying(signal),
            State::ShowingTime => self.showing_time(signal),
            State::ShowingDay => self.showing_day(signal),
            State::Setting => self.setting(signal),
            State::SettingHours => self.setting_hours(signal),
            State::SettingMinutes => self.setting_minutes(signal),
        }
    }
}

impl Watch {
    /// A new watch: Monday 00:00:00, showing the time. Call `init` before use.
    pub fn new() -> Watch {
        Watch {
            state: State::Watch,
            hours: 0,
            minutes: 0,
            seconds: 0,
            day: 0,
            log: Vec::new(),
            last_display: State::ShowingTime,
        }
    }

    /// The clock as text, for example `Mon 07:05:09`.
    pub fn time(&self) -> String {
        format!(
            "{} {:02}:{:02}:{:02}",
            DAYS[self.day], self.hours, self.minutes, self.seconds
        )
    }

    /// Do something to the display: here, just record and print it.
    fn act(&mut self, action: &'static str) {
        println!("    {action}");
        self.log.push(action);
    }

    /// Move the clock on by one second.
    fn advance(&mut self) {
        self.seconds += 1;
        if self.seconds == 60 {
            self.seconds = 0;
            self.minutes += 1;
        }
        if self.minutes == 60 {
            self.minutes = 0;
            self.hours += 1;
        }
        if self.hours == 24 {
            self.hours = 0;
            self.day = (self.day + 1) % DAYS.len();
        }
    }

    // ---- the state functions: one per state --------------------------------

    /// Level 1, the outermost state. Its rule applies everywhere, unless a
    /// state further in handles the event first.
    fn watch(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Event(Event::Tick) => {
                self.advance();
                Reply::Handled
            }
            _ => Reply::Unhandled,
        }
    }

    /// Level 2. Its rule applies in `ShowingTime` and `ShowingDay`.
    fn displaying(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Event(Event::Set) => Reply::Tran(State::Setting),
            _ => Reply::Unhandled,
        }
    }

    fn showing_time(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Entry => {
                self.last_display = State::ShowingTime;
                self.act("show time");
                Reply::Handled
            }
            Signal::Event(Event::Mode) => Reply::Tran(State::ShowingDay),
            _ => Reply::Unhandled,
        }
    }

    fn showing_day(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Entry => {
                self.last_display = State::ShowingDay;
                self.act("show day");
                Reply::Handled
            }
            Signal::Event(Event::Mode) => Reply::Tran(State::ShowingTime),
            _ => Reply::Unhandled,
        }
    }

    /// Level 2. Its rules apply in `SettingHours` and `SettingMinutes`.
    fn setting(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Entry => {
                self.act("blink on");
                Reply::Handled
            }
            Signal::Exit => {
                self.seconds = 0;
                self.act("blink off");
                Reply::Handled
            }
            // Overrides the parent: `watch` never sees this `Tick`, so the
            // clock stands still while it is being set.
            Signal::Event(Event::Tick) => Reply::Handled,
            // Leave setting from either child, back to the display that was
            // showing before.
            Signal::Event(Event::Mode) => Reply::Tran(self.last_display),
            _ => Reply::Unhandled,
        }
    }

    fn setting_hours(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Entry => {
                self.act("select hours");
                Reply::Handled
            }
            Signal::Event(Event::Adjust) => {
                self.hours = (self.hours + 1) % 24;
                Reply::Handled
            }
            Signal::Event(Event::Set) => Reply::Tran(State::SettingMinutes),
            _ => Reply::Unhandled,
        }
    }

    fn setting_minutes(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Entry => {
                self.act("select minutes");
                Reply::Handled
            }
            Signal::Event(Event::Adjust) => {
                self.minutes = (self.minutes + 1) % 60;
                Reply::Handled
            }
            Signal::Event(Event::Set) => Reply::Tran(self.last_display),
            _ => Reply::Unhandled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Event::*;
    use super::*;

    /// Feed `events` to a new watch and return it.
    fn run(events: impl IntoIterator<Item = Event>) -> Watch {
        let mut watch = Watch::new();
        watch.init();
        for event in events {
            watch.dispatch(event);
        }
        watch
    }

    /// Check the log, the final state and the clock after `events`.
    fn check(events: impl IntoIterator<Item = Event>, log: &[&str], state: State, time: &str) {
        let watch = run(events);
        assert_eq!(watch.log, log);
        assert_eq!(watch.state(), state);
        assert_eq!(watch.time(), time);
    }

    #[test]
    fn starts_showing_the_time() {
        let watch = run([]);
        assert_eq!(watch.log, ["show time"]);
        assert_eq!(watch.state(), State::ShowingTime);
        assert!(watch.is_in(State::Displaying));
        assert!(watch.is_in(State::Watch));
        assert!(!watch.is_in(State::Setting));
    }

    #[test]
    fn mode_switches_between_time_and_day() {
        check([Mode], &["show time", "show day"], State::ShowingDay, "Mon 00:00:00");
        check(
            [Mode, Mode],
            &["show time", "show day", "show time"],
            State::ShowingTime,
            "Mon 00:00:00",
        );
    }

    #[test]
    fn tick_is_inherited_from_two_levels_up() {
        check([Tick, Tick], &["show time"], State::ShowingTime, "Mon 00:00:02");
        check(
            [Mode, Tick, Tick],
            &["show time", "show day"],
            State::ShowingDay,
            "Mon 00:00:02",
        );
    }

    #[test]
    fn the_clock_rolls_over() {
        let mut watch = run([]);
        (watch.hours, watch.minutes, watch.seconds) = (23, 59, 59);
        watch.dispatch(Tick);
        assert_eq!(watch.time(), "Tue 00:00:00");

        (watch.day, watch.hours, watch.minutes, watch.seconds) = (6, 23, 59, 59);
        watch.dispatch(Tick);
        assert_eq!(watch.time(), "Mon 00:00:00");
    }

    #[test]
    fn set_enters_setting_at_hours() {
        check(
            [Set],
            &["show time", "blink on", "select hours"],
            State::SettingHours,
            "Mon 00:00:00",
        );
    }

    #[test]
    fn set_is_inherited_by_showing_day() {
        check(
            [Mode, Set],
            &["show time", "show day", "blink on", "select hours"],
            State::SettingHours,
            "Mon 00:00:00",
        );
    }

    #[test]
    fn setting_the_time() {
        // `blink on` appears once: moving from hours to minutes stays inside
        // `Setting`.
        check(
            [Set, Adjust, Adjust, Set, Adjust, Set],
            &[
                "show time",
                "blink on",
                "select hours",
                "select minutes",
                "blink off",
                "show time",
            ],
            State::ShowingTime,
            "Mon 02:01:00",
        );
    }

    #[test]
    fn tick_is_overridden_while_setting() {
        check(
            [Tick, Tick, Tick, Set, Tick, Tick],
            &["show time", "blink on", "select hours"],
            State::SettingHours,
            "Mon 00:00:03",
        );
        check(
            [Tick, Tick, Tick, Set, Set, Tick, Tick],
            &["show time", "blink on", "select hours", "select minutes"],
            State::SettingMinutes,
            "Mon 00:00:03",
        );
    }

    #[test]
    fn leaving_setting_zeroes_the_seconds() {
        check(
            [Tick, Tick, Tick, Set, Mode],
            &["show time", "blink on", "select hours", "blink off", "show time"],
            State::ShowingTime,
            "Mon 00:00:00",
        );
    }

    #[test]
    fn mode_leaves_setting_from_minutes_too() {
        check(
            [Set, Set, Mode],
            &[
                "show time",
                "blink on",
                "select hours",
                "select minutes",
                "blink off",
                "show time",
            ],
            State::ShowingTime,
            "Mon 00:00:00",
        );
    }

    #[test]
    fn leaving_setting_returns_to_the_day_display() {
        check(
            [Mode, Set, Set, Set],
            &[
                "show time",
                "show day",
                "blink on",
                "select hours",
                "select minutes",
                "blink off",
                "show day",
            ],
            State::ShowingDay,
            "Mon 00:00:00",
        );
    }

    #[test]
    fn adjust_is_ignored_while_displaying() {
        check([Adjust], &["show time"], State::ShowingTime, "Mon 00:00:00");
    }

    #[test]
    fn hours_and_minutes_wrap_when_adjusted() {
        let mut watch = run([Set]);
        watch.hours = 23;
        watch.dispatch(Adjust);
        assert_eq!(watch.hours, 0);

        watch.dispatch(Set);
        watch.minutes = 59;
        watch.dispatch(Adjust);
        assert_eq!(watch.minutes, 0);
        assert_eq!(watch.hours, 0); // minutes wrapping does not touch hours
    }
}

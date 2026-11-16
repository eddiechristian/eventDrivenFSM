//! The reusable mechanism for hierarchical state machines (states inside
//! states). It knows nothing about ovens or watches.
//!
//! It lives in the library so every exercise under `src/bin/` can use it.
//!
//! Compared with `bin/v2/fsm.rs`:
//!
//! | v2 (flat)                          | this file                             |
//! |------------------------------------|---------------------------------------|
//! | a state is a function pointer      | a state is a value of the machine's   |
//! |                                    | own `State` enum; `handle` picks the  |
//! |                                    | function that goes with it            |
//! | states stand alone                 | a state can have a `parent` and a     |
//! |                                    | starting child (`initial`)            |
//! | a state function returns nothing   | it returns a `Reply`                  |
//! | a state function gets an event     | it gets a `Signal`: entry, exit or    |
//! |                                    | an event                              |
//! | the state calls `tran` itself      | the state returns `Reply::Tran` and   |
//! |                                    | the mechanism does the rest           |
//! | `dispatch(&event)`                 | `dispatch(event)`                     |
//!
//! Because a state is a plain value, nothing here needs a `static`, a
//! lifetime or a `'static` bound.

/// What a state function is being told.
pub enum Signal<E> {
    /// The state has just been entered. Run its entry action.
    Entry,
    /// The state is about to be left. Run its exit action.
    Exit,
    /// Something happened. React to it, or pass it up.
    Event(E),
}

/// What a state function answers. `S` is the machine's state enum.
pub enum Reply<S> {
    /// Done. No state change.
    Handled,
    /// Not mine. Ask my parent.
    Unhandled,
    /// Done. Now go to this state.
    Tran(S),
}

/// What a machine implements to get `init`, `dispatch` and `is_in` for free.
pub trait Machine: Sized {
    /// The events this machine reacts to.
    type Event;

    /// The names of this machine's states: an enum with one variant per
    /// state, parents included.
    type State: Copy + PartialEq;

    /// The current (innermost) state.
    fn state(&self) -> Self::State;

    /// Store the current state. Only the mechanism should call this: a state
    /// function changes state by returning `Reply::Tran`.
    fn set_state(&mut self, state: Self::State);

    /// The state `state` sits inside. `None` for an outermost state.
    fn parent(state: Self::State) -> Option<Self::State>;

    /// The child to start in when `state` is the target of a transition.
    /// `None` for a state with no children.
    fn initial(state: Self::State) -> Option<Self::State>;

    /// Call the state function for `state`.
    fn handle(&mut self, state: Self::State, signal: &Signal<Self::Event>) -> Reply<Self::State>;

    /// Enter the starting state: its parents first, then the state, then its
    /// `initial` children.
    fn init(&mut self) {
        for state in path_to_top::<Self>(self.state()).into_iter().rev() {
            self.handle(state, &Signal::Entry);
        }
        enter_initial_children(self);
    }

    /// Give an event to the current state. If it answers `Unhandled`, give
    /// the event to its parent, and so on upward.
    fn dispatch(&mut self, event: Self::Event) {
        let signal = Signal::Event(event);
        let mut asked = Some(self.state());

        while let Some(state) = asked {
            match self.handle(state, &signal) {
                Reply::Handled => return,
                Reply::Unhandled => asked = Self::parent(state),
                Reply::Tran(target) => {
                    tran(self, target);
                    return;
                }
            }
        }
        // Nobody handled it: the event is ignored.
    }

    /// Is the machine in `state`, either directly or in one of its children?
    fn is_in(&self, state: Self::State) -> bool {
        path_to_top::<Self>(self.state()).contains(&state)
    }
}

/// The state itself, then its parent, then its parent's parent, and so on.
fn path_to_top<M: Machine>(state: M::State) -> Vec<M::State> {
    let mut path = vec![state];
    let mut current = state;
    while let Some(parent) = M::parent(current) {
        path.push(parent);
        current = parent;
    }
    path
}

/// Change state, running exit and entry actions on the way.
///
/// Example, oven cooking and the door opens (`Cooking` -> `DoorOpen`):
///
/// ```text
/// from = [Cooking, DoorClosed]     to = [DoorOpen]
/// common parent: none
/// exit Cooking, exit DoorClosed, enter DoorOpen
/// ```
///
/// Example, stop pressed while cooking (`Cooking` -> `Paused`):
///
/// ```text
/// from = [Cooking, DoorClosed]     to = [Paused, DoorClosed]
/// common parent: DoorClosed (not exited, not entered)
/// exit Cooking, enter Paused
/// ```
fn tran<M: Machine>(machine: &mut M, target: M::State) {
    let from = path_to_top::<M>(machine.state());
    let to = path_to_top::<M>(target);

    // The nearest state that contains both ends. Everything from there
    // upward is left alone.
    let common = from.iter().find(|state| to.contains(state)).copied();
    let is_common = |state: &M::State| common == Some(*state);

    // Exits: child first, stopping below the common parent.
    for &state in from.iter().take_while(|s| !is_common(s)) {
        machine.handle(state, &Signal::Exit);
    }

    // Entries: parent first, starting below the common parent.
    let to_enter: Vec<M::State> = to.iter().copied().take_while(|s| !is_common(s)).collect();
    for state in to_enter.into_iter().rev() {
        machine.handle(state, &Signal::Entry);
    }

    machine.set_state(target);
    enter_initial_children(machine);
}

/// If the current state has a starting child, enter it, and repeat.
fn enter_initial_children<M: Machine>(machine: &mut M) {
    while let Some(child) = M::initial(machine.state()) {
        machine.handle(child, &Signal::Entry);
        machine.set_state(child);
    }
}

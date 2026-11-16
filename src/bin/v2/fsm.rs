//! The reusable FSM mechanism: Listing 1 as a Rust module.
//!
//! This is the same code as Part 1 of `../v1/main.rs`, with the two keyboard-specific
//! names replaced by placeholders:
//!
//!   `M` = the machine type (v1: `Keyboard`)
//!   `E` = the event type   (v1: `Event`)
//!
//! | v1 (no generics)                  | v2 (this file)                       |
//! |-----------------------------------|--------------------------------------|
//! | `fn(&mut Keyboard, &Event)`       | `fn(&mut M, &E)`                     |
//! | `state` field inside `Keyboard`   | `Fsm<M, E>` struct, embedded by `M`  |
//! | `Keyboard::dispatch` / `tran`     | `Machine::dispatch` / `tran`         |
//!
//! C does the same job with casts: `Keyboard` embeds `Fsm` as its first
//! member and pointers are cast between the two. Generics give the same
//! reuse while the compiler still checks every type.

/// A state is a function: "here is the machine and an event, react to it".
///
/// C: `typedef void (*State)(Fsm *, Event const *);`
pub type State<M, E> = fn(&mut M, &E);

/// Holds the current state. A machine embeds one of these.
///
/// C: `struct Fsm { State state__; }`
pub struct Fsm<M, E> {
    state: State<M, E>,
}

impl<M, E> Fsm<M, E> {
    /// Start out pointing at the `initial` function. Nothing runs yet.
    ///
    /// C: `FsmCtor_(me, init)`
    pub fn new(initial: State<M, E>) -> Fsm<M, E> {
        Fsm { state: initial }
    }
}

/// What a machine implements to get `init`, `dispatch` and `tran` for free.
///
/// The machine only has to say what its event type is and where its `Fsm`
/// lives. The three methods below are then written once, for every machine.
pub trait Machine: Sized {
    /// The events this machine reacts to.
    type Event;

    /// Where this machine keeps its `Fsm`.
    fn fsm(&mut self) -> &mut Fsm<Self, Self::Event>;

    /// Run the `initial` function once.
    ///
    /// C: `FsmInit(me, e)` - in the C listing it is the same code as dispatch.
    fn init(&mut self, event: &Self::Event) {
        self.dispatch(event);
    }

    /// Give an event to the current state.
    ///
    /// C: `FsmDispatch(me, e)`
    fn dispatch(&mut self, event: &Self::Event) {
        let handler = self.fsm().state; // which function are we pointing at?
        handler(self, event); //           call it
    }

    /// Change state: point at a different function.
    ///
    /// C: `FsmTran_(me, target)`
    fn tran(&mut self, target: State<Self, Self::Event>) {
        self.fsm().state = target;
    }
}

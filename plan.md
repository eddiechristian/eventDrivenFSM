# Plan: exercises for event-driven state machines

Source article: [State Machines for Event-Driven Systems](https://barrgroup.com/blog/state-machines-event-driven-systems)

## Done

- [x] Keyboard with a Shift key, v1: no generics (`src/bin/v1/main.rs`)
- [x] Keyboard with a Shift key, v2: generics, reusable FSM mechanism (`src/bin/v2/`)
- [x] Microwave oven, v4: hierarchical states on a generic mechanism
      (`src/bin/v4/`, mechanism in `src/hsm.rs`)
- [x] Digital watch, v5: three levels, second machine on the same mechanism
      (`src/bin/v5/`)

## Recommended path

Do these in order.

- [ ] **Comment stripper** - easy to test: feed it strings, compare the output.
- [ ] **Time bomb** - adds time (tick events) and guard conditions.
- [ ] **Calculator** (flat) - by now the duplication across states shows why
      hierarchical state machines exist.
- [ ] **Microwave oven with nested enums** - the first hierarchical machine:
      inherited transitions, entry and exit actions, history.
- [x] **Microwave oven on a generic mechanism (v4)** - the same oven on a
      reusable engine.
- [x] **Digital watch (v5)** - three levels of nesting and a child that
      overrides its parent, on the same engine.
- [ ] **Calculator again**, converted to the same mechanism.

## All exercise ideas, easiest first

1. **Turnstile**
   - States: locked, unlocked. Events: coin, push.
   - Teaches: writing a machine from scratch without looking at the keyboard code.

2. **Traffic light with a pedestrian button**
   - Events: tick, button pressed.
   - Teaches: time as an event. The loop feeds `Tick`; the machine counts ticks in a field.

3. **Comment stripper or word counter**
   - Each input character is an event. States: code, slash, comment, star.
   - Teaches: the function-pointer style on a real text-processing job.
   - Full description: see "Comment stripper in detail" below.

4. **Vending machine**
   - Coins accumulate in a field; "select" only works with enough credit.
   - Teaches: extended state and guard conditions (same event, different
     outcome depending on data).

5. **Time bomb**
   - Set a timeout with up/down, arm it, defuse it by entering a code before
     the ticks run out.
   - Teaches: ticks, extended state and guards together in a small machine.

6. **Button debouncer with click / double-click / long-press**
   - Events: press, release, tick.
   - Teaches: a realistic embedded task.

7. **Microwave oven or digital watch**
   - Modes within modes: door open while cooking, setting hours versus minutes.
   - Teaches: where a flat machine starts to strain.
   - Full description: see "Microwave oven in detail" and "Digital watch
     in detail" below.

8. **Calculator**
   - States: operand1, opEntered, operand2, result. Plus negative numbers,
     decimal points and divide-by-zero errors.
   - Teaches: "clear" and "off" must be handled in every state, so the same
     transitions get repeated in each handler. That repetition is the problem
     hierarchical state machines solve.

## Comment stripper in detail

### Goal

Read C-style source text one character at a time and write it back out with
the comments removed. Two kinds of comment:

- line comment: `//` up to the end of the line
- block comment: `/*` up to the next `*/`

Example:

```text
in:   x = 1; // set x
      y = a / b; /* divide */ z = 3;

out:  x = 1; 
      y = a / b;  z = 3;
```

### Why a state machine is needed

One character is not enough to decide what to do. A `/` could be a division,
the start of `//`, or the start of `/*`. A `*` inside a block comment could be
ordinary text or the start of `*/`. The machine remembers "I just saw a `/`"
or "I just saw a `*`" by being in a different state.

### Events

```rust
enum Event {
    Char(char), // the next character of the input
    End,        // no more input
}
```

### Extended state

- `output: String` - the text with comments removed.

### States

| State           | Meaning                                              |
|-----------------|------------------------------------------------------|
| `code`          | Normal text. Characters are copied to the output.    |
| `slash`         | Just saw a `/` in code. Not yet known what it means. |
| `line_comment`  | Inside `// ...`. Characters are dropped.             |
| `block_comment` | Inside `/* ...`. Characters are dropped.             |
| `star`          | Inside a block comment and just saw a `*`.           |

### Transitions

| State           | Event           | Action                  | Next state      |
|-----------------|-----------------|-------------------------|-----------------|
| `code`          | `/`             | nothing (hold it back)  | `slash`         |
| `code`          | any other char  | output the char         | `code`          |
| `slash`         | `/`             | nothing                 | `line_comment`  |
| `slash`         | `*`             | nothing                 | `block_comment` |
| `slash`         | any other char  | output `/` and the char | `code`          |
| `slash`         | `End`           | output `/`              | `code`          |
| `line_comment`  | newline         | output the newline      | `code`          |
| `line_comment`  | any other char  | nothing                 | `line_comment`  |
| `block_comment` | `*`             | nothing                 | `star`          |
| `block_comment` | any other char  | nothing                 | `block_comment` |
| `star`          | `/`             | nothing                 | `code`          |
| `star`          | `*`             | nothing                 | `star`          |
| `star`          | any other char  | nothing                 | `block_comment` |

`End` in any state not listed is ignored.

```text
                    '/'                '/'
        +------+ --------> +-------+ --------> +--------------+
   ---> | code |           | slash |           | line_comment |
        +------+ <-------- +-------+           +--------------+
          ^  ^     other       |                      |
          |  |                 | '*'                  |
          |  +-----------------|----------------------+
          |       newline      v
          |             +---------------+   '*'    +------+
          |             | block_comment | -------> | star | --+
          |             +---------------+ <------- +------+   | '*'
          |                                other     |  ^     |
          |                    '/'                   |  +-----+
          +------------------------------------------+
```

### Traps to watch for

- **The held-back slash.** In `code`, a `/` is not output straight away. If
  the next character is not `/` or `*`, the `slash` state must output both
  the `/` and that character. Forgetting this turns `a / b` into `a  b`.
- **Slash at the very end.** Input `x /` must still output the `/`. This is
  what the `End` event is for.
- **`**/`.** In `star`, another `*` must stay in `star`, not go back to
  `block_comment`. Otherwise `/* text **/` never ends.
- **The newline after a line comment** is kept, so line numbers do not change.

### Test cases

| Input                    | Expected output       | Checks                    |
|--------------------------|-----------------------|---------------------------|
| `a = b / c;`             | `a = b / c;`          | division is not a comment |
| `x = 1; // note\ny = 2;` | `x = 1; \ny = 2;`     | line comment, newline kept|
| `a /* b */ c`            | `a  c`                | block comment             |
| `a /* b * c */ d`        | `a  d`                | lone `*` inside a comment |
| `a /* b **/ c`           | `a  c`                | the `**/` trap            |
| `a /* b\nc */ d`         | `a  d`                | comment spanning lines    |
| `x /`                    | `x /`                 | slash at end of input     |
| `a // b`                 | `a `                  | input ends inside comment |

### Steps

- [ ] Write it in the v1 style (no generics, one file, `src/stripper.rs`).
- [ ] Add a `strip(input: &str) -> String` helper: create the machine, dispatch
      `Event::Char` for each character, dispatch `Event::End`, return `output`.
- [ ] Turn the test-case table into unit tests.
- [ ] Rewrite it on top of `v2/fsm.rs`. Only the plumbing should change, not
      the state functions. This proves the generic mechanism is reusable.

### Stretch goals

- [ ] **String literals.** `s = "// not a comment";` must come out unchanged.
      Add a `string` state entered on `"` and left on the next `"`.
- [ ] **Escapes in strings.** `"say \"hi\" // still a string"` needs a
      `string_escape` state entered on `\`.
- [ ] **Char literals.** `'"'` must not start a string. Same pattern as strings.
- [ ] **Keep line numbers.** Output the newlines found inside block comments.
- [ ] **Unterminated block comment.** On `End` in `block_comment` or `star`,
      report an error instead of ending silently.

## Microwave oven in detail

The first hierarchical state machine. Do it after the flat calculator.

### Goal

Model a microwave oven with a door, a heater, a lamp and a countdown timer.
The machine reacts to buttons, the door and a one-second tick, and records
what it does to the hardware in a log:

```text
events:  AddTime(2)  Start  Tick  Tick
log:     heater on, beep, heater off
```

### Why a hierarchy is needed

Opening the door must work whatever the oven is doing. In a flat machine
`idle`, `cooking` and `paused` each need their own `DoorOpened` transition,
and a fourth copy appears with every state added later. With a hierarchy the
three states become children of one parent, `door_closed`, and the parent
handles `DoorOpened` once.

The same goes for actions. "Heater off" must run however `cooking` is left:
stop button, timer finished or door opened. As an exit action it is written
once and cannot be forgotten.

### Events

```rust
enum Event {
    DoorOpened,
    DoorClosed,
    Start,        // start button
    Stop,         // stop button
    AddTime(u32), // add this many seconds
    Tick,         // one second has passed
}
```

### Extended state

- `remaining: u32` - seconds of cooking left.
- `log: Vec<&'static str>` - what the oven did, for the tests.
- the child of `door_closed` that was last active, for history (step 3):
  `last_closed: Closed` in design A, `history: State` in design B.

### States

```text
top
+-- door_open
+-- door_closed          starts in idle
    +-- idle
    +-- cooking
    +-- paused
```

| State         | Parent        | Entry action | Exit action  | Meaning                         |
|---------------|---------------|--------------|--------------|---------------------------------|
| `door_open`   | none          | `lamp on`    | `lamp off`   | Door is open. Nothing can run.  |
| `door_closed` | none          |              |              | Parent of the three below.      |
| `idle`        | `door_closed` |              |              | Not cooking. Time can be set.   |
| `cooking`     | `door_closed` | `heater on`  | `heater off` | Heating and counting down.      |
| `paused`      | `door_closed` |              |              | Stopped with time still left.   |

The machine is always in exactly one of `door_open`, `idle`, `cooking` or
`paused`. `door_closed` is never the current state by itself.

### Transitions

| State         | Event        | Guard            | Action                    | Next state    |
|---------------|--------------|------------------|---------------------------|---------------|
| `door_open`   | `DoorClosed` |                  |                           | `door_closed` |
| `door_closed` | `DoorOpened` |                  |                           | `door_open`   |
| `door_closed` | `AddTime(n)` |                  | `remaining += n`          | (stay)        |
| `idle`        | `Start`      | `remaining > 0`  |                           | `cooking`     |
| `cooking`     | `Tick`       | `remaining > 1`  | `remaining -= 1`          | (stay)        |
| `cooking`     | `Tick`       | `remaining <= 1` | `remaining = 0`, `beep`   | `idle`        |
| `cooking`     | `Stop`       |                  |                           | `paused`      |
| `paused`      | `Start`      |                  |                           | `cooking`     |
| `paused`      | `Stop`       |                  | `remaining = 0`           | `idle`        |

Anything not listed is ignored. That includes `AddTime` while the door is open.

- The two `door_closed` rows apply in `idle`, `cooking` and `paused`. They
  are the inherited transitions.
- `door_open` -> `door_closed` lands in `idle`, the parent's starting child.
  Step 3 changes this to the remembered child.

### Two designs

Build the oven twice. Both must pass the same test cases.

| Design                      | Where            | Purpose                                          |
|-----------------------------|------------------|--------------------------------------------------|
| A: nested enums             | `src/bin/oven/`  | Learn the three ideas with nothing hidden.       |
| B: generic mechanism (v4)   | `src/bin/v4/`    | A reusable engine, as v2 was for flat machines.  |

Do A first. B then answers "how do I make this general?".

### Design A: nested enums

The hierarchy is written into the type. A state inside a state is an enum
inside an enum:

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
enum State {
    DoorOpen,
    DoorClosed(Closed), // a state with a state inside it
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Closed {
    Idle,
    Cooking,
    Paused,
}
```

`State::DoorClosed(Closed::Cooking)` means: the door is closed, and within
that, the oven is cooking.

**Inherited transitions** are `match` arms with `DoorClosed(_)`, which means
"door closed, whichever child". Child arms come first, parent arms after, so
the child gets the first chance at an event:

```rust
fn dispatch(&mut self, event: Event) {
    use {Closed::*, State::*};

    let next = match (self.state, event) {
        // rules for one child
        (DoorClosed(Idle), Event::Start) if self.remaining > 0 => Some(DoorClosed(Cooking)),
        (DoorClosed(Cooking), Event::Stop) => Some(DoorClosed(Paused)),
        // ... the other child rules from the transition table

        // rules for the parent: any child
        (DoorClosed(_), Event::DoorOpened) => Some(DoorOpen),
        (DoorClosed(_), Event::AddTime(n)) => { self.remaining += n; None }

        (DoorOpen, Event::DoorClosed) => Some(DoorClosed(Idle)),
        _ => None, // ignored
    };

    if let Some(next) = next {
        self.exit(self.state);
        self.state = next;
        self.enter(next);
    }
}
```

An arm returns `Some(state)` to change state and `None` to stay.

**Entry and exit actions** are two functions, called only from the one place
above where the state changes:

```rust
fn enter(&mut self, state: State) {
    match state {
        State::DoorOpen => self.log.push("lamp on"),
        State::DoorClosed(Closed::Cooking) => self.log.push("heater on"),
        _ => {}
    }
}
```

`exit` is the same with `lamp off` and `heater off`.

**History** is one field, `last_closed: Closed`. `exit` saves the child when
the state being left is `DoorClosed(child)`. The `DoorClosed` event then goes
to `DoorClosed(self.last_closed)` instead of `DoorClosed(Idle)`.

Limits of this design:

- It is one `match` for this oven. Nothing is reusable by another machine.
- `enter` and `exit` are correct only because the parent `DoorClosed` has no
  actions of its own. If it had, `Cooking` -> `Paused` would wrongly run them,
  because that transition never leaves `DoorClosed`.

### Design B: generic mechanism (v4)

The same oven on a reusable engine. The mechanism is in the library so that
other exercises can use it:

```text
src/
    lib.rs        the library: `pub mod hsm;`
    hsm.rs        the mechanism, generic, knows nothing about ovens
    bin/v4/
        main.rs   runs the oven
        oven.rs   the oven, written on top of hsm.rs
```

`oven.rs` imports it with `use event_driven_fsm::hsm::{...}`.

Three things `hsm.rs` has that `v2/fsm.rs` does not:

1. **A handler can say "not mine".** A state function returns a reply instead
   of nothing.
2. **A state knows its parent.**
3. **Entry and exit are delivered to the handler** like events, so each state
   keeps its own actions.

A state is a value of an enum that the machine defines, one variant per
state, parents included. The machine tells the mechanism about the hierarchy
through trait methods.

```rust
// hsm.rs

/// What a state function is being told.
pub enum Signal<E> {
    Entry,
    Exit,
    Event(E),
}

/// What a state function answers. `S` is the machine's state enum.
pub enum Reply<S> {
    Handled,   // done, no state change
    Unhandled, // not mine, ask my parent
    Tran(S),   // done, now go to this state
}

/// What a machine implements to get `init`, `dispatch` and `is_in` for free.
pub trait Machine: Sized {
    type Event;
    type State: Copy + PartialEq;

    fn state(&self) -> Self::State;
    fn set_state(&mut self, state: Self::State);
    fn parent(state: Self::State) -> Option<Self::State>;
    fn initial(state: Self::State) -> Option<Self::State>; // starting child
    fn handle(&mut self, state: Self::State, signal: &Signal<Self::Event>) -> Reply<Self::State>;

    fn init(&mut self) { /* enter the starting state and its initial children */ }
    fn dispatch(&mut self, event: Self::Event) { /* see below */ }
    fn is_in(&self, state: Self::State) -> bool { /* in it, or in one of its children? */ }
}
```

The oven names its states and fills in the trait:

```rust
// oven.rs

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum State { DoorOpen, DoorClosed, Idle, Cooking, Paused }

impl Machine for Oven {
    type Event = Event;
    type State = State;

    fn state(&self) -> State { self.state }
    fn set_state(&mut self, state: State) { self.state = state; }

    // who is inside whom
    fn parent(state: State) -> Option<State> {
        match state {
            State::Idle | State::Cooking | State::Paused => Some(State::DoorClosed),
            State::DoorOpen | State::DoorClosed => None,
        }
    }

    // where a parent starts
    fn initial(state: State) -> Option<State> {
        match state {
            State::DoorClosed => Some(State::Idle),
            _ => None,
        }
    }

    // which function belongs to which state
    fn handle(&mut self, state: State, signal: &Signal<Event>) -> Reply<State> {
        match state {
            State::Cooking => self.cooking(signal),
            // ... one line per state
        }
    }
}

impl Oven {
    fn cooking(&mut self, signal: &Signal<Event>) -> Reply<State> {
        match signal {
            Signal::Entry => { self.act("heater on"); Reply::Handled }
            Signal::Exit => { self.act("heater off"); Reply::Handled }
            Signal::Event(Event::Stop) => Reply::Tran(State::Paused),
            // ...
            _ => Reply::Unhandled,
        }
    }
}
```

**Dispatch:** give the event to the current state. On `Unhandled`, give it to
the parent, and so on upward. If the top is reached, the event is ignored.

**Transition** from the current state to a target:

1. List the ancestors of the current state and of the target.
2. Find the nearest state that is in both lists (the common parent). There
   may be none.
3. Send `Exit` to the current state and each ancestor below the common
   parent, child first.
4. Send `Entry` to each state from just below the common parent down to the
   target, parent first.
5. If the target has an `initial` child, enter that too, and repeat.

**History:** each child of `DoorClosed` stores itself in `self.history` on
`Entry`. `door_open` returns `Reply::Tran(self.history)` on `DoorClosed`.

**Compared with the article.** The article's C code stores the state as a
function pointer, as v1 and v2 do. Here the state is a name and `handle`
looks up the function. That costs one `match` per call and one line per
state. In return the compiler rejects a state that has no parent rule or no
handler, and nothing needs a `static` or a lifetime.

### Traps to watch for

Both designs:

- **Exit order is child first, entry order is parent first.** Opening the
  door while cooking must log `heater off` before `lamp on`.
- **The action runs before the exit.** The code does the action and then asks
  for the state change, so the last tick logs `beep` before `heater off`. UML
  says the action comes after the exit. Accept the difference.

Design B only:

- **The common parent is not exited.** `idle` -> `cooking` must not send
  `Exit` or `Entry` to `door_closed`. Neither has an action here, so the log
  will not show the mistake. Add a temporary `println!` to check.
- **Exit from where the machine is, not from the state that handled the
  event.** `door_closed` handles `DoorOpened`, but the exits start at
  `cooking`.
- **A target with children.** A transition to `door_closed` is not finished
  until `idle` has been entered.
- **`Entry` and `Exit` must not return `Tran`.** Only `Event` may.
- **`hsm.rs` must not mention the oven.** If it needs `Oven` or `Event` by
  name, something oven-specific has leaked into the mechanism.

### Test cases

Each test starts a new oven: door closed, `idle`, `remaining` 0, empty log.
Both designs must pass all of them. In design A the final state `cooking` is
`State::DoorClosed(Closed::Cooking)`; in design B it is `State::Cooking`.

| Events                                              | Expected log                         | Final state | Checks                      |
|-----------------------------------------------------|--------------------------------------|-------------|-----------------------------|
| `AddTime(2), Start, Tick, Tick`                     | `heater on, beep, heater off`        | `idle`      | full cook                   |
| `Start`                                             | (empty)                              | `idle`      | guard: no time set          |
| `AddTime(5), Start, DoorOpened`                     | `heater on, heater off, lamp on`     | `door_open` | inherited from `cooking`    |
| `AddTime(5), Start, Stop, DoorOpened`               | `heater on, heater off, lamp on`     | `door_open` | inherited from `paused`     |
| `DoorOpened, DoorClosed`                            | `lamp on, lamp off`                  | `idle`      | inherited from `idle`       |
| `DoorOpened, AddTime(5), DoorClosed, Start`         | `lamp on, lamp off`                  | `idle`      | `AddTime` ignored when open |
| `AddTime(5), Start, Stop, Stop`                     | `heater on, heater off`              | `idle`      | second stop clears the time |

After step 3 (history):

| Events                                              | Expected log                                    | Final state | `remaining` |
|-----------------------------------------------------|-------------------------------------------------|-------------|-------------|
| `AddTime(5), Start, DoorOpened, DoorClosed`         | `heater on, heater off, lamp on, lamp off, heater on` | `cooking` | 5       |
| `AddTime(5), Start, Tick, Stop, DoorOpened, DoorClosed` | `heater on, heater off, lamp on, lamp off`  | `paused`    | 4           |

Before step 3 both of these end in `idle`, and the first has no second
`heater on`.

### Steps

Design A, in `src/bin/oven/`:

- [ ] **A1: inherited transitions.** The two enums and `dispatch`. No entry
      or exit actions yet. Test the final states.
- [ ] **A2: entry and exit actions.** Add `enter` and `exit`. Test the logs
      in the first table.
- [ ] **A3: history.** Add `last_closed`. Test the second table.

Design B, in `src/bin/v4/`:

- [x] **B1: mechanism without actions.** Write `hsm.rs` with `Reply` and
      the `Machine` trait. `tran` only changes the
      current state. Write `oven.rs` on top of it. Test the final states.
- [x] **B2: entry and exit actions.** Add `Signal` and the full transition
      algorithm. Test the logs in the first table.
- [x] **B3: history.** Add the `history` field. Test the second table.
- [x] **B4: prove it is reusable.** Put a second machine on `hsm.rs` without
      editing it. Done with the digital watch in v5.

### Stretch goals

- [ ] **Safe resume.** A real microwave does not restart when the door
      closes. If the history is `cooking`, go to `paused` instead.
- [ ] **Lamp while cooking.** The lamp is also on in `cooking`. Notice that
      this fits badly: the lamp is shared by two states with no common parent.
- [ ] **Pause timeout.** After 60 ticks in `paused`, clear the time and go to
      `idle`.
- [ ] **Self-transition.** Add an event that leaves and re-enters `cooking`
      (exit and entry actions both run) and make the design B algorithm handle a
      target equal to the current state.
- [ ] **Convert the calculator** to the `hsm.rs` mechanism: an `on` parent handles
      `Clear` and `Off` once.

## Digital watch in detail

The second machine on the shared mechanism (`src/hsm.rs`). Implemented in
`src/bin/v5/`. Run it with `cargo run --bin v5`.

The tables below write state names as `showing_time`. In the code they are
variants of the `State` enum: `State::ShowingTime`, and so on.

### Goal

Model a digital watch with three buttons (Mode, Set, Adjust) and a
one-second tick. It shows the time or the day of the week, and the time can
be set. The clock itself is ordinary data: `hours`, `minutes`, `seconds` and
`day`.

### What it adds to the oven

- **Three levels of nesting.** The oven had two.
- **A child overrides its parent.** `watch` handles `Tick` for every state,
  but `setting` handles `Tick` first and does nothing. The clock stands still
  while it is being set.
- **A parent with entry and exit actions.** `setting` turns blinking on and
  off. Moving from hours to minutes stays inside `setting`, so `blink on`
  must not run again. The log shows it if the mechanism gets this wrong, which
  the oven's log could not.
- **A second user of the mechanism.** `src/hsm.rs` was not edited for the
  watch.

### Events

```rust
enum Event {
    Mode,   // mode button
    Set,    // set button
    Adjust, // adjust button
    Tick,   // one second has passed
}
```

### Extended state

- `hours`, `minutes`, `seconds: u32` and `day: usize` (0 = Mon) - the clock.
- `log: Vec<&'static str>` - what the watch did to its display, for the tests.
- `last_display: State` - the child of `displaying` that was last active.

### States

```text
watch                      starts in displaying
+-- displaying             starts in showing_time
|   +-- showing_time
|   +-- showing_day
+-- setting                starts in setting_hours
    +-- setting_hours
    +-- setting_minutes
```

| State             | Parent       | Entry action     | Exit action                  |
|-------------------|--------------|------------------|------------------------------|
| `watch`           | none         |                  |                              |
| `displaying`      | `watch`      |                  |                              |
| `showing_time`    | `displaying` | `show time`      |                              |
| `showing_day`     | `displaying` | `show day`       |                              |
| `setting`         | `watch`      | `blink on`       | `seconds = 0`, `blink off`   |
| `setting_hours`   | `setting`    | `select hours`   |                              |
| `setting_minutes` | `setting`    | `select minutes` |                              |

The watch is always in exactly one of the four innermost states.

### Transitions

| State             | Event    | Action                    | Next state        |
|-------------------|----------|---------------------------|-------------------|
| `watch`           | `Tick`   | advance the clock 1 s     | (stay)            |
| `displaying`      | `Set`    |                           | `setting`         |
| `showing_time`    | `Mode`   |                           | `showing_day`     |
| `showing_day`     | `Mode`   |                           | `showing_time`    |
| `setting`         | `Tick`   | nothing (overrides `watch`) | (stay)          |
| `setting`         | `Mode`   |                           | last display      |
| `setting_hours`   | `Adjust` | `hours = (hours + 1) % 24`     | (stay)       |
| `setting_hours`   | `Set`    |                           | `setting_minutes` |
| `setting_minutes` | `Adjust` | `minutes = (minutes + 1) % 60` | (stay)       |
| `setting_minutes` | `Set`    |                           | last display      |

Anything not listed is ignored, such as `Adjust` while displaying.

"Last display" is `showing_time` or `showing_day`, whichever was showing when
`Set` was pressed.

How one event travels, for `Tick`:

```text
in showing_day:    showing_day? no -> displaying? no -> watch? yes: advance
in setting_hours:  setting_hours? no -> setting? yes: do nothing (watch is never asked)
```

### Test cases

Each test starts a new watch: `Mon 00:00:00`. Starting it logs `show time`,
so every log begins with that.

| Events                              | Expected log (after `show time`)                           | Final state       | Clock          | Checks                           |
|-------------------------------------|-------------------------------------------------------------|-------------------|----------------|----------------------------------|
| (none)                              | (nothing more)                                              | `showing_time`    | `Mon 00:00:00` | starting children are entered    |
| `Mode`                              | `show day`                                                  | `showing_day`     | `Mon 00:00:00` | mode switch                      |
| `Mode, Tick, Tick`                  | `show day`                                                  | `showing_day`     | `Mon 00:00:02` | `Tick` inherited two levels up   |
| `Mode, Set`                         | `show day, blink on, select hours`                          | `setting_hours`   | `Mon 00:00:00` | `Set` inherited, parent entered first |
| `Set, Adjust, Adjust, Set, Adjust, Set` | `blink on, select hours, select minutes, blink off, show time` | `showing_time` | `Mon 02:01:00` | `blink on` runs once         |
| `Tick, Tick, Tick, Set, Tick, Tick` | `blink on, select hours`                                    | `setting_hours`   | `Mon 00:00:03` | `Tick` overridden while setting  |
| `Tick, Tick, Tick, Set, Mode`       | `blink on, select hours, blink off, show time`              | `showing_time`    | `Mon 00:00:00` | exit action zeroes the seconds   |
| `Set, Set, Mode`                    | `blink on, select hours, select minutes, blink off, show time` | `showing_time` | `Mon 00:00:00` | `Mode` inherited from `setting`  |
| `Mode, Set, Set, Set`               | `show day, blink on, select hours, select minutes, blink off, show day` | `showing_day` | `Mon 00:00:00` | history            |
| `Adjust`                            | (nothing more)                                              | `showing_time`    | `Mon 00:00:00` | ignored event                    |

Also tested: `23:59:59` plus one tick gives the next day at `00:00:00`
(Sunday wraps to Monday), and adjusting wraps hours at 24 and minutes at 60.

### Steps

- [x] Move the mechanism to the library: `src/lib.rs` and `src/hsm.rs`.
      Programs import it with `use event_driven_fsm::hsm::...`.
- [x] Write `src/bin/v5/watch.rs` and `main.rs`.
- [x] Turn the test-case table into unit tests.

### Stretch goals

- [ ] **Stopwatch mode.** A third child of `displaying`, with `stopped` and
      `running` children of its own (a fourth level).
- [ ] **Setting the day.** A third child of `setting`.
- [ ] **Setting timeout.** After 10 ticks with no button pressed, `setting`
      returns to the display. Each button press must restart the count.
- [ ] **Alarm.** The alarm must sound whatever the display shows, which one
      state hierarchy cannot express cleanly. Try it as a second machine
      running beside the watch.

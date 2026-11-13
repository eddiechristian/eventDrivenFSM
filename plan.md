# Plan: exercises for event-driven state machines

Source article: [State Machines for Event-Driven Systems](https://barrgroup.com/blog/state-machines-event-driven-systems)

## Done

- [x] Keyboard with a Shift key, v1: no generics (`src/v1.rs`)
- [x] Keyboard with a Shift key, v2: generics, reusable FSM mechanism (`src/v2/`)

## Recommended path

Do these three in order.

- [ ] **Comment stripper** - easy to test: feed it strings, compare the output.
- [ ] **Time bomb** - adds time (tick events) and guard conditions.
- [ ] **Calculator** - by now the duplication across states shows why
      hierarchical state machines exist.

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

## Later

- [ ] Hierarchical state machines (states nested in states), then redo the
      calculator with them and compare.

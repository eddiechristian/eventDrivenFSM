enum Event {
    Char(char), // the next character of the input
    Slash,
    Star,
    End, // no more input
}

type State = fn(&mut File, &Event);

pub struct File {
    state: State,
    /// Everything typed so far (ordinary data the machine carries along).
    pub output: String,
}

impl File {
    /// Start out pointing at the `initial` function. Nothing runs yet.
    ///
    /// C: `FsmCtor_(me, init)`
    pub fn new() -> File {
        File {
            state: File::initial,
            output: String::new(),
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

impl File {
    /// Not a real state: it only exists to get us into `default`.
    fn initial(&mut self, _event: &Event) {
        println!("[initial] File initialized -> default");
        self.tran(File::default);
    }

    fn default(&mut self, event: &Event) {
        match event {
            Event::Slash => {
                self.tran(File::slash);
            }
            Event::Star => {
                self.tran(File::star);
            }
            Event::End => {}
            Event::Char(char) => {
                self.output.push(*char);
                self.tran(File::code);
            }
        }
    }

    fn slash(&mut self, event: &Event) {
        match event {
            Event::Slash => {
                self.tran(File::line_comment);
            }
            Event::Star => {
                self.tran(File::block_comment);
            }
            Event::End => {}
            Event::Char(char) => {
                self.output.push('/');
                self.output.push(*char);
                self.tran(File::code);
            }
        }
    }
    fn code(&mut self, event: &Event) {
        match event {
            Event::Slash => {
                self.tran(File::slash);
            }
            Event::Star => {
                self.tran(File::star);
            }
            Event::End => {}
            Event::Char(char) => {
                self.output.push(*char);
            }
        }
    }
    fn star(&mut self, event: &Event) {
        match event {
            Event::Slash => {
                self.tran(File::code);
            }
            _ => {}
        }
    }
    fn line_comment(&mut self, event: &Event) {
        match event {
            Event::End => {
                self.tran(File::code);
            }
            Event::Char(char) => match *char as u8 {
                b'\n' => {
                    self.output.push(*char);
                    self.tran(File::code);
                }
                b'\r' => {
                    self.output.push(*char);
                    self.tran(File::code);
                }
                _ => {}
            },
            _ => {}
        }
    }
    fn block_comment(&mut self, event: &Event) {
        match event {
            Event::Star => {
                self.tran(File::star);
            }
            _ => {}
        }
    }
}

fn main() {
    let mut file = File::new();

    // `initial` ignores its event, so it does not matter which one we pass.
    file.init(&Event::Char('x'));

    // `a = b / c;`
    /*

    let events = [
        Event::Char('a'),
        Event::Char(' '),
        Event::Char('='),
        Event::Char(' '),
        Event::Char('b'),
        Event::Char(' '),
        Event::Slash,
        Event::Char(' '),
        Event::Char('c'),
        Event::Char(';'),
        Event::End,
    ];
    */
    //   x = 1; // note\ny = 2;
    let events = [
        Event::Char('x'),
        Event::Char(' '),
        Event::Char('='),
        Event::Char(' '),
        Event::Char('1'),
        Event::Char(';'),
        Event::Char(' '),
        Event::Slash,
        Event::Slash,
        Event::Char(' '),
        Event::Char('n'),
        Event::Char('o'),
        Event::Char('t'),
        Event::Char('e'),
        Event::Slash,
        Event::Char('\n'),
        Event::Char('y'),
        Event::Char(' '),
        Event::Char('='),
        Event::Char(' '),
        Event::Char('2'),
        Event::Char(';'),
        Event::Char('\n'),
        Event::End,
    ];

    // The same call every time. What happens depends on the current state.
    for event in &events {
        file.dispatch(event);
    }

    println!("output: {}", file.output);
}


pub type Span = (usize, usize);

pub trait Buffer {
    type Item: PartialEq; //Fixme: do we need PartialEq?
    type State;

    fn next(&mut self) -> Option<Span>;

    fn read(&self, span: Span) -> Option<Self::Item>;

    fn state(&self) -> Self::State;
    fn restore(&mut self, state: Self::State);
}



// Implement Buffer for ascii text
pub struct AsciiBuffer<'a> {
    // a slice of u8 is a buffer for ascii text
    text: &'a [u8],
    state: usize,
}

impl<'a> AsciiBuffer<'a> {
    fn new(text: &'a [u8]) -> Self {
        AsciiBuffer {
            text,
            state: 0,
        }
    }
}

// implement trait Buffer for AsciiBuffer
impl<'a> Buffer for AsciiBuffer<'a> {
    type Item = char;
    type State = usize;

    fn next(&mut self) -> Option<Span> {
        if self.state + 1 < self.text.len() {
            let span = (self.state, self.state + 1);
            self.state += 1;
            Some(span)
        } else {
            None
        }
    }

    fn read(&self, span: Span) -> Option<Self::Item> {
        Some(self.text[span.0] as char)
    }

    fn state(&self) -> Self::State {
        self.state
    }

    fn restore(&mut self, state: Self::State) {
        self.state = state;
    }
}
use unicode_segmentation::GraphemeCursor;
use crate::buffer::{Buffer, Span};


// Implement Buffer for ascii text
pub struct UnicodeBuffer<'a> {
    // a slice of u8 is a buffer for ascii text
    text: &'a str,
    state: GraphemeCursor,
}

impl<'a> UnicodeBuffer<'a> {
    fn new(text: &'a str) -> Self {
        UnicodeBuffer {
            text,
            state: GraphemeCursor::new(0, text.len(), true),
        }
    }
}

// implement trait Buffer for a buffer of unicode text
impl<'a> Buffer for UnicodeBuffer<'a> {
    type Item = char;
    type State = usize;

    fn next(&mut self) -> Option<Span> {
        if let Ok(Some(next)) = self.state.next_boundary(self.text, 0) {
            let span = (self.state.cur_cursor(), next);
            self.state.set_cursor(next);
            Some(span)
        } else {
            None
        }
    }

    fn read(&self, span: Span) -> Option<Self::Item> {
        Some(self.text[span.0..span.1].chars().next().unwrap())
    }

    fn state(&self) -> Self::State {
        self.state.cur_cursor()
    }

    fn restore(&mut self, state: Self::State) {
        self.state.set_cursor(state)
    }
}
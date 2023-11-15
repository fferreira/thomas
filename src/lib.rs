mod peg;
mod streams;
mod buffer;
mod unicode_buffer;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

// The public function for this library is parse
pub use peg::parse;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

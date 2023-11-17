mod peg;

// The public function for this library is parse
pub use peg::parse;

#[cfg(test)]
mod tests {
    use crate::peg::{CST, Grammar, Rule};

    #[test]
    fn parse_a() {
        use super::*;
        let mut input = "a".chars();
        let mut grammar = Grammar::new();
        grammar.insert("a".to_string(), Rule::Terminal('a'));
        let (rest, cst) = parse(&grammar, "a", &mut input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Terminal(&'a')));
    }
}
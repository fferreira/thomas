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
        grammar.insert("A".to_string(), Rule::Terminal('a'));
        let (rest, cst) = parse(&grammar, "A", &mut input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("A".to_string(), Box::new(CST::Terminal(&'a')))));
    }

    #[test]
    fn parse_a_or_b() {
        use super::*;
        let mut input = "b".chars();
        let mut grammar = Grammar::new();
        grammar.insert("AORB".to_string(), Rule::Choice(vec![Rule::Terminal('a'), Rule::Terminal('b')]));
        let (rest, cst) = parse(&grammar, "AORB", &mut input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("AORB".to_string(), Box::new(CST::Terminal(&'b')))));
    }
}
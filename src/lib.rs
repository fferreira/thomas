// The public function for this library is parse
pub use peg::parse;
pub use terminal::unicode::{innit, is_cat};

mod peg;
mod terminal;

#[cfg(test)]
mod tests {
    use crate::peg::{CST, Grammar, Rule};

    #[test]
    fn parse_a() {
        use super::*;
        let input = "a".chars();
        let mut grammar = Grammar::new();
        // Rule to recognise a single 'a'
        grammar.insert("A".to_string(), Rule::Terminal(innit('a')));
        let (rest, cst) = parse(&grammar, "A", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("A".to_string(), Box::new(CST::Terminal('a')))));
    }

    #[test]
    fn parse_a_or_b() {
        use super::*;
        let input = "b".chars();
        let mut grammar = Grammar::new();
        grammar.insert("AORB".to_string(), Rule::Choice(vec![Rule::Terminal(innit('a')), Rule::Terminal(innit( 'b'))]));
        let (rest, cst) = parse(&grammar, "AORB", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("AORB".to_string(), Box::new(CST::Terminal('b')))));
    }

    #[test]
    fn parse_many_a_or_b_zero() {
        use super::*;
        let input = "".chars();
        let mut grammar = Grammar::new();
        grammar.insert("AORB".to_string(), Rule::ZeroOrMore(Box::new(Rule::Choice(vec![Rule::Terminal(innit('a')), Rule::Terminal(innit('b'))]))));
        let (rest, cst) = parse(&grammar, "AORB", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, None);
    }

    #[test]
    fn parse_many_a_or_b_one() {
        use super::*;
        let input = "b".chars();
        let mut grammar = Grammar::new();
        grammar.insert("AORB".to_string(), Rule::ZeroOrMore(Box::new(Rule::Choice(vec![Rule::Terminal(innit('a')), Rule::Terminal(innit('b'))]))));
        let (rest, cst) = parse(&grammar, "AORB", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("AORB".to_string(), Box::new(CST::Terminal('b')))));
    }

    #[test]
    fn parse_many_a_or_b_more() {
        use super::*;
        let input = "bab".chars();
        let mut grammar = Grammar::new();
        grammar.insert("AORB".to_string(), Rule::ZeroOrMore(Box::new(Rule::Choice(vec![Rule::Terminal(innit('a')), Rule::Terminal(innit('b'))]))));
        let (rest, cst) = parse(&grammar, "AORB", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("AORB".to_string(), Box::new(CST::Sequence(vec![CST::Terminal('b'), CST::Terminal('a'), CST::Terminal('b')])))));
    }

    #[test]
    fn parse_number() {
        use super::*;
        let input = "123".chars();
        let mut grammar = Grammar::new();
        grammar.insert("NUMBER".to_string(), Rule::OneOrMore(Box::new(Rule::Terminal(is_cat(unicode_general_category::GeneralCategory::DecimalNumber)))));
        let (rest, cst) = parse(&grammar, "NUMBER", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("NUMBER".to_string(), Box::new(CST::Sequence(vec![CST::Terminal('1'), CST::Terminal('2'), CST::Terminal('3')])))));
    }

    #[test]
    fn parse_zero() {
        use super::*;
        let input = "1".chars();
        let mut grammar = Grammar::new();
        grammar.insert("ZERO".to_string(), Rule::ZeroOrMore(Box::new(Rule::Terminal(innit('0')))));
        let (rest, cst) = parse(&grammar, "ZERO", input).unwrap();
        assert_eq!(rest.clone().next(), Some('1'));
        assert_eq!(cst, None); // when parsing zero or more, the result for an empty input is None (but it succeeds). I'm not completely sure this is the right behaviour.
    }

    //#[test]
    fn left_recursion() {
        use super::*;
        let input = "A".chars();
        let mut grammar = Grammar::new();
        grammar.insert("A".to_string(), Rule::Choice(vec![Rule::NonStream("A".to_string()), Rule::Terminal(innit('a'))]));
        let (rest, cst) = parse(&grammar, "A", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("A".to_string(), Box::new(CST::Terminal('a')))));
    }
}
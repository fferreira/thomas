// The public function for this library is parse
pub use terminal::unicode::{innit, is_cat, is_digit};

mod peg;
mod terminal;

#[cfg(test)]
mod tests {
    use std::str::Chars;
    use crate::peg::{CST, Error, Grammar, Rule};

    #[test]
    fn parse_empty() {
        let input = "x".chars();
        let mut grammar : Grammar<Chars, char> = Grammar::new();
        grammar.insert("EMPTY".to_string(), Rule::Empty);
        let (rest, cst) = grammar.parse("EMPTY", input).unwrap();
        assert_eq!(rest.clone().next(), Some('x'));
        assert_eq!(cst, None);
    }

    #[test]
    fn parse_empty_then_a() {
        use super::*;
        let input = "a".chars();
        let mut grammar = Grammar::new();
        grammar.insert("A".to_string(), Rule::Sequence(vec![Rule::Empty, Rule::Terminal(innit('a'))]));
        let (rest, cst) = grammar.parse("A", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("A".to_string(), Box::new(CST::Terminal('a')))));
    }

    #[test]
    fn parse_a() {
        use super::*;
        let input = "a".chars();
        let mut grammar = Grammar::new();
        // Rule to recognise a single 'a'
        grammar.insert("A".to_string(), Rule::Terminal(innit('a')));
        let (rest, cst) = grammar.parse("A", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("A".to_string(), Box::new(CST::Terminal('a')))));
    }

    #[test]
    fn parse_a_or_b() {
        use super::*;
        let input = "b".chars();
        let mut grammar = Grammar::new();
        grammar.insert("AORB".to_string(), Rule::Choice(vec![Rule::Terminal(innit('a')), Rule::Terminal(innit('b'))]));
        let (rest, cst) = grammar.parse("AORB", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("AORB".to_string(), Box::new(CST::Terminal('b')))));
    }

    #[test]
    fn parse_optional_not_there() {
        use super::*;
        let input = "b".chars();
        let mut grammar = Grammar::new();
        grammar.insert("OPTIONAL".to_string(), Rule::Sequence(vec![Rule::Optional(Box::new(Rule::Terminal(innit('a')))), Rule::Terminal(innit('b'))]));
        let (rest, cst) = grammar.parse("OPTIONAL", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("OPTIONAL".to_string(), Box::new(CST::Terminal('b')))));
    }

    #[test]
    fn parse_optional_and_there() {
        use super::*;
        let input = "ab".chars();
        let mut grammar = Grammar::new();
        grammar.insert("OPTIONAL".to_string(), Rule::Sequence(vec![Rule::Optional(Box::new(Rule::Terminal(innit('a')))), Rule::Terminal(innit('b'))]));
        let (rest, cst) = grammar.parse("OPTIONAL", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("OPTIONAL".to_string(), Box::new(CST::Sequence(vec![CST::Terminal('a'), CST::Terminal('b')])))));
    }

    #[test]
    fn parse_many_a_or_b_zero() {
        use super::*;
        let input = "".chars();
        let mut grammar = Grammar::new();
        grammar.insert("AORB".to_string(), Rule::ZeroOrMore(Box::new(Rule::Choice(vec![Rule::Terminal(innit('a')), Rule::Terminal(innit('b'))]))));
        let (rest, cst) = grammar.parse("AORB", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, None);
    }

    #[test]
    fn parse_many_a_or_b_one() {
        use super::*;
        let input = "b".chars();
        let mut grammar = Grammar::new();
        grammar.insert("AORB".to_string(), Rule::ZeroOrMore(Box::new(Rule::Choice(vec![Rule::Terminal(innit('a')), Rule::Terminal(innit('b'))]))));
        let (rest, cst) = grammar.parse("AORB", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("AORB".to_string(), Box::new(CST::Terminal('b')))));
    }

    #[test]
    fn parse_many_a_or_b_more() {
        use super::*;
        let input = "bab".chars();
        let mut grammar = Grammar::new();
        grammar.insert("AORB".to_string(), Rule::ZeroOrMore(Box::new(Rule::Choice(vec![Rule::Terminal(innit('a')), Rule::Terminal(innit('b'))]))));
        let (rest, cst) = grammar.parse("AORB", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("AORB".to_string(), Box::new(CST::Sequence(vec![CST::Terminal('b'), CST::Terminal('a'), CST::Terminal('b')])))));
    }

    #[test]
    fn parse_number() {
        use super::*;
        let input = "123".chars();
        let mut grammar = Grammar::new();
        grammar.insert("NUMBER".to_string(), Rule::OneOrMore(Box::new(Rule::Terminal(is_cat(unicode_general_category::GeneralCategory::DecimalNumber)))));
        let (rest, cst) = grammar.parse("NUMBER", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("NUMBER".to_string(), Box::new(CST::Sequence(vec![CST::Terminal('1'), CST::Terminal('2'), CST::Terminal('3')])))));
    }

    #[test]
    fn parse_zero() {
        use super::*;
        let input = "1".chars();
        let mut grammar = Grammar::new();
        grammar.insert("ZERO".to_string(), Rule::ZeroOrMore(Box::new(Rule::Terminal(innit('0')))));
        let (rest, cst) = grammar.parse("ZERO", input).unwrap();
        assert_eq!(rest.clone().next(), Some('1'));
        assert_eq!(cst, None); // when parsing zero or more, the result for an empty input is None (but it succeeds). I'm not completely sure this is the right behaviour.
    }

    #[test]
    fn test_memoization() {
        use super::*;
        let input = "acb".chars();
        let mut grammar = Grammar::new();
        grammar.insert("A".to_string(), Rule::Terminal(innit('a')));
        grammar.insert("B".to_string(), Rule::Terminal(innit('b')));
        grammar.insert("C".to_string(), Rule::Terminal(innit('c')));
        grammar.insert("ABC".to_string(), Rule::Sequence(vec![Rule::NonStream("A".to_string()), Rule::NonStream("B".to_string()), Rule::NonStream("C".to_string())]));
        grammar.insert("ACB".to_string(), Rule::Sequence(vec![Rule::NonStream("A".to_string()), Rule::NonStream("C".to_string()), Rule::NonStream("B".to_string())]));
        grammar.insert("ABCorACB".to_string(), Rule::Choice(vec![Rule::NonStream("ABC".to_string()), Rule::NonStream("ACB".to_string())]));

        let (rest, cst) = grammar.parse("ABCorACB", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst,
                   Some(CST::Node("ABCorACB".to_string(),
                                  Box::new(CST::Node("ACB".to_string(),
                                                     Box::new(CST::Sequence(vec![CST::Node("A".to_string(), Box::new(CST::Terminal('a'))),
                                                                                 CST::Node("C".to_string(), Box::new(CST::Terminal('c'))),
                                                                                 CST::Node("B".to_string(), Box::new(CST::Terminal('b')))])))))));
    }

    #[test]
    fn and_predicate_test_succeed(){
        use super::*;
        let input = "2".chars();
        let mut grammar = Grammar::new();
        grammar.insert("TWO".to_string(), Rule::Terminal(innit('2')));
        grammar.insert("DIGIT".to_string(), Rule::Terminal(is_digit()));
        // parses any digit as long as it is a 2
        grammar.insert("PREDICATE".to_string(), Rule::Sequence(vec![Rule::AndPredicate(Box::new(Rule::NonStream("TWO".to_string()))), Rule::NonStream("DIGIT".to_string())]));
        let (rest, cst) = grammar.parse("PREDICATE", input).unwrap();
        assert_eq!(rest.clone().next(), None);
        assert_eq!(cst, Some(CST::Node("PREDICATE".to_string(), Box::new(CST::Node("DIGIT".to_string(), Box::new(CST::Terminal('2')))))));
    }

    #[test]
    fn and_predicate_test_fail(){
        use super::*;
        let input = "3".chars();
        let mut grammar = Grammar::new();
        grammar.insert("TWO".to_string(), Rule::Terminal(innit('2')));
        grammar.insert("DIGIT".to_string(), Rule::Terminal(is_digit()));
        // parses any digit as long as it is a 2
        grammar.insert("PREDICATE".to_string(), Rule::Sequence(vec![Rule::AndPredicate(Box::new(Rule::NonStream("TWO".to_string()))), Rule::NonStream("DIGIT".to_string())]));
        let res = grammar.parse("PREDICATE", input);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::CannotMatchStreamItem);
    }

    #[test]
    fn left_recursion() {
        use super::*;
        let input = "A".chars();
        let mut grammar = Grammar::new();
        grammar.insert("A".to_string(), Rule::Choice(vec![Rule::NonStream("A".to_string()), Rule::Terminal(innit('a'))]));
        let res = grammar.parse("A", input);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), Error::CannotFindValidChoice);
        // let (rest, cst) = parse(&grammar, "A", input).unwrap();
        // assert_eq!(rest.clone().next(), None);
        // assert_eq!(cst, Some(CST::Node("A".to_string(), Box::new(CST::Terminal('a')))));
    }
}
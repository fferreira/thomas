use std::collections::HashMap;


// A trait for Streams

pub trait Stream {
    type Item : PartialEq + std::fmt::Debug;
    fn next(&self) -> Option<(&Self, &Self::Item)>;
    fn peek(&self) -> Option<&Self::Item>;
    fn drop(& self) -> Option<&Self>;
}

// A PEG rule
#[derive(Debug, Clone)]
pub enum Rule<S: Stream> {
    Empty,
    Terminal(S::Item),
    NonStream(String),
    Sequence(Vec<Rule<S>>),
    Choice(Vec<Rule<S>>),
    Optional(Box<Rule<S>>),
    ZeroOrMore(Box<Rule<S>>),
    OneOrMore(Box<Rule<S>>),
    AndPredicate(Box<Rule<S>>),
}

// The type of a PEG grammar is a map from rule names to rules
pub type Grammar<S> = HashMap<String, Rule<S>>;

// The concrete syntax tree
#[derive(Debug, Clone)]
pub enum CST <'a, S : Stream>  {
    Empty,
    Terminal(&'a S::Item),
    Node(String, Box<CST<'a, S>>),
    Sequence(Vec<CST<'a, S>>),
}

// Parser errors
#[derive(Debug, Clone)]
pub enum Error {
    CannotFindRule(String),
    CannotParseStream,
    CannotFindValidChoice,
    CannotMatchStreamItem,
    ViolationCannotDropStreamItem,
    Unexpected,
}

fn zero_or_more<'a, S: Stream>(grammar: &'a Grammar<S>, rule: &'a Rule<S>, input: &'a S) -> Result<(&'a S, Vec<CST<'a, S>>), Error> {
    let mut rest = input;
    let mut cst = Vec::new();
    loop {
        match parse_rule(grammar, rule, rest) {
            Ok((new_rest, new_cst)) => {
                rest = new_rest;
                cst.push(new_cst);
            }
            Err(_) => break,
        }
    }
    Ok((rest, cst))
}

// Parse a rule
pub fn parse_rule<'a, S: Stream>(grammar: &'a Grammar<S>, rule: &'a Rule<S>, input: &'a S) -> Result<(&'a S, CST<'a, S>), Error> {
    match rule {
        Rule::Empty => Ok((input, CST::Empty)),

        Rule::Terminal(t) => {
            match input.next() {
                Some((rest, item)) => {
                    if item == t {
                        Ok((rest, CST::Terminal(t)))
                    } else {
                        Err(Error::Unexpected)
                    }
                }
                None => Err(Error::CannotParseStream),
            }
        }
        Rule::NonStream(name) => {
            let rule = grammar.get(name).ok_or(Error::CannotFindRule(name.to_string()))?;
            let (rest, res) = parse_rule(grammar, rule, input)?;
            Ok((rest, CST::Node(name.to_string(), Box::new(res))))
        }
        Rule::Sequence(rules) => {
            let mut rest = input;
            let mut seq_node = Vec::new();
            for rule in rules {
                let (new_rest, item) = parse_rule(grammar, rule, rest)?;
                rest = new_rest;
                seq_node.push(item);
            }
            Ok((rest, CST::Sequence(seq_node)))
        }
        Rule::Choice(rules) => {
            for rule in rules {
                match parse_rule(grammar, rule, input) {
                    Ok((rest, cst)) => return Ok((rest, cst)),
                    Err(_) => (),
                }
            }
            Err(Error::CannotFindValidChoice)
        }
        Rule::Optional(rule) => {
            match parse_rule(grammar, rule, input) {
                Ok((rest, cst)) => Ok((rest, cst)),
                Err(_) => Ok((input, CST::Empty)),
            }
        }
        Rule::ZeroOrMore(rule) => {
            let (rest, cst) = zero_or_more(grammar, rule, input)?;
            Ok((rest, CST::Sequence(cst)))
        }
        Rule::OneOrMore(rule) => {
            let (rest, cst) = zero_or_more(grammar, rule, input)?;
            if cst.len() == 0 {
                Err(Error::CannotMatchStreamItem)
            } else {
                Ok((rest, CST::Sequence(cst)))
            }
        }
        Rule::AndPredicate(rule) => {
            match parse_rule(grammar, rule, input) {
                Ok((_, _)) => Ok((input, CST::Empty)),
                Err(err) => Err(err),
            }
        }
    }
}

// Parse using a Grammar
pub fn parse<'a, S: Stream>(grammar: &'a Grammar<S>, rule: &str, input: &'a S) -> Result<(&'a S, CST<'a, S>), Error> {
    let rule = grammar.get(rule).ok_or(Error::CannotFindRule(rule.to_string()))?;
    let (rest, cst) = parse_rule(grammar, rule, input)?;
    Ok((rest, cst))
}

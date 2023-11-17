use std::collections::HashMap;

// A PEG rule
#[derive(Debug, Clone)]
pub enum Rule<I> {
    Empty,
    Terminal(I),
    NonStream(String),
    Sequence(Vec<Rule<I>>),
    Choice(Vec<Rule<I>>),
    Optional(Box<Rule<I>>),
    ZeroOrMore(Box<Rule<I>>),
    OneOrMore(Box<Rule<I>>),
    AndPredicate(Box<Rule<I>>),
}

// The type of a PEG grammar is a map from rule names to rules
pub type Grammar<S> = HashMap<String, Rule<S>>;

// The concrete syntax tree
#[derive(Debug, Clone, PartialEq)]
pub enum CST<'a, S> {
    Terminal(&'a S),
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
    EmptyNonOptionalParserResult,
    Unexpected,
}

type ParserResult<'a, I, E> = Result<(I, Option<CST<'a, E>>), Error>;
type ParserResultMany<'a, I, E> = Result<(I, Vec<CST<'a, E>>), Error>;

fn zero_or_more<'a, I: Iterator + Clone>(grammar: &'a Grammar<I::Item>, rule: &'a Rule<I::Item>, input: &mut I) -> ParserResultMany<'a, I, I::Item>
where
    I::Item: PartialEq,
{
    let mut cst = Vec::new();
    loop {

        match parse_rule(grammar, rule, input) {
            Ok((rest, cst_op)) => {
                *input = rest;
                match cst_op {
                    Some(cst_op) => cst.push(cst_op),
                    None => break,
                }
            }
            Err(_) => break,
        }
    }
    Ok((input.clone(), cst))
}

// Parse a rule
pub fn parse_rule<'a, I : Iterator + Clone>(grammar: &'a Grammar<I::Item>, rule: &'a Rule<I::Item>, input: &mut I) -> ParserResult<'a, I, I::Item>
where
    I::Item: PartialEq,
{
    match rule {
        Rule::Empty => Ok((input.clone(), None)),

        Rule::Terminal(t) => {
            match input.next() {
                Some(item) => {
                    if item == *t {
                        Ok((input.clone(), Some(CST::Terminal(t))))
                    } else {
                        Err(Error::CannotMatchStreamItem)
                    }
                }
                None => Err(Error::CannotParseStream),
            }
        }
        Rule::NonStream(name) => {
            let rule = grammar.get(name).ok_or(Error::CannotFindRule(name.to_string()))?;
            let (rest, res) = parse_rule(grammar, rule, input)?;
            Ok((rest, Some(CST::Node(name.to_string(), Box::new(res.ok_or(Error::EmptyNonOptionalParserResult)?)))))
        }
        Rule::Sequence(rules) => {
            let mut seq_node = Vec::new();
            for rule in rules {
                let (rest, item_op) = parse_rule(grammar, rule, input)?;
                *input = rest;
                match item_op {
                    Some(item_op) => seq_node.push(item_op),
                    None => break,
                }
            }
            Ok((input.clone(), Some(CST::Sequence(seq_node))))
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
                Ok((rest, cst_op)) => Ok((rest, cst_op)),
                Err(_) => Ok((input.clone(), None)),
            }
        }
        Rule::ZeroOrMore(rule) => {
            let (rest, cst) = zero_or_more(grammar, rule, input)?;
            Ok((rest, Some(CST::Sequence(cst))))
        }
        Rule::OneOrMore(rule) => {
            let (rest, cst) = zero_or_more(grammar, rule, input)?;
            if cst.len() == 0 {
                Err(Error::CannotMatchStreamItem)
            } else {
                Ok((rest, Some(CST::Sequence(cst))))
            }
        }
        Rule::AndPredicate(rule) => {
            match parse_rule(grammar, rule, input) {
                Ok((_, _)) => Ok((input.clone(), None)),
                Err(err) => Err(err),
            }
        }
    }
}

// Parse using a Grammar
pub fn parse<'a, I: Iterator + Clone>(grammar: &'a Grammar<I::Item>, rule: &str, input: &mut I) -> ParserResult<'a, I, I::Item>
where
    I::Item: PartialEq,
{
    let rule = grammar.get(rule).ok_or(Error::CannotFindRule(rule.to_string()))?;
    let (rest, cst) = parse_rule(grammar, rule, input)?;
    Ok((rest, cst))
}


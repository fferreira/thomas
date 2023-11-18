use std::collections::HashMap;

// A PEG rule
//#[derive(Debug, Clone)]
pub enum Rule<I, O> {
    Empty,
    Terminal(Box<dyn Fn(&I) -> Option<O>>),
    NonStream(String),
    Sequence(Vec<Rule<I, O>>),
    Choice(Vec<Rule<I, O>>),
    Optional(Box<Rule<I, O>>),
    ZeroOrMore(Box<Rule<I, O>>),
    OneOrMore(Box<Rule<I, O>>),
    AndPredicate(Box<Rule<I, O>>),
}

// The type of a PEG grammar is a map from rule names to rules
pub type Grammar<I, O> = HashMap<String, Rule<I, O>>;

// The concrete syntax tree
#[derive(Debug, Clone, PartialEq)]
pub enum CST<O> {
    Terminal(O),
    Node(String, Box<CST<O>>),
    Sequence(Vec<CST<O>>),
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

type ParserResult<'a, I, E> = Result<(I, Option<CST<E>>), Error>;
type ParserResultMany<'a, I, E> = Result<(I, Vec<CST<E>>), Error>;

fn zero_or_more<'a, I: Iterator + Clone, O : Clone>(grammar: &'a Grammar<I::Item, O>, rule: &'a Rule<I::Item, O>, input: &mut I) -> ParserResultMany<'a, I, O>
    where I::Item: Clone, {
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
pub fn parse_rule<'a, I: Iterator + Clone, O : Clone>(grammar: &'a Grammar<I::Item, O>, rule: &'a Rule<I::Item, O>, input: &mut I) -> ParserResult<'a, I, O>
    where I::Item: Clone {
    match rule {
        Rule::Empty => Ok((input.clone(), None)),

        Rule::Terminal(f) => {
            match input.next() {
                Some(item) => {
                    match f(&item) {
                        Some(t) => Ok((input.clone(), Some(CST::Terminal(t)))),
                        None => return Err(Error::CannotMatchStreamItem),
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
                let ni = &mut input.clone();
                match parse_rule(grammar, rule, ni) {
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
            let (rest, csts) = zero_or_more(grammar, rule, input)?;
            match csts.len() {
                0 => Ok((rest, None)),
                1 => Ok((rest, Some(csts[0].clone()))),
                _ => Ok((rest, Some(CST::Sequence(csts)))),
            }
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
pub fn parse<'a, I: Iterator + Clone, O : Clone>(grammar: &'a Grammar<I::Item, O>, rule_name: &str, input: &mut I) -> ParserResult<'a, I, O>
    where I::Item: Clone {
    let rule = grammar.get(rule_name).ok_or(Error::CannotFindRule(rule_name.to_string()))?;
    let (rest, cst) = parse_rule(grammar, rule, input)?;
    match cst {
        Some(cst) => Ok((rest, Some(CST::Node(rule_name.to_string(), Box::new(cst))))),
        None => Ok((rest, None)),
    }
}


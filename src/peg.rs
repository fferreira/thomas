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
pub struct Grammar<I, O> {
    table: HashMap<String, Rule<I, O>>,
}

impl<I, O> Grammar<I, O> {
    pub fn new() -> Grammar<I, O> {
        Grammar {
            table: HashMap::new(),
        }
    }
    pub fn insert(&mut self, name: String, rule: Rule<I, O>) {
        self.table.insert(name, rule);
    }
    pub fn get(&self, name: &str) -> Result<&Rule<I, O>, Error> {
        self.table.get(name).ok_or(Error::CannotFindRule(name.to_string()))
    }
}

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

type ParserOutput<I, E> = (ParserInput<I>, Option<CST<E>>);
type ParserResult<I, E> = Result<ParserOutput<I, E>, Error>;
type ParserOutputMany<I, E> = (ParserInput<I>, Vec<CST<E>>);
type ParserResultMany<I, E> = Result<ParserOutputMany<I, E>, Error>;

// Packrat style memoization table
#[derive(Debug, Clone)]
pub struct MemoTable<I, O> {
    table: HashMap<(String, usize), ParserOutput<I, O>>,
}

// Parser input
#[derive(Debug, Clone)]
pub struct ParserInput<I> {
    input: I,
    pos: usize,
}

impl<I: Iterator> ParserInput<I> {
    fn new(input: I) -> ParserInput<I> {
        ParserInput {
            input,
            pos: 0,
        }
    }

    pub fn next(&mut self) -> Option<I::Item> {
        let item = self.input.next();
        if item.is_some() {
            self.pos += 1;
        }
        item
    }

    fn pos(&self) -> usize {
        self.pos
    }
}

fn zero_or_more<I: Iterator + Clone, O: Clone>(grammar: &Grammar<I::Item, O>, memo: &mut MemoTable<I, O>, rule: &Rule<I::Item, O>, input: &mut ParserInput<I>) -> ParserResultMany<I, O>
    where I::Item: Clone, {
    let mut cst = Vec::new();
    loop {
        match parse_rule(grammar, memo, rule, input) {
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
pub fn parse_rule<I: Iterator + Clone, O: Clone>(grammar: &Grammar<I::Item, O>, memo: &mut MemoTable<I, O>, rule: &Rule<I::Item, O>, input: &mut ParserInput<I>) -> ParserResult<I, O>
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
            let (rest, res) = apply_rule(grammar, memo, name.as_str(), input)?;
            Ok((rest, Some(CST::Node(name.to_string(), Box::new(res.ok_or(Error::EmptyNonOptionalParserResult)?)))))
        }
        Rule::Sequence(rules) => {
            let mut seq_node = Vec::new();
            for rule in rules {
                let (rest, item_op) = parse_rule(grammar, memo, rule, input)?;
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
                match parse_rule(grammar, memo, rule, ni) {
                    Ok((rest, cst)) => return Ok((rest, cst)),
                    Err(_) => (),
                }
            }
            Err(Error::CannotFindValidChoice)
        }
        Rule::Optional(rule) => {
            match parse_rule(grammar, memo, rule, input) {
                Ok((rest, cst_op)) => Ok((rest, cst_op)),
                Err(_) => Ok((input.clone(), None)),
            }
        }
        Rule::ZeroOrMore(rule) => {
            let orig = input.clone();
            let (rest, csts) = zero_or_more(grammar, memo, rule, input)?;
            match csts.len() {
                0 => Ok((orig, None)),
                1 => Ok((rest, Some(csts[0].clone()))),
                _ => Ok((rest, Some(CST::Sequence(csts)))),
            }
        }
        Rule::OneOrMore(rule) => {
            let (rest, cst) = zero_or_more(grammar, memo, rule, input)?;
            if cst.len() == 0 {
                Err(Error::CannotMatchStreamItem)
            } else {
                Ok((rest, Some(CST::Sequence(cst))))
            }
        }
        Rule::AndPredicate(rule) => {
            match parse_rule(grammar, memo, rule, input) {
                Ok((_, _)) => Ok((input.clone(), None)),
                Err(err) => Err(err),
            }
        }
    }
}

fn apply_rule<'a, I, O>(grammar: &Grammar<I::Item, O>, memo: &mut MemoTable<I, O>, rule_name: &str, input: &mut ParserInput<I>) -> ParserResult<I, O>
    where I: Iterator + Clone, I::Item: Clone, O: Clone {
    if let Some(result) = memo.table.get(&(rule_name.into(), input.pos())) {
        return Ok(result.clone());
    }
    let (rest, cst) = parse_rule(grammar, memo, grammar.get(&rule_name)?, input)?;
    memo.table.insert((rule_name.into(), input.pos()), (rest.clone(), cst.clone()));
    Ok((rest, cst))
}


// Parse using a Grammar
pub fn parse<I: Iterator + Clone, O: Clone>(grammar: &Grammar<I::Item, O>, rule_name: &str, input: I) -> ParserResult<I, O>
    where I::Item: Clone, I: Clone {
    let mut memo = MemoTable {
        table: HashMap::new(),
    };

    let (rest, cst) = apply_rule(grammar, &mut memo, rule_name.into(), &mut ParserInput::new(input))?;
    match cst {
        Some(cst) => Ok((rest, Some(CST::Node(rule_name.to_string(), Box::new(cst))))),
        None => Ok((rest, None)),
    }
}


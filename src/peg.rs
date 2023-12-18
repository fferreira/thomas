use std::collections::HashMap;

// A PEG rule
//#[derive(Debug, Clone)]
pub enum Rule<It, O> {
    Empty,
    Terminal(Box<dyn Fn(&It) -> Option<O>>),
    NonStream(String),
    Sequence(Vec<Rule<It, O>>),
    Choice(Vec<Rule<It, O>>),
    Optional(Box<Rule<It, O>>),
    ZeroOrMore(Box<Rule<It, O>>),
    OneOrMore(Box<Rule<It, O>>),
    AndPredicate(Box<Rule<It, O>>),
}

// The type of a PEG grammar is a map from rule names to rules
pub struct Grammar<I, O> where
    I: Iterator + Clone
{
    table: HashMap<String, Rule<I::Item, O>>,
}



// The concrete syntax tree
#[derive(Debug, Clone, PartialEq)]
pub enum CST<O> {
    Terminal(O),
    Node(String, Box<CST<O>>),
    Sequence(Vec<CST<O>>),
}

// Parser errors
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    CannotFindRule(String),
    CannotParseStream,
    CannotFindValidChoice,
    CannotMatchStreamItem,
    EmptyNonOptionalParserResult,
    LeftRecursionNotSupported,
    Unexpected,
}

type ParserOutput<I, E> = (ParserInput<I>, Option<CST<E>>);
type ParserResult<I, E> = Result<ParserOutput<I, E>, Error>;
type ParserOutputMany<I, E> = (ParserInput<I>, Vec<CST<E>>);
type ParserResultMany<I, E> = Result<ParserOutputMany<I, E>, Error>;

// Packrat style memoization table
#[derive(Debug, Clone)]
struct MemoTable<I, O> {
    table: HashMap<(String, usize), ParserResult<I, O>>,
}

impl <I, O> MemoTable<I, O> {
    fn new() -> MemoTable<I, O> {
        MemoTable {
            table: HashMap::new(),
        }
    }

    fn insert(&mut self, key: (String, usize), value: ParserResult<I, O>) {
        self.table.insert(key, value);
    }

    fn get(&self, key: &(String, usize)) -> Option<&ParserResult<I, O>> {
        self.table.get(key)
    }
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

// The implementation of the PEG parser
impl<I, O> Grammar<I, O>
    where
        I: Iterator + Clone,
        I::Item: Clone,
        O: Clone {
    pub fn new() -> Grammar<I, O> {
        Grammar {
            table: HashMap::new(),
        }
    }
    pub fn insert(&mut self, name: String, rule: Rule<I::Item, O>) {
        self.table.insert(name, rule);
    }
    fn get(&self, name: &str) -> Result<&Rule<I::Item, O>, Error> {
        self.table.get(name).ok_or(Error::CannotFindRule(name.to_string()))
    }

    // Parse using a Grammar
    pub fn parse(&self, rule_name: &str, input: I) -> ParserResult<I, O> {
        let mut memo = MemoTable::new();

        let (rest, cst) = self.apply_rule(&mut memo, rule_name.into(), &mut ParserInput::new(input))?;
        match cst {
            Some(cst) => Ok((rest, Some(CST::Node(rule_name.to_string(), Box::new(cst))))),
            None => Ok((rest, None)),
        }
    }

    // applies a rule by name, using memoization
    fn apply_rule(&self, memo: &mut MemoTable<I, O>, rule_name: &str, input: &mut ParserInput<I>) -> ParserResult<I, O>
        where I: Iterator + Clone, I::Item: Clone, O: Clone {
        if let Some(result) = memo.get(&(rule_name.into(), input.pos())) {
            return result.clone();
        }
        let pos = input.pos();
        memo.insert((rule_name.into(), pos), Err(Error::LeftRecursionNotSupported));
        let (rest, cst) = self.eval_rule(memo, self.get(rule_name)?, input)?;
        memo.insert((rule_name.into(), pos), Ok((rest.clone(), cst.clone())));
        Ok((rest, cst))
    }

    fn zero_or_more(&self, memo: &mut MemoTable<I, O>, rule: &Rule<I::Item, O>, input: &mut ParserInput<I>) -> ParserResultMany<I, O>
        where I::Item: Clone, {
        let mut cst = Vec::new();
        loop {
            match self.eval_rule(memo, rule, input) {
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

    // evaluates parsing the body of a rule
    fn eval_rule(&self, memo: &mut MemoTable<I, O>, rule: &Rule<I::Item, O>, input: &mut ParserInput<I>) -> ParserResult<I, O>
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
                let (rest, res) = self.apply_rule(memo, name.as_str(), input)?;
                Ok((rest, Some(CST::Node(name.to_string(), Box::new(res.ok_or(Error::EmptyNonOptionalParserResult)?)))))
            }
            Rule::Sequence(rules) => {
                let mut seq_node = Vec::new();
                for rule in rules {
                    let (rest, item_op) = self.eval_rule(memo, rule, input)?;
                    *input = rest;
                    match item_op {
                        Some(item_op) => seq_node.push(item_op),
                        None => (),
                    }
                }
                match seq_node.len() { // This optimisation may prove to be a mistake. Consider having sequences of length 1, and even 0.
                    0 => Ok((input.clone(), None)),
                    1 => Ok((input.clone(), Some(seq_node[0].clone()))),
                    _ => Ok((input.clone(), Some(CST::Sequence(seq_node))))
                }
            }
            Rule::Choice(rules) => {
                for rule in rules {
                    let ni = &mut input.clone();
                    match self.eval_rule(memo, rule, ni) {
                        Ok((rest, cst)) => return Ok((rest, cst)),
                        Err(_) => (),
                    }
                }
                Err(Error::CannotFindValidChoice)
            }
            Rule::Optional(rule) => {
                let orig = input.clone();
                match self.eval_rule(memo, rule, input) {
                    Ok((rest, cst_op)) => Ok((rest, cst_op)),
                    Err(_) => Ok((orig, None)),
                }
            }
            Rule::ZeroOrMore(rule) => {
                let orig = input.clone();
                let (rest, csts) = self.zero_or_more(memo, rule, input)?;
                match csts.len() {
                    0 => Ok((orig, None)),
                    1 => Ok((rest, Some(csts[0].clone()))),
                    _ => Ok((rest, Some(CST::Sequence(csts)))),
                }
            }
            Rule::OneOrMore(rule) => {
                let (rest, cst) = self.zero_or_more(memo, rule, input)?;
                if cst.len() == 0 {
                    Err(Error::CannotMatchStreamItem)
                } else {
                    Ok((rest, Some(CST::Sequence(cst))))
                }
            }
            Rule::AndPredicate(rule) => {
                let orig = input.clone();
                match self.eval_rule(memo, rule, input) {
                    Ok((_, _)) => Ok((orig, None)),
                    Err(err) => Err(err),
                }
            }
        }
    }
}


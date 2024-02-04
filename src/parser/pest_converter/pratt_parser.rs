use std::collections::BTreeMap;
use std::iter::Peekable;
use std::ops::BitOr;

use pest::iterators::Pair;
use pest::RuleType;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Assoc {
    Left,
    Right,
}

type Prec = u32;
const PREC_STEP: Prec = 10;

pub struct Op<R: RuleType> {
    rule: R,
    affix: Affix,
    next: Option<Box<Op<R>>>,
}

impl<R: RuleType> Op<R> {
    pub fn prefix(rule: R) -> Self {
        Self {
            rule,
            affix: Affix::Prefix,
            next: None,
        }
    }

    pub fn postfix(rule: R) -> Self {
        Self {
            rule,
            affix: Affix::Postfix,
            next: None,
        }
    }

    pub fn infix(rule: R, assoc: Assoc) -> Self {
        Self {
            rule,
            affix: Affix::Infix(assoc),
            next: None,
        }
    }
}

impl<R: RuleType> BitOr for Op<R> {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self {
        fn assign_next<R: RuleType>(op: &mut Op<R>, next: Op<R>) {
            if let Some(ref mut child) = op.next {
                assign_next(child, next);
            } else {
                op.next = Some(Box::new(next));
            }
        }

        assign_next(&mut self, rhs);
        self
    }
}

enum Affix {
    Prefix,
    Postfix,
    Infix(Assoc),
}

pub struct PrattConfig<R: RuleType> {
    prec: Prec,
    ops: BTreeMap<R, (Affix, Prec)>,
    has_prefix: bool,
    has_postfix: bool,
    has_infix: bool,
}

impl<R: RuleType> Default for PrattConfig<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: RuleType> PrattConfig<R> {
    pub fn new() -> Self {
        Self {
            prec: PREC_STEP,
            ops: BTreeMap::new(),
            has_prefix: false,
            has_postfix: false,
            has_infix: false,
        }
    }

    pub fn op(mut self, op: Op<R>) -> Self {
        self.prec += PREC_STEP;
        let mut iter = Some(op);
        while let Some(Op { rule, affix, next }) = iter.take() {
            match affix {
                Affix::Prefix => self.has_prefix = true,
                Affix::Postfix => self.has_postfix = true,
                Affix::Infix(_) => self.has_infix = true,
            }
            self.ops.insert(rule, (affix, self.prec));
            iter = next.map(|op| *op);
        }
        self
    }
}

pub(crate) trait PrattContext<'pratt, 'i, R, T>
where
    R: RuleType + 'pratt,
{
    fn config(&self) -> &PrattConfig<R>;

    fn map_primary(&mut self, pair: Pair<'i, R>) -> T;

    fn map_prefix(&mut self, _pair: Pair<'i, R>, _rhs: T) -> T {
        unreachable!();
    }
    fn map_postfix(&mut self, _lhs: T, _pair: Pair<'i, R>) -> T {
        unreachable!();
    }
    fn map_infix(&mut self, _lhs: T, _pair: Pair<'i, R>, _rhs: T) -> T {
        unreachable!();
    }
}

pub(crate) trait PrattParsing<'pratt, 'i, R, T>
where
    R: RuleType + 'pratt,
{
    fn pratt_parse<P: Iterator<Item = Pair<'i, R>>>(&mut self, pairs: P) -> T;
}

impl<'pratt, 'i, R, T, C> PrattParsing<'pratt, 'i, R, T> for C
where
    R: RuleType + 'pratt,
    C: PrattContext<'pratt, 'i, R, T>,
{
    fn pratt_parse<P: Iterator<Item = Pair<'i, R>>>(&mut self, pairs: P) -> T {
        expr(self, &mut pairs.peekable(), 0)
    }
}

fn expr<'pratt, 'i, C, T, R, P>(context: &mut C, pairs: &mut Peekable<P>, rbp: Prec) -> T
where
    C: PrattContext<'pratt, 'i, R, T>,
    P: Iterator<Item = Pair<'i, R>>,
    R: RuleType + 'pratt,
{
    let mut lhs = nud(context, pairs);
    while rbp < lbp(pairs, context.config()) {
        lhs = led(context, pairs, lhs);
    }
    lhs
}

/// Null-Denotation
///
/// "the action that should happen when the symbol is encountered
///  as start of an expression (most notably, prefix operators)
fn nud<'pratt, 'i, C, T, R, P>(context: &mut C, pairs: &mut Peekable<P>) -> T
where
    C: PrattContext<'pratt, 'i, R, T>,
    P: Iterator<Item = Pair<'i, R>>,
    R: RuleType + 'pratt,
{
    let pair = pairs.next().expect("Pratt parsing expects non-empty Pairs");
    match context.config().ops.get(&pair.as_rule()) {
        Some((Affix::Prefix, prec)) => {
            let rhs = expr(context, pairs, *prec - 1);
            context.map_prefix(pair, rhs)
        }
        None => context.map_primary(pair),
        _ => panic!("Expected prefix or primary expression, found {}", pair),
    }
}

/// Left-Denotation
///
/// "the action that should happen when the symbol is encountered
/// after the start of an expression (most notably, infix and postfix operators)"
fn led<'pratt, 'i, C, T, R, P>(context: &mut C, pairs: &mut Peekable<P>, lhs: T) -> T
where
    C: PrattContext<'pratt, 'i, R, T>,
    P: Iterator<Item = Pair<'i, R>>,
    R: RuleType + 'pratt,
{
    let pair = pairs.next().unwrap();
    match context.config().ops.get(&pair.as_rule()) {
        Some((Affix::Infix(assoc), prec)) => {
            let rhs = match *assoc {
                Assoc::Left => expr(context, pairs, *prec),
                Assoc::Right => expr(context, pairs, *prec - 1),
            };
            context.map_infix(lhs, pair, rhs)
        }
        Some((Affix::Postfix, _)) => context.map_postfix(lhs, pair),
        _ => panic!("Expected postfix or infix expression, found {}", pair),
    }
}

/// Left-Binding-Power
///
/// "describes the symbol's precedence in infix form (most notably, operator precedence)"
fn lbp<'pratt, 'i, R, P>(pairs: &mut Peekable<P>, config: &PrattConfig<R>) -> Prec
where
    P: Iterator<Item = Pair<'i, R>>,
    R: RuleType + 'pratt,
{
    match pairs.peek() {
        Some(pair) => match config.ops.get(&pair.as_rule()) {
            Some((_, prec)) => *prec,
            None => panic!("Expected operator, found {}", pair),
        },
        None => 0,
    }
}

pub mod ifrpn;
pub mod irpn;
pub mod ivrpn;
pub mod lrpn;
pub mod rpn;

use std::borrow::{Borrow, Cow};
use std::hash::Hash;
use std::iter::Peekable;
use std::marker::PhantomData;
use std::str::CharIndices;

use smallvec::{SmallVec, smallvec};

use crate::resolver::{Locked, LockedResolver, ResolverState};
use crate::{
    ConstantResolver, DefaultResolver, EmptyResolver, Error, ExprFn, SmallResolver,
    context::Context, op::Op, prelude::*,
};
use crate::{parsing, LContext, ParseError, Ptr};

/// Represents the compiled expression.
///
/// This struct should be built by using the [`Expr::compile`] method. This
/// method automatically returns the best available representation of the expression
/// depending on the context you will be using it with.
///
/// After compilation, the expression can be evaluated using the [`Expr::eval`] method.
#[derive(Debug, PartialEq)]
pub struct Expr<Token>
{
    tokens: Vec<Token>,
}

impl<Token> Expr<Token>
{
    pub fn len(&self) -> usize
    {
        self.tokens.len()
    }
}

trait NotIndexedResolver {}
impl<S: ResolverState, K: Borrow<str> + PartialEq<String> + Eq + Hash, T> NotIndexedResolver
    for DefaultResolver<S, K, T>
{
}
impl<S: ResolverState, T> NotIndexedResolver for ConstantResolver<S, T> {}
impl<S: ResolverState, K: AsRef<str> + Eq, T> NotIndexedResolver for SmallResolver<S, K, T> {}
impl<S: ResolverState> NotIndexedResolver for EmptyResolver<S> {}

pub trait ExprCompiler<'e, 'c, S, V, F, LV, LF, T>
where
    S: ResolverState,
{
    fn compile(expr: &'e str, ctx: &'c Context<S, V, F, LV, LF>) -> Result<Expr<T>, Error<'e>>;
}

pub trait ExprEvaluator<'e, S, V, F, LV, LF>
where
    S: ResolverState,
{
    fn eval(&self, ctx: &Context<S, V, F, LV, LF>, stack: &mut Vec<f64>) -> Result<f64, Error<'e>>;
}

impl<'e, T> TryFrom<&'e str> for Expr<T>
where
    T: From<f64> + From<&'e str> + From<Op> + From<(&'e str, usize)>,
{
    type Error = crate::Error<'e>;

    fn try_from(input: &'e str) -> Result<Self, Self::Error>
    {
        let infix_expr = Expr::<Infix>::try_from(input)?;
        Expr::<T>::try_from(infix_expr)
    }
}

impl<'e, T, V, F> TryFrom<(&'e str, &'e LContext<V, F>)> for Expr<T>
where
    T: From<f64> + From<Ptr<'e, f64>> + From<Op> + From<(Ptr<'e, ExprFn>, usize)>,
    V: Resolver<Locked, f64> + LockedResolver<f64>,
    F: Resolver<Locked, ExprFn> + LockedResolver<ExprFn>,
{
    type Error = crate::Error<'e>;

    fn try_from((input, ctx): (&'e str, &'e LContext<V, F>)) -> Result<Self, Self::Error>
    {
        let infix_expr = Expr::<Infix>::try_from(input)?;
        Expr::<T>::try_from((infix_expr, ctx))
    }
}

impl<'e, T, V, F> TryFrom<(Expr<Infix<'e>>, &'e LContext<V, F>)> for Expr<T>
where
    T: From<f64> + From<Ptr<'e, f64>> + From<Op> + From<(Ptr<'e, ExprFn>, usize)>,
    V: Resolver<Locked, f64> + LockedResolver<f64>,
    F: Resolver<Locked, ExprFn> + LockedResolver<ExprFn>,
{
    type Error = Error<'e>;

    // shunting yard algorithm
    fn try_from((expr, ctx): (Expr<Infix<'e>>, &'e LContext<V, F>)) -> Result<Self, Self::Error>
    {
        let mut f64_cache: SmallVec<[f64; 4]> = smallvec![];
        let mut output: Vec<T> = Vec::with_capacity(expr.len());
        let mut ops: Vec<Infix> = Vec::new();

        for tok in expr.into_iter() {
            match tok {
                Infix::Num(num) => {
                    output.push(num.into());
                    f64_cache.push(num);
                }
                Infix::Var(name) => {
                    let var_ptr = ctx
                        .vars()
                        .get_ptr(name)
                        .ok_or_else(|| Error::UnknownVar(Cow::Borrowed(name)))?;
                    output.push(T::from(var_ptr));
                    f64_cache.clear();
                }
                Infix::Op(op) => {
                    while let Some(Infix::Op(top)) = ops.last() {
                        let prec = op.precedence();
                        let top_prec = top.precedence();
                        let should_pop =
                            top_prec > prec || (!op.is_right_associative() && top_prec == prec);

                        if should_pop {
                            if let Some(Infix::Op(op)) = ops.pop() {
                                pre_evaluate(&mut output, &mut f64_cache, op);
                            }
                        } else {
                            break;
                        }
                    }
                    ops.push(Infix::Op(op));
                }
                Infix::LParen => ops.push(tok),
                Infix::RParen => {
                    while let Some(top) = ops.pop() {
                        match top {
                            Infix::LParen => break,
                            Infix::Op(op) => pre_evaluate(&mut output, &mut f64_cache, op),
                            _ => unreachable!("no more elements should be inside ops"),
                        }
                    }
                }
                Infix::Fn(name, args) => {
                    let fn_ptr = ctx
                        .fns()
                        .get_ptr(name)
                        .ok_or_else(|| Error::UnknownFn(Cow::Borrowed(name)))?;
                    let fn_token = T::from((fn_ptr, args.len()));

                    for arg_tokens in args {
                        let rpn_arg: Expr<T> = Expr::try_from((arg_tokens, ctx))?;
                        output.extend(rpn_arg.tokens);
                    }

                    output.push(fn_token);
                    f64_cache.clear();
                }
            }
        }

        while let Some(Infix::Op(op)) = ops.pop() {
            pre_evaluate(&mut output, &mut f64_cache, op);
        }

        debug_assert!(ops.is_empty());

        return Ok(Expr { tokens: output });

        // const folding
        fn pre_evaluate<'e, T>(output: &mut Vec<T>, f64_cache: &mut SmallVec<[f64; 4]>, op: Op)
        where
            T: From<f64> + From<Op>,
        {
            let n_operands = op.num_operands();

            if f64_cache.len() >= n_operands {
                let output_len = output.len();
                let f64_cache_len = f64_cache.len();

                let start = f64_cache_len - n_operands;
                let num = op.apply(&f64_cache[start..]);

                let token: T = num.into();

                output.truncate(output_len - n_operands);
                output.push(token);

                f64_cache.truncate(f64_cache_len - n_operands);
                f64_cache.push(num);
            } else {
                let token: T = op.into();
                output.push(token);
                f64_cache.clear();
            }
        }
    }
}

impl<'e, T> TryFrom<Expr<Infix<'e>>> for Expr<T>
where
    T: From<f64> + From<&'e str> + From<Op> + From<(&'e str, usize)>,
{
    type Error = Error<'e>;

    // shunting yard algorithm
    fn try_from(expr: Expr<Infix<'e>>) -> Result<Self, Self::Error>
    {
        let mut f64_cache: SmallVec<[f64; 4]> = smallvec![];
        let mut output: Vec<T> = Vec::with_capacity(expr.len());
        let mut ops: Vec<Infix> = Vec::new();

        for tok in expr.into_iter() {
            match tok {
                Infix::Num(num) => {
                    output.push(num.into());
                    f64_cache.push(num);
                }
                Infix::Var(name) => {
                    output.push(name.into());
                    f64_cache.clear();
                }
                Infix::Op(op) => {
                    while let Some(Infix::Op(top)) = ops.last() {
                        let prec = op.precedence();
                        let top_prec = top.precedence();
                        let should_pop =
                            top_prec > prec || (!op.is_right_associative() && top_prec == prec);

                        if should_pop {
                            if let Some(Infix::Op(op)) = ops.pop() {
                                pre_evaluate(&mut output, &mut f64_cache, op);
                            }
                        } else {
                            break;
                        }
                    }
                    ops.push(Infix::Op(op));
                }
                Infix::LParen => ops.push(tok),
                Infix::RParen => {
                    while let Some(top) = ops.pop() {
                        match top {
                            Infix::LParen => break,
                            Infix::Op(op) => pre_evaluate(&mut output, &mut f64_cache, op),
                            _ => unreachable!("no more elements should be inside ops"),
                        }
                    }
                }
                Infix::Fn(name, args) => {
                    let fn_token = T::from((name, args.len()));

                    for arg_tokens in args {
                        let rpn_arg: Expr<T> = arg_tokens.try_into()?;
                        output.extend(rpn_arg.tokens);
                    }

                    output.push(fn_token);
                    f64_cache.clear();
                }
            }
        }

        while let Some(Infix::Op(op)) = ops.pop() {
            pre_evaluate(&mut output, &mut f64_cache, op);
        }

        debug_assert!(ops.is_empty());

        return Ok(Expr { tokens: output });

        // const folding
        fn pre_evaluate<'e, T>(output: &mut Vec<T>, f64_cache: &mut SmallVec<[f64; 4]>, op: Op)
        where
            T: From<f64> + From<Op>,
        {
            let n_operands = op.num_operands();

            if f64_cache.len() >= n_operands {
                let output_len = output.len();
                let f64_cache_len = f64_cache.len();

                let start = f64_cache_len - n_operands;
                let num = op.apply(&f64_cache[start..]);

                let token: T = num.into();

                output.truncate(output_len - n_operands);
                output.push(token);

                f64_cache.truncate(f64_cache_len - n_operands);
                f64_cache.push(num);
            } else {
                let token: T = op.into();
                output.push(token);
                f64_cache.clear();
            }
        }
    }
}

// TODO: Move to a different module (lexer.rs)

enum Infix
{
    Op(Op),
    LParen,
    RParen,
}

struct LexData<'e>
{
    input: &'e str,
    chars: Peekable<CharIndices<'e>>,
}

struct LexBuffers<T>
{
    f64_cache: SmallVec<[f64; 4]>,
    output: Vec<T>,
    ops: Vec<Infix>,
}

struct Lexer<'e, 'c, T, S: 'c, V: 'c, F: 'c, LV: 'c, LF: 'c>
where
    T: From<f64>
        + From<(&'e str, Option<&'c Context<S, V, F, LV, LF>>)>
        + From<Op>
        + From<(&'e str, usize, Option<&'c Context<S, V, F, LV, LF>>)>,
    S: ResolverState,
{
    data: LexData<'e>,
    buffers: LexBuffers<T>,
    state: State,

    _ctx_lifetime: PhantomData<&'c ()>,
    _ctx_state: PhantomData<S>,
    _ctx_var: PhantomData<V>,
    _ctx_fn: PhantomData<F>,
    _ctx_lk_var: PhantomData<LV>,
    _ctx_lk_fn: PhantomData<LF>,
}

impl<'e, 'c, T, S: 'c, V: 'c, F: 'c, LV: 'c, LF: 'c> Lexer<'e, 'c, T, S, V, F, LV, LF>
where
    T: From<f64>
        + From<(&'e str, Option<&'c Context<S, V, F, LV, LF>>)>
        + From<Op>
        + From<(&'e str, usize, Option<&'c Context<S, V, F, LV, LF>>)>,
    S: ResolverState,
{
    fn new(expr: &'e str) -> Self
    {
        Lexer {
            data: LexData {
                input: expr,
                chars: expr.char_indices().peekable(),
            },
            
            state: State::Default,
            
            buffers: LexBuffers {
                f64_cache: smallvec![],
                output: Vec::with_capacity(expr.len() / 2),
                ops: Vec::new(),
            },
            
            _ctx_lifetime: PhantomData,
            _ctx_state: PhantomData,
            _ctx_var: PhantomData,
            _ctx_fn: PhantomData,
            _ctx_lk_var: PhantomData,
            _ctx_lk_fn: PhantomData,
        }
    }

    fn lex(&mut self, ctx: Option<&Context<S, V, F, LV, LF>>) -> Result<Vec<T>, Error<'e>>
    {
        let chars = &mut self.data.chars;
        let input = self.data.input;

        while let Some((i, c)) = chars.next() {
            match c {
                ' ' | '\t' | '\n' => {
                    // Ignore whitespace
                }
                '(' | '[' => {
                    self.buffers.ops.push(Infix::LParen);
                }
                ')' | ']' => {
                    while let Some(top) = self.buffers.ops.pop() {
                        match top {
                            Infix::LParen => break,
                            Infix::Op(op) => pre_evaluate(&mut self.buffers, op),
                            _ => unreachable!("no more elements should be inside ops"),
                        }
                    }
                }
                _ => {
                    let next_state = self.state.lex(self, ctx, i, c)?;
                    self.state = next_state;
                }
            }
        }

        while let Some(Infix::Op(op)) = self.buffers.ops.pop() {
            pre_evaluate(&mut self.buffers, op);
        }
        
        debug_assert!(self.buffers.ops.is_empty());
        
        Ok(self.buffers.clone())
    }
}

enum State
{
    Default,
    ExpectingOperator,
}

impl State
{
    #[inline(always)]
    fn lex<'e, 'c, T, S, V, F, LV, LF>(
        &mut self,
        lexer: &mut Lexer<'e, 'c, T, S, V, F, LV, LF>,
        ctx: Option<&'c Context<S, V, F, LV, LF>>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    where 
        S: ResolverState,
        T: From<f64>
            + From<(&'e str, Option<&'c Context<S, V, F, LV, LF>>)>
            + From<Op>
            + From<(&'e str, usize, Option<&'c Context<S, V, F, LV, LF>>)>,
    {
        match self {
            State::ExpectingOperator => Self::handle_expecting_operator(lexer, i, c),
            State::Default => Self::handle_default(lexer, ctx, i, c),
        }
    }

    #[inline(always)]
    fn handle_expecting_operator<'e, 'c, T, S, V, F, LV, LF>(
        lexer: &mut Lexer<'e, 'c, T, S, V, F, LV, LF>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    where 
        S: ResolverState,
        T: From<f64>
            + From<(&'e str, Option<&'c Context<S, V, F, LV, LF>>)>
            + From<Op>
            + From<(&'e str, usize, Option<&'c Context<S, V, F, LV, LF>>)>,
    {
        let op = match c {
            '+' => Op::Add,
            '-' => Op::Sub,
            '*' => Op::Mul,
            '/' => Op::Div,
            '^' => Op::Pow,
            '%' => Op::Mod,
            _ => {
                return Err(Error::ParseError(ParseError::UnexpectedChar(
                    Cow::Owned(c),
                    i,
                )));
            }
        };

        process_operator(&mut lexer.buffers, op);
        
        Ok(State::Default)
    }

    #[inline(always)]
    fn handle_default<'e, 'c, T, S, V, F, LV, LF>(
        lexer: &mut Lexer<'e, 'c, T, S, V, F, LV, LF>,
        ctx: Option<&'c Context<S, V, F, LV, LF>>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    where 
        S: ResolverState,
        T: From<f64>
            + From<(&'e str, Option<&'c Context<S, V, F, LV, LF>>)>
            + From<Op>
            + From<(&'e str, usize, Option<&'c Context<S, V, F, LV, LF>>)>,
    {
        return match c {
            '-' => {
                process_operator(&mut lexer.buffers, Op::Neg);
                Ok(State::Default)
            }
            // numbers
            '0'..='9' | '.' => {
                let num = parsing::parse_uf64(c, &mut lexer.data.chars);

                lexer.buffers.output.push(T::from(num));
                lexer.buffers.f64_cache.push(num);
                
                Ok(State::ExpectingOperator)
            }

            // identifiers (variables or functions)
            'a'..='z' | 'A'..='Z' | '_' => {
                let start_index = i;
                let end_index = lexer.data.input.len();

                let token = loop {
                    if let Some(&(i, d)) = lexer.data.chars.peek() {
                        if d.is_alphanumeric() || d == '_' {
                            lexer.data.chars.next();
                            continue;
                        }

                        // function found, parse functions using the same lexer
                        if d == '(' || d == '[' {
                            let fn_name = &lexer.data.input[start_index..i];
                            lexer.data.chars.next();

                            let mut params = Vec::with_capacity(2);

                            let mut depth = 1;
                            let mut start_index = i + 1; // Skipping the opening bracket of the function call
                            let mut end_index = lexer.data.input.len();

                            while let Some((i, d)) = lexer.data.chars.next() {
                                match d {
                                    '(' | '[' => depth += 1,
                                    ')' | ']' => depth -= 1,
                                    // If depth is greater than 1, it is a parameter separator of a nested function call
                                    ',' if depth == 1 => {
                                        let param_expr =
                                            Expr::<Infix>::try_from(&lexer.data.input[start_index..i])?;
                                        params.push(param_expr);
                                        start_index = i + 1;
                                    }
                                    _ => {}
                                }

                                if depth == 0 {
                                    end_index = i;
                                    break;
                                }
                            }

                            if depth != 0 {
                                return Err(Error::ParseError(ParseError::UnmatchedParentheses(i)));
                            }

                            let param_expr =
                                Expr::<Infix>::try_from(&lexer.data.input[start_index..end_index])?;
                            params.push(param_expr);

                            Infix::Fn(fn_name, params);
                        }

                        break T::from((&lexer.data.input[start_index..i], ctx))
                    } else {
                        break T::from((&lexer.data.input[start_index..end_index], ctx))
                    }
                };

                lexer.buffers.output.push(token);
                lexer.buffers.f64_cache.clear();
                Ok(State::ExpectingOperator)
            }

            _ => Err(Error::ParseError(ParseError::UnexpectedChar(
                Cow::Owned(c),
                i,
            ))),
        };
    }
}

fn process_operator<T>(buffers: &mut LexBuffers<T>, op: Op)
where
    T: From<f64> + From<Op>,
{
    while let Some(Infix::Op(top)) = buffers.ops.last() {
        let prec = op.precedence();
        let top_prec = top.precedence();
        let should_pop =
            top_prec > prec || (!op.is_right_associative() && top_prec == prec);
        
        if should_pop {
            if let Some(Infix::Op(op)) = buffers.ops.pop() {
                pre_evaluate(buffers, op);
            }
        } else {
            break;
        }
    }
    buffers.ops.push(Infix::Op(op));
}

fn pre_evaluate<'e, T>(buffers: &mut LexBuffers<T>, op: Op)
where
    T: From<f64> + From<Op>,
{
    let n_operands = op.num_operands();

    if buffers.f64_cache.len() >= n_operands {
        let output_len = buffers.output.len();
        let f64_cache_len = buffers.f64_cache.len();

        let start = f64_cache_len - n_operands;
        let num = op.apply(&buffers.f64_cache[start..]);

        let token: T = num.into();

        buffers.output.truncate(output_len - n_operands);
        buffers.output.push(token);

        buffers.f64_cache.truncate(f64_cache_len - n_operands);
        buffers.f64_cache.push(num);
    } else {
        let token: T = op.into();
        buffers.output.push(token);
        buffers.f64_cache.clear();
    }
}
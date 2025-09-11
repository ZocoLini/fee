use std::{borrow::Cow, iter::Peekable, str::CharIndices};

use smallvec::{SmallVec, smallvec};

use crate::{
    expr::{ParseableToken, Op}, parsing, prelude::*, resolver::{LockedResolver, ResolverState}, Error, ParseError
};

enum Infix
{
    Op(Op),
    /// Left parenthesis and the number of commas before it
    LParen(usize),
    /// Function and the start and end indices of the function name
    Fn(usize, usize),
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

struct Lexer<'e>
{
    data: LexData<'e>,

    state: State,
}

impl<'e, 'c, T, S, V, F, LV, LF> TryFrom<(&'e str, &'c Context<S, V, F, LV, LF>)> for Expr<T>
where
    T: ParseableToken<'e, 'c, S, V, F, LV, LF> + Copy,
    S: ResolverState,
    V: Resolver<S, f64>,
    F: Resolver<S, ExprFn>,
    LV: Resolver<Locked, f64> + LockedResolver<f64>,
    LF: Resolver<Locked, ExprFn> + LockedResolver<ExprFn>,
{
    type Error = crate::Error<'e>;

    fn try_from((input, ctx): (&'e str, &'c Context<S, V, F, LV, LF>))
    -> Result<Self, Self::Error>
    {
        let mut buffers = LexBuffers {
            f64_cache: smallvec![],
            output: Vec::with_capacity(input.len() / 2),
            ops: Vec::new(),
        };

        let tokens = Lexer::new(input).lex(&mut buffers, ctx)?;

        Ok(Expr { tokens })
    }
}

impl<'e> Lexer<'e>
{
    fn new(expr: &'e str) -> Self
    {
        Lexer {
            data: LexData {
                input: expr,
                chars: expr.char_indices().peekable(),
            },

            state: State::Default,
        }
    }

    fn lex<'c, T, S, V, F, LV, LF>(
        mut self,
        buffers: &mut LexBuffers<T>,
        ctx: &'c Context<S, V, F, LV, LF>,
    ) -> Result<Vec<T>, Error<'e>>
    where
        S: ResolverState,
        T: ParseableToken<'e, 'c, S, V, F, LV, LF> + Copy,
    {
        let mut comma_count = 0;

        while let Some((i, c)) = self.data.chars.next() {
            match c {
                ' ' | '\t' | '\n' => {
                    // Ignore whitespace
                }
                '(' | '[' => {
                    buffers.ops.push(Infix::LParen(comma_count));
                }
                ')' | ']' => {
                    while let Some(top) = buffers.ops.pop() {
                        match top {
                            Infix::LParen(commas) => {
                                if let Some(Infix::Fn(start, end)) = buffers.ops.last() {
                                    let fn_token = T::fun(
                                        &self.data.input[*start..*end],
                                        comma_count - commas + 1,
                                        ctx,
                                    );
                                    
                                    buffers.f64_cache.clear();
                                    buffers.output.push(fn_token);
                                    
                                    comma_count = commas;
                                    buffers.ops.pop();
                                }

                                break;
                            }
                            Infix::Op(op) => pre_evaluate(buffers, op),
                            Infix::Fn(_, _) => {
                                panic!("fn token popped while unfolding after rparen")
                            }
                        }
                    }
                }
                ',' => {
                    comma_count += 1;
                    self.state = State::Default;

                    while let Some(top) = buffers.ops.last() {
                        match top {
                            Infix::LParen(_) => break,
                            Infix::Op(op) => {
                                let op = *op;
                                buffers.ops.pop();
                                pre_evaluate(buffers, op);
                            }
                            Infix::Fn(_, _) => {
                                panic!("fn token popped while unfolding after comma")
                            }
                        }
                    }
                }
                _ => {
                    let next_state = self.state.lex(&mut self.data, buffers, ctx, i, c)?;
                    self.state = next_state;
                }
            }
        }

        while let Some(Infix::Op(op)) = buffers.ops.pop() {
            pre_evaluate(buffers, op);
        }

        debug_assert!(buffers.ops.is_empty());

        Ok(buffers.output.clone())
    }
}

enum State
{
    Default,
    ExpectingOperator,
}

impl State
{
    #[inline]
    fn lex<'e, 'c, T, S, V, F, LV, LF>(
        &mut self,
        data: &mut LexData<'e>,
        buffers: &mut LexBuffers<T>,
        ctx: &'c Context<S, V, F, LV, LF>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    where
        S: ResolverState,
        T: ParseableToken<'e, 'c, S, V, F, LV, LF>
    {
        match self {
            State::ExpectingOperator => Self::handle_expecting_operator(data, buffers, i, c),
            State::Default => Self::handle_default(data, buffers, ctx, i, c),
        }
    }

    #[inline]
    fn handle_expecting_operator<'e, 'c, T, S: 'c, V: 'c, F: 'c, LV: 'c, LF: 'c>(
        _data: &mut LexData<'e>,
        buffers: &mut LexBuffers<T>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    where
        S: ResolverState,
        T: ParseableToken<'e, 'c, S, V, F, LV, LF>
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

        process_operator(buffers, op);

        Ok(State::Default)
    }

    #[inline]
    fn handle_default<'e, 'c, T, S, V, F, LV, LF>(
        data: &mut LexData<'e>,
        buffers: &mut LexBuffers<T>,
        ctx: &'c Context<S, V, F, LV, LF>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    where
        S: ResolverState,
        T: ParseableToken<'e, 'c, S, V, F, LV, LF>
    {
        return match c {
            '-' => {
                process_operator(buffers, Op::Neg);
                Ok(State::Default)
            }
            // numbers
            '0'..='9' | '.' => {
                let num = parsing::parse_uf64(c, &mut data.chars);

                buffers.output.push(T::num(num));
                buffers.f64_cache.push(num);

                Ok(State::ExpectingOperator)
            }

            // identifiers (variables or functions)
            'a'..='z' | 'A'..='Z' | '_' => {
                let start_index = i;
                let end_index = data.input.len();

                let token = loop {
                    if let Some(&(i, d)) = data.chars.peek() {
                        if d.is_alphanumeric() || d == '_' {
                            data.chars.next();
                            continue;
                        }

                        // function found
                        if d == '(' || d == '[' {
                            buffers.ops.push(Infix::Fn(start_index, i));
                            return Ok(State::Default);
                        }

                        break T::var(&data.input[start_index..i], ctx);
                    } else {
                        break T::var(&data.input[start_index..end_index], ctx);
                    }
                };

                buffers.output.push(token);
                buffers.f64_cache.clear();
                Ok(State::ExpectingOperator)
            }

            _ => Err(Error::ParseError(ParseError::UnexpectedChar(
                Cow::Owned(c),
                i,
            ))),
        };
    }
}

#[inline]
fn process_operator<'e, 'c, T, S, V, F, LV, LF>(buffers: &mut LexBuffers<T>, op: Op)
where
    S: ResolverState,
    T: ParseableToken<'e, 'c, S, V, F, LV, LF>
{
    while let Some(Infix::Op(top)) = buffers.ops.last() {
        let prec = op.precedence();
        let top_prec = top.precedence();
        let should_pop = top_prec > prec || (!op.is_right_associative() && top_prec == prec);

        if should_pop {
            pre_evaluate(buffers, *top);
            buffers.ops.pop();
        } else {
            break;
        }
    }
    buffers.ops.push(Infix::Op(op));
}

#[inline]
fn pre_evaluate<'e, 'c, T, S, V, F, LV, LF>(buffers: &mut LexBuffers<T>, op: Op)
where
    S: ResolverState,
    T: ParseableToken<'e, 'c, S, V, F, LV, LF>
{
    let n_operands = op.num_operands();

    if buffers.f64_cache.len() >= n_operands {
        let output_len = buffers.output.len();
        let f64_cache_len = buffers.f64_cache.len();

        let start = f64_cache_len - n_operands;
        let args = unsafe { buffers.f64_cache.get_unchecked(start..) };
        let num = op.apply(args);

        let token: T = T::num(num);

        buffers.output.truncate(output_len - n_operands);
        buffers.output.push(token);

        buffers.f64_cache.truncate(f64_cache_len - n_operands);
        buffers.f64_cache.push(num);
    } else {
        let token: T = T::op(op);
        buffers.output.push(token);
        buffers.f64_cache.clear();
    }
}

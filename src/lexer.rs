use std::{borrow::Cow, iter::Peekable, ops::Deref, str::CharIndices};

use crate::{
    token::{InfixToken, Op},
    *,
};

#[derive(Debug, PartialEq)]
pub struct Infix;

#[derive(Debug, PartialEq)]
pub struct InfixExpr<'e>
{
    tokens: Vec<InfixToken<'e>>,
}

impl<'e> InfixExpr<'e>
{
    pub fn new(tokens: Vec<InfixToken<'e>>) -> Self
    {
        Self { tokens }
    }
}

impl<'e> Deref for InfixExpr<'e>
{
    type Target = Vec<InfixToken<'e>>;

    fn deref(&self) -> &Self::Target
    {
        &self.tokens
    }
}

impl<'e> IntoIterator for InfixExpr<'e>
{
    type Item = InfixToken<'e>;
    type IntoIter = std::vec::IntoIter<InfixToken<'e>>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.tokens.into_iter()
    }
}

impl<'e> TryFrom<&'e str> for InfixExpr<'e>
{
    type Error = crate::Error<'e>;

    fn try_from(input: &'e str) -> Result<Self, Self::Error>
    {
        let mut tokens: Vec<InfixToken<'e>> = Vec::new();
        let mut lexer = Lexer::new(input);

        while let Some(token) = lexer.next_token()? {
            tokens.push(token);
        }

        Ok(InfixExpr { tokens })
    }
}

struct Lexer<'e>
{
    input: &'e str,
    chars: Peekable<CharIndices<'e>>,
    state: State,
}

impl<'e> Lexer<'e>
{
    fn new(expr: &'e str) -> Self
    {
        Lexer {
            input: expr,
            chars: expr.char_indices().peekable(),
            state: State::ExpectingNumberProducer,
        }
    }

    fn next_token(&mut self) -> Result<Option<InfixToken<'e>>, Error<'e>>
    {
        let chars = &mut self.chars;
        let input = self.input;

        while let Some((i, c)) = chars.next() {
            match c {
                ' ' | '\t' | '\n' => {
                    // Ignore whitespace
                }
                '(' | '[' => {
                    return Ok(Some(InfixToken::LParen));
                }
                ')' | ']' => {
                    return Ok(Some(InfixToken::RParen));
                }
                _ => {
                    let (result, next_state) =
                        self.state.next_token(self.input, &mut self.chars, i, c);
                    self.state = next_state;
                    return result.map(|tok| Some(tok));
                }
            }
        }

        Ok(None)
    }
}

enum State
{
    ExpectingOperator,
    ExpectingNumberProducer,
    AfterError,
}

impl State
{
    fn next_token<'e>(
        &mut self,
        input: &'e str,
        chars: &mut Peekable<CharIndices<'e>>,
        i: usize,
        c: char,
    ) -> (Result<InfixToken<'e>, Error<'e>>, State)
    {
        match self {
            State::ExpectingOperator => Self::handle_expecting_operator(input, chars, i, c),
            State::ExpectingNumberProducer => Self::handle_number_or_ident(input, chars, i, c),
            State::AfterError => panic!("tried to continue parsing after error"),
        }
    }

    fn handle_expecting_operator<'e>(
        _input: &'e str,
        chars: &mut Peekable<CharIndices<'e>>,
        i: usize,
        c: char,
    ) -> (Result<InfixToken<'e>, Error<'e>>, State)
    {
        let token = match c {
            '+' => Ok(InfixToken::Op(Op::Add)),
            '-' => Ok(InfixToken::Op(Op::Sub)),
            '*' => Ok(InfixToken::Op(Op::Mul)),
            '/' => Ok(InfixToken::Op(Op::Div)),
            '^' => Ok(InfixToken::Op(Op::Pow)),
            _ => {
                return (
                    Err(Error::ParseError(ParseError::UnexpectedChar(
                        Cow::Owned(c),
                        i,
                    ))),
                    State::AfterError,
                );
            }
        };

        (token, State::ExpectingNumberProducer)
    }

    fn handle_number_or_ident<'e>(
        input: &'e str,
        chars: &mut Peekable<CharIndices<'e>>,
        i: usize,
        c: char,
    ) -> (Result<InfixToken<'e>, Error<'e>>, State)
    {
        return match c {
            // numbers
            '0'..='9' | '.' | '-' => {
                let value = fast_parse_f64(c, chars);

                (Ok(InfixToken::Num(value)), State::ExpectingOperator)
            }

            // identifiers (variables or functions)
            'a'..='z' | 'A'..='Z' | '_' => {
                let start_index = i;
                let mut end_index = input.len();

                let token = loop {
                    if let Some(&(i, d)) = chars.peek() {
                        if d.is_alphanumeric() || d == '_' {
                            chars.next();
                            continue;
                        }

                        // function found
                        if d == '(' || d == '[' {
                            let fn_name = &input[start_index..i];
                            chars.next();

                            let mut params = Vec::with_capacity(2);

                            let mut depth = 1;
                            let mut start_index = i + 1; // Skipping the opening bracket of the function call
                            let mut end_index = input.len();

                            // TODO:: Avoid this, token FunctionStart, Comma ...
                            while let Some((i, d)) = chars.next() {
                                match d {
                                    '(' | '[' => depth += 1,
                                    ')' | ']' => depth -= 1,
                                    ',' => {
                                        let param_expr =
                                            match InfixExpr::try_from(&input[start_index..i]) {
                                                Ok(expr) => expr,
                                                Err(err) => {
                                                    return (Err(err), State::AfterError);
                                                }
                                            };
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

                            let param_expr =
                                match InfixExpr::try_from(&input[start_index..end_index]) {
                                    Ok(expr) => expr,
                                    Err(err) => return (Err(err), State::AfterError),
                                };
                            params.push(param_expr);

                            break InfixToken::Fn(fn_name, params);
                        }

                        break InfixToken::Var(&input[start_index..i]);
                    } else {
                        break InfixToken::Var(&input[start_index..end_index]);
                    }
                };

                (Ok(token), State::ExpectingOperator)
            }

            _ => (
                Err(Error::ParseError(ParseError::UnexpectedChar(
                    Cow::Owned(c),
                    i,
                ))),
                State::AfterError,
            ),
        };

        fn fast_parse_f64(c: char, chars: &mut Peekable<CharIndices>) -> f64
        {
            let mut value: f64 = 0.0;
            let mut frac = 0.1;
            let mut is_fraction = false;

            let multiplier = if c == '-' {
                -1.0
            } else {
                match c {
                    '0'..='9' => {
                        if is_fraction {
                            value += (c as u8 - b'0') as f64 * frac;
                            frac *= 0.1;
                        } else {
                            value = value * 10.0 + (c as u8 - b'0') as f64;
                        }
                    }
                    '.' if !is_fraction => {
                        is_fraction = true;
                    }
                    _ => unreachable!("can't happend"),
                }

                1.0
            };

            while let Some(&(i, d)) = chars.peek() {
                match d {
                    '0'..='9' => {
                        chars.next();
                        if is_fraction {
                            value += (d as u8 - b'0') as f64 * frac;
                            frac *= 0.1;
                        } else {
                            value = value * 10.0 + (d as u8 - b'0') as f64;
                        }
                    }
                    '.' if !is_fraction => {
                        chars.next();
                        is_fraction = true;
                    }
                    _ => break,
                }
            }

            value
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_str_to_infix()
    {
        let expr = "2-4-2.4*5+6/p0";
        let infix_expr = InfixExpr::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![
                InfixToken::Num(2.0),
                InfixToken::Op(Op::Sub),
                InfixToken::Num(4.0),
                InfixToken::Op(Op::Sub),
                InfixToken::Num(2.4),
                InfixToken::Op(Op::Mul),
                InfixToken::Num(5.0),
                InfixToken::Op(Op::Add),
                InfixToken::Num(6.0),
                InfixToken::Op(Op::Div),
                InfixToken::Var("p0"),
            ]
        );

        let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
        let infix_expr = InfixExpr::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![
                InfixToken::Num(2.0),
                InfixToken::Op(Op::Sub),
                InfixToken::LParen,
                InfixToken::Num(4.0),
                InfixToken::Op(Op::Add),
                InfixToken::LParen,
                InfixToken::Var("p19"),
                InfixToken::Op(Op::Sub),
                InfixToken::Num(2.0),
                InfixToken::RParen,
                InfixToken::Op(Op::Mul),
                InfixToken::LParen,
                InfixToken::Var("p19"),
                InfixToken::Op(Op::Add),
                InfixToken::Num(2.0),
                InfixToken::RParen,
                InfixToken::RParen,
            ]
        );

        let expr = "abs((2 + 3) * 4, sqrt(5))";
        let infix_expr = InfixExpr::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![InfixToken::Fn(
                "abs",
                vec![
                    InfixExpr {
                        tokens: vec![
                            InfixToken::LParen,
                            InfixToken::Num(2.0),
                            InfixToken::Op(Op::Add),
                            InfixToken::Num(3.0),
                            InfixToken::RParen,
                            InfixToken::Op(Op::Mul),
                            InfixToken::Num(4.0)
                        ]
                    },
                    InfixExpr {
                        tokens: vec![InfixToken::Fn(
                            "sqrt",
                            vec![InfixExpr {
                                tokens: vec![InfixToken::Num(5.0)]
                            }]
                        )]
                    }
                ]
            ),]
        );

        let expr = "abs((2 * 21) + p0)";
        let infix_expr = InfixExpr::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![InfixToken::Fn(
                "abs",
                vec![InfixExpr {
                    tokens: vec![
                        InfixToken::LParen,
                        InfixToken::Num(2.0),
                        InfixToken::Op(Op::Mul),
                        InfixToken::Num(21.0),
                        InfixToken::RParen,
                        InfixToken::Op(Op::Add),
                        InfixToken::Var("p0")
                    ]
                }]
            )]
        )
    }

    #[test]
    fn test_errors()
    {
        // TODO: The indices of the errors are relative to the start of the function due
        //  to the recursion used to parse the expression

        let expr = "abs((2.0.0 + 3) * 4, sqrt(5))";
        let result = InfixExpr::try_from(expr);

        assert_eq!(
            result,
            Err(Error::ParseError(ParseError::UnexpectedChar(
                Cow::Owned('.'),
                4
            )))
        );

        let expr = "abs((2 + 3) &* 4, sqrt(5))";
        let result = InfixExpr::try_from(expr);

        assert_eq!(
            result,
            Err(Error::ParseError(ParseError::UnexpectedChar(
                Cow::Owned('&'),
                8
            )))
        );
    }
}

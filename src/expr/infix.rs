use std::{borrow::Cow, iter::Peekable, ops::Deref, str::CharIndices};

use crate::{op::Op, *};

#[derive(Debug, PartialEq)]
pub enum InfixToken<'e>
{
    Num(f64),
    Var(&'e str),
    Fn(&'e str, Vec<Expr<InfixToken<'e>>>),
    Op(Op),
    LParen,
    RParen,
}

impl<'e> Deref for Expr<InfixToken<'e>>
{
    type Target = Vec<InfixToken<'e>>;

    fn deref(&self) -> &Self::Target
    {
        &self.tokens
    }
}

impl<'e> IntoIterator for Expr<InfixToken<'e>>
{
    type Item = InfixToken<'e>;
    type IntoIter = std::vec::IntoIter<InfixToken<'e>>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.tokens.into_iter()
    }
}

impl<'e> TryFrom<&'e str> for Expr<InfixToken<'e>>
{
    type Error = crate::Error<'e>;

    fn try_from(input: &'e str) -> Result<Self, Self::Error>
    {
        let mut lexer = Lexer::new(input);

        Ok(Expr {
            tokens: lexer.lex()?,
        })
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
            state: State::Default,
        }
    }

    fn lex(&mut self) -> Result<Vec<InfixToken<'e>>, Error<'e>>
    {
        let mut tokens: Vec<InfixToken<'e>> = Vec::with_capacity(self.input.len() / 2);

        let chars = &mut self.chars;
        let input = self.input;

        while let Some((i, c)) = chars.next() {
            match c {
                ' ' | '\t' | '\n' => {
                    // Ignore whitespace
                }
                '(' | '[' => {
                    tokens.push(InfixToken::LParen);
                }
                ')' | ']' => {
                    tokens.push(InfixToken::RParen);
                }
                _ => {
                    let next_state = self.state.lex(input, &mut tokens, chars, i, c)?;
                    self.state = next_state;
                }
            }
        }

        Ok(tokens)
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
    fn lex<'e>(
        &mut self,
        input: &'e str,
        output: &mut Vec<InfixToken<'e>>,
        chars: &mut Peekable<CharIndices<'e>>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    {
        match self {
            State::ExpectingOperator => Self::handle_expecting_operator(input, output, chars, i, c),
            State::Default => Self::handle_default(input, output, chars, i, c),
        }
    }

    #[inline(always)]
    fn handle_expecting_operator<'e>(
        _input: &'e str,
        output: &mut Vec<InfixToken<'e>>,
        _chars: &mut Peekable<CharIndices<'e>>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    {
        match c {
            '+' => output.push(InfixToken::Op(Op::Add)),
            '-' => output.push(InfixToken::Op(Op::Sub)),
            '*' => output.push(InfixToken::Op(Op::Mul)),
            '/' => output.push(InfixToken::Op(Op::Div)),
            '^' => output.push(InfixToken::Op(Op::Pow)),
            _ => {
                return Err(Error::ParseError(ParseError::UnexpectedChar(
                    Cow::Owned(c),
                    i,
                )));
            }
        };

        Ok(State::Default)
    }

    #[inline(always)]
    fn handle_default<'e>(
        input: &'e str,
        output: &mut Vec<InfixToken<'e>>,
        chars: &mut Peekable<CharIndices<'e>>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    {
        return match c {
            '-' => {
                output.push(InfixToken::Op(Op::Neg));
                Ok(State::Default)
            }
            // numbers
            '0'..='9' | '.' => {
                let value = parsing::parse_uf64(c, chars);
                output.push(InfixToken::Num(value));

                Ok(State::ExpectingOperator)
            }

            // identifiers (variables or functions)
            'a'..='z' | 'A'..='Z' | '_' => {
                let start_index = i;
                let end_index = input.len();

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
                                        let param_expr = match Expr::<InfixToken>::try_from(
                                            &input[start_index..i],
                                        ) {
                                            Ok(expr) => expr,
                                            Err(err) => {
                                                return Err(err);
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
                                Expr::<InfixToken>::try_from(&input[start_index..end_index])?;
                            params.push(param_expr);

                            break InfixToken::Fn(fn_name, params);
                        }

                        break InfixToken::Var(&input[start_index..i]);
                    } else {
                        break InfixToken::Var(&input[start_index..end_index]);
                    }
                };

                output.push(token);
                Ok(State::ExpectingOperator)
            }

            _ => Err(Error::ParseError(ParseError::UnexpectedChar(
                Cow::Owned(c),
                i,
            ))),
        };
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
        let infix_expr = Expr::<InfixToken>::try_from(expr).unwrap();
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

        let expr = "2 - (4 + (p19 - 2) * (-p19 + 2))";
        let infix_expr = Expr::<InfixToken>::try_from(expr).unwrap();
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
                InfixToken::Op(Op::Neg),
                InfixToken::Var("p19"),
                InfixToken::Op(Op::Add),
                InfixToken::Num(2.0),
                InfixToken::RParen,
                InfixToken::RParen,
            ]
        );

        let expr = "abs((2 + 3) * 4, sqrt(5))";
        let infix_expr = Expr::<InfixToken>::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![InfixToken::Fn(
                "abs",
                vec![
                    Expr::<InfixToken> {
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
                    Expr::<InfixToken> {
                        tokens: vec![InfixToken::Fn(
                            "sqrt",
                            vec![Expr::<InfixToken> {
                                tokens: vec![InfixToken::Num(5.0)]
                            }]
                        )]
                    }
                ]
            ),]
        );

        let expr = "abs((2 * 21) + p0)";
        let infix_expr = Expr::<InfixToken>::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![InfixToken::Fn(
                "abs",
                vec![Expr::<InfixToken> {
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
        let result = Expr::<InfixToken>::try_from(expr);

        assert_eq!(
            result,
            Err(Error::ParseError(ParseError::UnexpectedChar(
                Cow::Owned('.'),
                4
            )))
        );

        let expr = "abs((2 + 3) &* 4, sqrt(5))";
        let result = Expr::<InfixToken>::try_from(expr);

        assert_eq!(
            result,
            Err(Error::ParseError(ParseError::UnexpectedChar(
                Cow::Owned('&'),
                8
            )))
        );
    }
}

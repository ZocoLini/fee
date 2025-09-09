use std::{borrow::Cow, iter::Peekable, ops::Deref, str::CharIndices};

use crate::{expr::Expr, op::Op, *};

#[derive(Debug, PartialEq)]
pub enum Infix<'e>
{
    Num(f64),
    Var(&'e str),
    Fn(&'e str, Vec<Expr<Infix<'e>>>),
    Op(Op),
    LParen,
    RParen,
}

impl<'e> Deref for Expr<Infix<'e>>
{
    type Target = Vec<Infix<'e>>;

    fn deref(&self) -> &Self::Target
    {
        &self.tokens
    }
}

impl<'e> IntoIterator for Expr<Infix<'e>>
{
    type Item = Infix<'e>;
    type IntoIter = std::vec::IntoIter<Infix<'e>>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.tokens.into_iter()
    }
}

impl<'e> TryFrom<&'e str> for Expr<Infix<'e>>
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

    fn lex(&mut self) -> Result<Vec<Infix<'e>>, Error<'e>>
    {
        let mut tokens: Vec<Infix<'e>> = Vec::with_capacity(self.input.len() / 2);

        let chars = &mut self.chars;
        let input = self.input;

        while let Some((i, c)) = chars.next() {
            match c {
                ' ' | '\t' | '\n' => {
                    // Ignore whitespace
                }
                '(' | '[' => {
                    tokens.push(Infix::LParen);
                }
                ')' | ']' => {
                    tokens.push(Infix::RParen);
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
        output: &mut Vec<Infix<'e>>,
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
        output: &mut Vec<Infix<'e>>,
        _chars: &mut Peekable<CharIndices<'e>>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    {
        match c {
            '+' => output.push(Infix::Op(Op::Add)),
            '-' => output.push(Infix::Op(Op::Sub)),
            '*' => output.push(Infix::Op(Op::Mul)),
            '/' => output.push(Infix::Op(Op::Div)),
            '^' => output.push(Infix::Op(Op::Pow)),
            '%' => output.push(Infix::Op(Op::Mod)),
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
        output: &mut Vec<Infix<'e>>,
        chars: &mut Peekable<CharIndices<'e>>,
        i: usize,
        c: char,
    ) -> Result<State, Error<'e>>
    {
        return match c {
            '-' => {
                output.push(Infix::Op(Op::Neg));
                Ok(State::Default)
            }
            // numbers
            '0'..='9' | '.' => {
                let value = parsing::parse_uf64(c, chars);
                output.push(Infix::Num(value));

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
                                    // If depth is greater than 1, it is a parameter separator of a nested function call
                                    ',' if depth == 1 => {
                                        let param_expr =
                                            Expr::<Infix>::try_from(&input[start_index..i])?;
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
                                Expr::<Infix>::try_from(&input[start_index..end_index])?;
                            params.push(param_expr);

                            break Infix::Fn(fn_name, params);
                        }

                        break Infix::Var(&input[start_index..i]);
                    } else {
                        break Infix::Var(&input[start_index..end_index]);
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
        let infix_expr = Expr::<Infix>::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![
                Infix::Num(2.0),
                Infix::Op(Op::Sub),
                Infix::Num(4.0),
                Infix::Op(Op::Sub),
                Infix::Num(2.4),
                Infix::Op(Op::Mul),
                Infix::Num(5.0),
                Infix::Op(Op::Add),
                Infix::Num(6.0),
                Infix::Op(Op::Div),
                Infix::Var("p0"),
            ]
        );

        let expr = "2 - (4 + (p19 - 2) * (-p19 + 2))";
        let infix_expr = Expr::<Infix>::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![
                Infix::Num(2.0),
                Infix::Op(Op::Sub),
                Infix::LParen,
                Infix::Num(4.0),
                Infix::Op(Op::Add),
                Infix::LParen,
                Infix::Var("p19"),
                Infix::Op(Op::Sub),
                Infix::Num(2.0),
                Infix::RParen,
                Infix::Op(Op::Mul),
                Infix::LParen,
                Infix::Op(Op::Neg),
                Infix::Var("p19"),
                Infix::Op(Op::Add),
                Infix::Num(2.0),
                Infix::RParen,
                Infix::RParen,
            ]
        );

        let expr = "abs((2 + 3) * 4, sqrt(5))";
        let infix_expr = Expr::<Infix>::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![Infix::Fn(
                "abs",
                vec![
                    Expr::<Infix> {
                        tokens: vec![
                            Infix::LParen,
                            Infix::Num(2.0),
                            Infix::Op(Op::Add),
                            Infix::Num(3.0),
                            Infix::RParen,
                            Infix::Op(Op::Mul),
                            Infix::Num(4.0)
                        ]
                    },
                    Expr::<Infix> {
                        tokens: vec![Infix::Fn(
                            "sqrt",
                            vec![Expr::<Infix> {
                                tokens: vec![Infix::Num(5.0)]
                            }]
                        )]
                    }
                ]
            ),]
        );

        let expr = "abs((2 * 21) + p0)";
        let infix_expr = Expr::<Infix>::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![Infix::Fn(
                "abs",
                vec![Expr::<Infix> {
                    tokens: vec![
                        Infix::LParen,
                        Infix::Num(2.0),
                        Infix::Op(Op::Mul),
                        Infix::Num(21.0),
                        Infix::RParen,
                        Infix::Op(Op::Add),
                        Infix::Var("p0")
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
        let result = Expr::<Infix>::try_from(expr);

        assert_eq!(
            result,
            Err(Error::ParseError(ParseError::UnexpectedChar(
                Cow::Owned('.'),
                4
            )))
        );

        let expr = "abs((2 + 3) &* 4, sqrt(5))";
        let result = Expr::<Infix>::try_from(expr);

        assert_eq!(
            result,
            Err(Error::ParseError(ParseError::UnexpectedChar(
                Cow::Owned('&'),
                8
            )))
        );
    }
}

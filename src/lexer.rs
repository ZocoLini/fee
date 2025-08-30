use std::{borrow::Cow, iter::Peekable, ops::Deref, str::CharIndices};

use crate::{
    token::{Operator, Token},
    *,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Infix;

#[derive(Debug, Clone, PartialEq)]
pub struct Expr<'e, Type>
{
    tokens: Vec<Token<'e>>,
    type_: Type,
}

impl<'e, T> Expr<'e, T>
{
    pub fn new(tokens: Vec<Token<'e>>, type_: T) -> Self
    {
        Self { tokens, type_ }
    }
}

impl<'e, T> Deref for Expr<'e, T>
{
    type Target = Vec<Token<'e>>;

    fn deref(&self) -> &Self::Target
    {
        &self.tokens
    }
}

impl<'e, T> IntoIterator for Expr<'e, T>
{
    type Item = Token<'e>;
    type IntoIter = std::vec::IntoIter<Token<'e>>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.tokens.into_iter()
    }
}

impl<'e> TryFrom<&'e str> for Expr<'e, Infix>
{
    type Error = crate::Error<'e>;

    fn try_from(input: &'e str) -> Result<Self, Self::Error>
    {
        let mut tokens: Vec<Token<'e>> = Vec::new();
        let mut lexer = Lexer::new(input);

        while let Some(token) = lexer.next_token()? {
            tokens.push(token);
        }

        Ok(Expr {
            tokens,
            type_: Infix,
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
            state: State::ExpectingNumberProducer,
        }
    }

    fn next_token(&mut self) -> Result<Option<Token<'e>>, Error<'e>>
    {
        let chars = &mut self.chars;
        let input = self.input;

        while let Some(&(i, c)) = chars.peek() {
            match c {
                ' ' | '\t' | '\n' => {
                    chars.next();
                }
                '(' | '[' => {
                    chars.next();
                    return Ok(Some(Token::LParen));
                }
                ')' | ']' => {
                    chars.next();
                    return Ok(Some(Token::RParen));
                }
                _ => {
                    let (result, next_state) = self.state.next_token(self.input, &mut self.chars);
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
    ) -> (Result<Token<'e>, Error<'e>>, State)
    {
        match self {
            State::ExpectingOperator => Self::handle_expecting_operator(input, chars),
            State::ExpectingNumberProducer => Self::handle_number_or_ident(input, chars),
            State::AfterError => panic!("tried to continue parsing after error"),
        }
    }

    fn handle_expecting_operator<'e>(
        _input: &'e str,
        chars: &mut Peekable<CharIndices<'e>>,
    ) -> (Result<Token<'e>, Error<'e>>, State)
    {
        let &(i, c) = chars.peek().expect("Already peeked");
        match c {
            '+' => {
                chars.next();
                (
                    Ok(Token::Operator(Operator::Add)),
                    State::ExpectingNumberProducer,
                )
            }
            '-' => {
                chars.next();
                (
                    Ok(Token::Operator(Operator::Sub)),
                    State::ExpectingNumberProducer,
                )
            }
            '*' => {
                chars.next();
                (
                    Ok(Token::Operator(Operator::Mul)),
                    State::ExpectingNumberProducer,
                )
            }
            '/' => {
                chars.next();
                (
                    Ok(Token::Operator(Operator::Div)),
                    State::ExpectingNumberProducer,
                )
            }
            '^' => {
                chars.next();
                (
                    Ok(Token::Operator(Operator::Pow)),
                    State::ExpectingNumberProducer,
                )
            }
            _ => (
                Err(Error::ParseError(ParseError::UnexpectedChar(
                    Cow::Owned(c),
                    i,
                ))),
                State::AfterError,
            ),
        }
    }

    fn handle_number_or_ident<'e>(
        input: &'e str,
        chars: &mut Peekable<CharIndices<'e>>,
    ) -> (Result<Token<'e>, Error<'e>>, State)
    {
        let &(i, c) = chars.peek().expect("Already peeked");

        match c {
            // numbers
            '0'..='9' | '.' | '-' => {
                chars.next();
                let multiplier = if c == '-' { -1.0 } else { 1.0 };

                let start_index = i;
                let mut end_index = input.len();

                while let Some(&(i, d)) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        chars.next();
                    } else {
                        end_index = i;
                        break;
                    }
                }

                let num_str = &input[start_index..end_index];
                let value: f64 = match num_str.parse() {
                    Ok(v) => v,
                    Err(_) => {
                        return (
                            Err(Error::ParseError(ParseError::InvalidNumber(
                                Cow::Borrowed(num_str),
                                i,
                            ))),
                            State::AfterError,
                        );
                    }
                };

                (
                    Ok(Token::Number(value * multiplier)),
                    State::ExpectingOperator,
                )
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

                            let mut params = Vec::new();

                            let mut depth = 1;
                            let mut start_index = i + 1; // Skipping the opening bracket of the function call
                            let mut end_index = input.len();

                            while let Some((i, d)) = chars.next() {
                                match d {
                                    '(' | '[' => depth += 1,
                                    ')' | ']' => depth -= 1,
                                    ',' => {
                                        let param_expr =
                                            match Expr::try_from(&input[start_index..i]) {
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

                            let param_expr = match Expr::try_from(&input[start_index..end_index]) {
                                Ok(expr) => expr,
                                Err(err) => return (Err(err), State::AfterError),
                            };
                            params.push(param_expr);

                            break Token::Function(fn_name, params);
                        }

                        break Token::Variable(&input[start_index..i]);
                    } else {
                        break Token::Variable(&input[start_index..end_index]);
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
        let infix_expr = Expr::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![
                Token::Number(2.0),
                Token::Operator(Operator::Sub),
                Token::Number(4.0),
                Token::Operator(Operator::Sub),
                Token::Number(2.4),
                Token::Operator(Operator::Mul),
                Token::Number(5.0),
                Token::Operator(Operator::Add),
                Token::Number(6.0),
                Token::Operator(Operator::Div),
                Token::Variable("p0"),
            ]
        );

        let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
        let infix_expr = Expr::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![
                Token::Number(2.0),
                Token::Operator(Operator::Sub),
                Token::LParen,
                Token::Number(4.0),
                Token::Operator(Operator::Add),
                Token::LParen,
                Token::Variable("p19"),
                Token::Operator(Operator::Sub),
                Token::Number(2.0),
                Token::RParen,
                Token::Operator(Operator::Mul),
                Token::LParen,
                Token::Variable("p19"),
                Token::Operator(Operator::Add),
                Token::Number(2.0),
                Token::RParen,
                Token::RParen,
            ]
        );

        let expr = "abs((2 + 3) * 4, sqrt(5))";
        let infix_expr = Expr::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![Token::Function(
                "abs",
                vec![
                    Expr {
                        tokens: vec![
                            Token::LParen,
                            Token::Number(2.0),
                            Token::Operator(Operator::Add),
                            Token::Number(3.0),
                            Token::RParen,
                            Token::Operator(Operator::Mul),
                            Token::Number(4.0)
                        ],
                        type_: Infix
                    },
                    Expr {
                        tokens: vec![Token::Function(
                            "sqrt",
                            vec![Expr {
                                tokens: vec![Token::Number(5.0)],
                                type_: Infix
                            }]
                        )],
                        type_: Infix
                    }
                ]
            ),]
        );

        let expr = "abs((2 * 21) + p0)";
        let infix_expr = Expr::try_from(expr).unwrap();
        assert_eq!(
            *infix_expr,
            vec![Token::Function(
                "abs",
                vec![Expr {
                    tokens: vec![
                        Token::LParen,
                        Token::Number(2.0),
                        Token::Operator(Operator::Mul),
                        Token::Number(21.0),
                        Token::RParen,
                        Token::Operator(Operator::Add),
                        Token::Variable("p0")
                    ],
                    type_: Infix
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
        let result = Expr::try_from(expr);

        assert_eq!(
            result,
            Err(Error::ParseError(ParseError::InvalidNumber(
                Cow::Borrowed("2.0.0"),
                1
            )))
        );

        let expr = "abs((2 + 3) &* 4, sqrt(5))";
        let result = Expr::try_from(expr);

        assert_eq!(
            result,
            Err(Error::ParseError(ParseError::UnexpectedChar(
                Cow::Owned('&'),
                8
            )))
        );
    }
}

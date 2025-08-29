use std::{borrow::Cow, ops::Deref};

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
        let mut chars = input.char_indices().peekable();

        let mut expecting_operator = false;
        
        while let Some(&(i, c)) = chars.peek() {
            match c {
                // ignoring whitespace
                ' ' | '\t' | '\n' => {
                    chars.next();
                }

                // numbers and substract operator
                '0'..='9' | '.' | '-' => {
                    chars.next();

                    if c == '-' && expecting_operator
                    {
                        tokens.push(Token::Operator(Operator::Sub));
                        expecting_operator = false;
                        continue;
                    }

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
                    let value: f64 = num_str.parse().map_err(|_| {
                        Error::ParseError(ParseError::InvalidNumber(Cow::Borrowed(num_str), i))
                    })?;
                    tokens.push(Token::Number(value * multiplier));
                    expecting_operator = true;
                }

                // variables or functions
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
                                                Expr::try_from(&input[start_index..i])?;
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

                                let param_expr = Expr::try_from(&input[start_index..end_index])?;
                                params.push(param_expr);

                                break Token::Function(fn_name, params);
                            }

                            break Token::Variable(&input[start_index..i]);
                        } else {
                            break Token::Variable(&input[start_index..i]);
                        }
                    };

                    tokens.push(token);
                }

                // operators and parentheses
                '+' => {
                    tokens.push(Token::Operator(Operator::Add));
                    chars.next();
                }
                '*' => {
                    tokens.push(Token::Operator(Operator::Mul));
                    chars.next();
                }
                '/' => {
                    tokens.push(Token::Operator(Operator::Div));
                    chars.next();
                }
                '^' => {
                    tokens.push(Token::Operator(Operator::Pow));
                    chars.next();
                }
                '(' | '[' => {
                    tokens.push(Token::LParen);
                    chars.next();
                }
                ')' | ']' => {
                    tokens.push(Token::RParen);
                    chars.next();
                }

                other => {
                    return Err(Error::ParseError(ParseError::UnexpectedChar(
                        Cow::Owned(other),
                        i,
                    )));
                }
            }
        }

        Ok(Expr {
            tokens,
            type_: Infix,
        })
    }
}
#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_str_to_infix()
    {
        let expr = "2-4-2.4*5+6/7";
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
                Token::Number(7.0),
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

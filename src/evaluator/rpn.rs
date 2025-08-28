use crate::{Error, prelude::*};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator
{
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

impl Operator
{
    pub fn precedence(&self) -> u8
    {
        match self {
            Operator::Add | Operator::Sub => 1,
            Operator::Mul | Operator::Div => 2,
            Operator::Pow => 3,
        }
    }

    fn is_right_associative(&self) -> bool
    {
        matches!(self, Operator::Pow)
    }

    fn apply(&self, lhs: f64, rhs: f64) -> f64
    {
        match self {
            Operator::Add => lhs + rhs,
            Operator::Sub => lhs - rhs,
            Operator::Mul => lhs * rhs,
            Operator::Div => lhs / rhs,
            Operator::Pow => lhs.powf(rhs),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'e>
{
    Number(f64),
    Variable(&'e str),
    /// name and number of arguments
    FunctionCall(&'e str, usize),
    Function(&'e str, Vec<Vec<Token<'e>>>),
    Operator(Operator),
    Comma,
    LParen,
    RParen,
}

pub struct RPNEvaluator<'e, 'c, V: VarResolver, F: FnResolver>
{
    expr: &'e str,
    ctx: &'c Context<V, F>,
    rpn: Vec<Token<'e>>,
}

impl<'e, 'c, V: VarResolver, F: FnResolver> RPNEvaluator<'e, 'c, V, F>
{
    pub fn new(expr: &'e str, ctx: &'c Context<V, F>) -> Result<Self, crate::Error>
    {
        let tokens = lex(expr)?;
        let rpn = shunting_yard(&tokens);
        Ok(RPNEvaluator { expr, ctx, rpn })
    }
}

impl<'e, 'c, V: VarResolver, F: FnResolver> Evaluator for RPNEvaluator<'e, 'c, V, F>
{
    fn eval(&self) -> f64
    {
        let mut stack = Vec::new();

        for tok in self.rpn.iter() {
            match tok {
                Token::Number(num) => stack.push(*num),
                Token::Variable(name) => {
                    stack.push(self.ctx.vals.get(*name).expect("Missing variable"))
                }
                Token::FunctionCall(name, argc) => {
                    if *argc > stack.len() {
                        panic!("Not enough args to call {name}")
                    }

                    let start_index = stack.len() - *argc;

                    let val = {
                        let args = stack.drain(start_index..stack.len());
                        let args = args.as_slice();

                        self.ctx
                            .funcs
                            .call(name, &args)
                            .unwrap_or_else(|| panic!("Unknown function: {}", name))
                    };

                    stack.push(val);
                }
                Token::Operator(op) => {
                    let b = stack.pop().expect("Stack underflow for operator");
                    let a = stack.pop().expect("Stack underflow for operator");
                    let res = op.apply(a, b);
                    stack.push(res);
                }
                _ => panic!("Unexpected token in RPN: {:?}", tok),
            }
        }

        if stack.len() != 1 {
            panic!("Stack didn't contain exactly one element after evaluation")
        } else {
            stack.pop().unwrap()
        }
    }
}

fn lex<'e>(input: &'e str) -> Result<Vec<Token<'e>>, crate::Error>
{
    let mut tokens = Vec::new();
    let mut chars = input.char_indices().peekable();

    while let Some(&(i, c)) = chars.peek() {
        match c {
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }

            // ignoring whitespace
            ' ' | '\t' | '\n' => {
                chars.next();
            }

            // TODO: The parser is expected to fail parsing 2 -4.
            //  State machine should solve this issue detecting if we are expecting an operator or a number
            // numbers and substract operator
            '0'..='9' | '.' | '-' => {
                chars.next();

                if c == '-'
                    && chars
                        .peek()
                        .map_or(true, |&(_, next)| !next.is_ascii_digit() && next != '.')
                {
                    tokens.push(Token::Operator(Operator::Sub));
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
                let value: f64 = num_str
                    .parse()
                    .map_err(|_| Error::InvalidNumber(format!("{num_str} at index {i}")))?;
                tokens.push(Token::Number(value * multiplier))
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

                            let mut depth = 1;
                            let start_index = i + 1; // Skipping the opening bracket of the function call
                            let mut end_index = input.len();

                            while let Some((i, d)) = chars.next() {
                                if d == '(' || d == '[' {
                                    depth += 1;
                                } else if d == ')' || d == ']' {
                                    depth -= 1;
                                }

                                if depth == 0 {
                                    end_index = i;
                                    break;
                                }
                            }

                            let fun_params = lex(&input[start_index..end_index])?;
                            let splitted_fun_params: Vec<Vec<Token>> = fun_params
                                .split(|tok| *tok == Token::Comma)
                                .map(|tokens| tokens.to_vec())
                                .collect();

                            break Token::Function(fn_name, splitted_fun_params);
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
                return Err(Error::UnexpectedToken(format!("{other} at index {i}")));
            }
        }
    }

    Ok(tokens)
}

fn shunting_yard<'e>(tokens: &[Token<'e>]) -> Vec<Token<'e>>
{
    let mut output: Vec<Token> = Vec::with_capacity(tokens.len());
    let mut ops: Vec<Token> = Vec::new();

    for tok in tokens {
        match tok {
            Token::Number(_) | Token::Variable(_) => {
                output.push(tok.clone());
            }
            Token::Operator(op) => {
                while let Some(Token::Operator(top)) = ops.last() {
                    let should_pop = if op.is_right_associative() {
                        op.precedence() < top.precedence()
                    } else {
                        op.precedence() <= top.precedence()
                    };

                    if should_pop {
                        output.push(ops.pop().unwrap());
                    } else {
                        break;
                    }
                }
                ops.push(tok.clone());
            }
            Token::LParen => ops.push(tok.clone()),
            Token::RParen => {
                while let Some(top) = ops.pop() {
                    if let Token::LParen = top {
                        break;
                    } else {
                        output.push(top);
                    }
                }
            }
            Token::Function(name, args) => {
                let fun_call_token = Token::FunctionCall(name, args.len());

                let rpn_args = args
                    .into_iter()
                    .map(|arg_tokens| shunting_yard(arg_tokens))
                    .collect::<Vec<_>>();

                rpn_args
                    .iter()
                    .for_each(|rpn_arg| output.extend_from_slice(rpn_arg));

                output.push(fun_call_token);
            }
            _ => panic!("Unexpected token"),
        }
    }

    while let Some(op) = ops.pop() {
        output.push(op);
    }

    output
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn test_str_to_rpn()
    {
        let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";

        let tokens = lex(expr).unwrap();
        assert_eq!(
            tokens,
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

        let rpn = shunting_yard(&tokens);
        assert_eq!(
            rpn,
            vec![
                Token::Number(2.0),
                Token::Number(4.0),
                Token::Variable("p19"),
                Token::Number(2.0),
                Token::Operator(Operator::Sub),
                Token::Variable("p19"),
                Token::Number(2.0),
                Token::Operator(Operator::Add),
                Token::Operator(Operator::Mul),
                Token::Operator(Operator::Add),
                Token::Operator(Operator::Sub)
            ]
        );

        let expr = "abs((2 + 3) * 4, sqrt(5))";

        let tokens = lex(expr).unwrap();
        assert_eq!(
            tokens,
            vec![Token::Function(
                "abs",
                vec![
                    vec![
                        Token::LParen,
                        Token::Number(2.0),
                        Token::Operator(Operator::Add),
                        Token::Number(3.0),
                        Token::RParen,
                        Token::Operator(Operator::Mul),
                        Token::Number(4.0)
                    ],
                    vec![Token::Function("sqrt", vec![vec![Token::Number(5.0)]])]
                ]
            ),]
        );

        let rpn = shunting_yard(&tokens);
        assert_eq!(
            rpn,
            vec![
                Token::Number(2.0),
                Token::Number(3.0),
                Token::Operator(Operator::Add),
                Token::Number(4.0),
                Token::Operator(Operator::Mul),
                Token::Number(5.0),
                Token::FunctionCall("sqrt", 1),
                Token::FunctionCall("abs", 2),
            ]
        );
    }
}

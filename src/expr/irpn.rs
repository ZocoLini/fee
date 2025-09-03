use std::borrow::Cow;

use smallvec::{SmallVec, smallvec};

use crate::{
    Error, EvalError, IndexedResolver,
    expr::infix::{InfixExpr, InfixToken},
    op::Op,
    parsing,
    prelude::*,
};

#[derive(Debug, PartialEq)]
pub enum IRpnToken
{
    Num(f64),
    Var(usize, usize),
    Fn(usize, usize, usize),
    Op(Op),
}

impl<'e> From<InfixToken<'e>> for IRpnToken
{
    fn from(token: InfixToken<'e>) -> Self
    {
        match token {
            InfixToken::Num(num) => IRpnToken::Num(num),
            InfixToken::Var(name) => {
                let name_bytes = name.as_bytes();

                let letter = name_bytes[0] - b'a';
                let idx = parsing::parse_usize(&name_bytes[1..]);

                IRpnToken::Var(letter as usize, idx)
            }
            InfixToken::Fn(name, args) => {
                let name_bytes = name.as_bytes();

                let letter = name_bytes[0] - b'a';
                let idx = parsing::parse_usize(&name_bytes[1..]);

                IRpnToken::Fn(letter as usize, idx, args.len())
            }
            InfixToken::Op(op) => IRpnToken::Op(op),
            _ => unreachable!("logic bug found"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct IRpnExpr
{
    tokens: Vec<IRpnToken>,
}

impl IRpnExpr
{
    pub fn new<'e>(expr: &'e str) -> Result<Self, crate::Error<'e>>
    {
        let infix_expr = InfixExpr::try_from(expr)?;
        IRpnExpr::try_from(infix_expr)
    }

    pub fn len(&self) -> usize
    {
        self.tokens.len()
    }

    pub(crate) fn eval(
        &self,
        ctx: &Context<IndexedResolver<f64>, IndexedResolver<ExprFn>>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'_>>
    {
        if self.tokens.len() == 1 {
            if let IRpnToken::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                IRpnToken::Num(num) => stack.push(*num),
                IRpnToken::Var(id, idx) => {
                    stack.push(*ctx.get_var_by_index(*id, *idx).ok_or_else(|| {
                        Error::EvalError(EvalError::UnknownVar(Cow::Owned(format!(
                            "{}{}",
                            (*id as u8 + b'a') as char,
                            idx
                        ))))
                    })?)
                }
                IRpnToken::Fn(id, idx, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
                    let val = match ctx.call_fn_by_index(*id, *idx, args) {
                        Some(value) => value,
                        None => {
                            return Err(Error::EvalError(EvalError::UnknownFn(Cow::Owned(
                                format!("{}{}", (*id as u8 + b'a') as char, idx),
                            ))));
                        }
                    };

                    stack.truncate(start);
                    stack.push(val);
                }
                IRpnToken::Op(op) => {
                    let start = stack.len() - op.num_operands();
                    let res = op.apply(&stack[start..]);
                    stack.truncate(start);
                    stack.push(res);
                }
            }
        }

        match stack.pop() {
            Some(result) if stack.is_empty() => Ok(result),
            _ => Err(Error::EvalError(EvalError::MalformedExpression)),
        }
    }
}

impl<'e> TryFrom<InfixExpr<'e>> for IRpnExpr
{
    type Error = crate::Error<'e>;

    // shunting yard algorithm
    fn try_from(expr: InfixExpr<'e>) -> Result<Self, Self::Error>
    {
        let mut f64_cache: SmallVec<[f64; 4]> = smallvec![];
        let mut output: Vec<IRpnToken> = Vec::with_capacity(expr.len());
        let mut ops: Vec<InfixToken> = Vec::new();

        for tok in expr.into_iter() {
            match tok {
                InfixToken::Num(num) => {
                    output.push(IRpnToken::Num(num));
                    f64_cache.push(num);
                }
                InfixToken::Var(name) => {
                    let name_bytes = name.as_bytes();

                    let letter = name_bytes[0] - b'a';
                    let idx = parsing::parse_usize(&name_bytes[1..]);

                    output.push(IRpnToken::Var(letter as usize, idx));
                    f64_cache.clear();
                }
                InfixToken::Op(op) => {
                    while let Some(InfixToken::Op(top)) = ops.last() {
                        let prec = op.precedence();
                        let top_prec = top.precedence();
                        let should_pop =
                            top_prec > prec || (!op.is_right_associative() && top_prec == prec);

                        if should_pop {
                            if let Some(InfixToken::Op(op)) = ops.pop() {
                                pre_evaluate(&mut output, &mut f64_cache, op);
                            }
                        } else {
                            break;
                        }
                    }
                    ops.push(InfixToken::Op(op));
                }
                InfixToken::LParen => ops.push(tok),
                InfixToken::RParen => {
                    while let Some(top) = ops.pop() {
                        match top {
                            InfixToken::LParen => break,
                            InfixToken::Op(op) => pre_evaluate(&mut output, &mut f64_cache, op),
                            _ => {
                                output.push(top.into());
                                f64_cache.clear();
                            }
                        }
                    }
                }
                InfixToken::Fn(name, args) => {
                    let name_bytes = name.as_bytes();

                    let letter = name_bytes[0] - b'a';
                    let idx = parsing::parse_usize(&name_bytes[1..]);

                    let token = IRpnToken::Fn(letter as usize, idx, args.len());

                    for arg_tokens in args {
                        let rpn_arg: IRpnExpr = arg_tokens.try_into()?;
                        output.extend(rpn_arg.tokens);
                    }

                    output.push(token);
                    f64_cache.clear();
                }
            }
        }

        while let Some(top) = ops.pop() {
            if let InfixToken::Op(op) = top {
                pre_evaluate(&mut output, &mut f64_cache, op);
            } else {
                output.push(top.into());
                f64_cache.clear(); // TODO: After this else the if doesn't need to be call because no operator uses 0 elements
            }
        }

        return Ok(IRpnExpr { tokens: output });

        fn pre_evaluate(output: &mut Vec<IRpnToken>, f64_cache: &mut SmallVec<[f64; 4]>, op: Op)
        {
            let n_operands = op.num_operands();

            if f64_cache.len() >= n_operands {
                let output_len = output.len();
                let f64_cache_len = f64_cache.len();

                let start = f64_cache_len - n_operands;
                let num = op.apply(&f64_cache[start..]);
                let token = IRpnToken::Num(num);

                output.truncate(output_len - n_operands + 1);
                output[output_len - n_operands] = token;

                f64_cache.truncate(f64_cache_len - n_operands + 1);
                f64_cache[f64_cache_len - n_operands] = num;
            } else {
                output.push(IRpnToken::Op(op));
                f64_cache.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests
{
    use crate::op::Op;

    use super::*;

    #[test]
    fn test_new()
    {
        let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
        let rpn_expr = IRpnExpr::new(expr).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpnToken::Num(2.0),
                IRpnToken::Num(4.0),
                IRpnToken::Var((b'p' - b'a') as usize, 19),
                IRpnToken::Num(2.0),
                IRpnToken::Op(Op::Sub),
                IRpnToken::Var((b'p' - b'a') as usize, 19),
                IRpnToken::Num(2.0),
                IRpnToken::Op(Op::Add),
                IRpnToken::Op(Op::Mul),
                IRpnToken::Op(Op::Add),
                IRpnToken::Op(Op::Sub)
            ]
        );

        let expr = "f0((2 + 3) * 4, f1(5))";
        let rpn_expr = IRpnExpr::new(expr).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpnToken::Num(20.0),
                IRpnToken::Num(5.0),
                IRpnToken::Fn((b'f' - b'a') as usize, 1, 1),
                IRpnToken::Fn((b'f' - b'a') as usize, 0, 2),
            ]
        );

        let expr = "(2 * 21) + 3 + -35 - ((5 * 80) + 5) + 10 + -p0";
        let rpn_expr = IRpnExpr::new(expr).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpnToken::Num(-385.0),
                IRpnToken::Var((b'p' - b'a') as usize, 0),
                IRpnToken::Op(Op::Neg),
                IRpnToken::Op(Op::Add),
            ]
        );

        let expr = "-y1 * (p2 - p3*y0)";
        let rpn_expr = IRpnExpr::new(expr).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                IRpnToken::Var((b'y' - b'a') as usize, 1),
                IRpnToken::Op(Op::Neg),
                IRpnToken::Var((b'p' - b'a') as usize, 2),
                IRpnToken::Var((b'p' - b'a') as usize, 3),
                IRpnToken::Var((b'y' - b'a') as usize, 0),
                IRpnToken::Op(Op::Mul),
                IRpnToken::Op(Op::Sub),
                IRpnToken::Op(Op::Mul),
            ]
        );
    }
}

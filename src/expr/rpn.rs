use std::borrow::Cow;

use smallvec::{SmallVec, smallvec};

use crate::{
    Error, EvalError, EvalRpn, Expr, context::Context, expr::infix::*, op::Op, prelude::*,
};

#[derive(Debug, PartialEq)]
pub enum RpnToken<'e>
{
    Num(f64),
    Var(&'e str),
    Fn(&'e str, usize),
    Op(Op),
}

impl<'e> From<InfixToken<'e>> for RpnToken<'e>
{
    fn from(token: InfixToken<'e>) -> Self
    {
        match token {
            InfixToken::Num(num) => RpnToken::Num(num),
            InfixToken::Var(name) => RpnToken::Var(name),
            InfixToken::Fn(name, args) => RpnToken::Fn(name, args.len()),
            InfixToken::Op(op) => RpnToken::Op(op),
            _ => unreachable!("logic bug found"),
        }
    }
}

impl<'e, V: Resolver<f64>, F: Resolver<ExprFn>> EvalRpn<V, F> for Expr<RpnToken<'e>>
{
    type Error = Error<'e>;

    fn eval(&self, ctx: &Context<V, F>, stack: &mut Vec<f64>) -> Result<f64, Self::Error>
    {
        if self.tokens.len() == 1 {
            if let RpnToken::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                RpnToken::Num(num) => stack.push(*num),
                RpnToken::Var(name) => stack.push(
                    *ctx.get_var(name)
                        .ok_or(Error::EvalError(EvalError::UnknownVar(Cow::Borrowed(name))))?,
                ),
                RpnToken::Fn(name, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
                    let val = match ctx.call_fn(name, args) {
                        Some(value) => value,
                        None => {
                            return Err(Error::EvalError(EvalError::UnknownFn(Cow::Borrowed(
                                name,
                            ))));
                        }
                    };

                    stack.truncate(start);
                    stack.push(val);
                }
                RpnToken::Op(op) => {
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

impl<'e> Expr<RpnToken<'e>>
{
    pub fn new(input: &'e str) -> Result<Self, Error<'e>>
    {
        let infix_expr = Expr::<InfixToken>::try_from(input)?;
        Self::try_from(infix_expr)
    }
}

impl<'e> TryFrom<&'e str> for Expr<RpnToken<'e>>
{
    type Error = crate::Error<'e>;

    fn try_from(input: &'e str) -> Result<Self, Self::Error>
    {
        let infix_expr = Expr::<InfixToken>::try_from(input)?;
        Self::try_from(infix_expr)
    }
}

impl<'e> TryFrom<Expr<InfixToken<'e>>> for Expr<RpnToken<'e>>
{
    type Error = crate::Error<'e>;

    // shunting yard algorithm
    fn try_from(expr: Expr<InfixToken<'e>>) -> Result<Self, Self::Error>
    {
        let mut f64_cache: SmallVec<[f64; 4]> = smallvec![];
        let mut output: Vec<RpnToken> = Vec::with_capacity(expr.len());
        let mut ops: Vec<InfixToken> = Vec::new();

        for tok in expr.into_iter() {
            match tok {
                InfixToken::Num(num) => {
                    output.push(RpnToken::Num(num));
                    f64_cache.push(num);
                }
                InfixToken::Var(var) => {
                    output.push(RpnToken::Var(var));
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
                    let fun_call_token = RpnToken::Fn(name, args.len());

                    for arg_tokens in args {
                        let rpn_arg: Expr<RpnToken<'e>> = arg_tokens.try_into()?;
                        output.extend(rpn_arg.tokens);
                    }

                    output.push(fun_call_token);
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

        return Ok(Expr { tokens: output });

        fn pre_evaluate<'t>(
            output: &mut Vec<RpnToken<'t>>,
            f64_cache: &mut SmallVec<[f64; 4]>,
            op: Op,
        )
        {
            let n_operands = op.num_operands();

            if f64_cache.len() >= n_operands {
                let output_len = output.len();
                let f64_cache_len = f64_cache.len();

                let start = f64_cache_len - n_operands;
                let num = op.apply(&f64_cache[start..]);
                let token = RpnToken::Num(num);

                output.truncate(output_len - n_operands + 1);
                output[output_len - n_operands] = token;

                f64_cache.truncate(f64_cache_len - n_operands + 1);
                f64_cache[f64_cache_len - n_operands] = num;
            } else {
                output.push(RpnToken::Op(op));
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
        let rpn_expr = Expr::<RpnToken>::try_from(expr).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                RpnToken::Num(2.0),
                RpnToken::Num(4.0),
                RpnToken::Var("p19"),
                RpnToken::Num(2.0),
                RpnToken::Op(Op::Sub),
                RpnToken::Var("p19"),
                RpnToken::Num(2.0),
                RpnToken::Op(Op::Add),
                RpnToken::Op(Op::Mul),
                RpnToken::Op(Op::Add),
                RpnToken::Op(Op::Sub)
            ]
        );

        let expr = "abs((2 + 3) * 4, sqrt(5))";
        let rpn_expr = Expr::<RpnToken>::try_from(expr).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                RpnToken::Num(20.0),
                RpnToken::Num(5.0),
                RpnToken::Fn("sqrt", 1),
                RpnToken::Fn("abs", 2),
            ]
        );

        let expr = "(2 * 21) + 3 + -35 - ((5 * 80) + 5) + 10 + -p0";
        let rpn_expr = Expr::<RpnToken>::try_from(expr).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                RpnToken::Num(-385.0),
                RpnToken::Var("p0"),
                RpnToken::Op(Op::Neg),
                RpnToken::Op(Op::Add),
            ]
        );

        let expr = "-y1 * (p2 - p3*y0)";
        let rpn_expr = Expr::<RpnToken>::try_from(expr).unwrap();
        assert_eq!(
            rpn_expr.tokens,
            vec![
                RpnToken::Var("y1"),
                RpnToken::Op(Op::Neg),
                RpnToken::Var("p2"),
                RpnToken::Var("p3"),
                RpnToken::Var("y0"),
                RpnToken::Op(Op::Mul),
                RpnToken::Op(Op::Sub),
                RpnToken::Op(Op::Mul),
            ]
        );
    }
}

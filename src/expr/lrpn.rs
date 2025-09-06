use crate::{
    Error, EvalError, Ptr,
    op::Op,
    prelude::*,
    resolver::{Locked, LockedResolver},
};

#[derive(Debug, PartialEq)]
pub enum LRpnToken<'a>
{
    Num(f64),
    Var(Ptr<'a, f64>),
    Fn(Ptr<'a, ExprFn>, usize),
    Op(Op),
}

impl<'a> From<f64> for LRpnToken<'a>
{
    fn from(num: f64) -> Self
    {
        LRpnToken::Num(num)
    }
}

impl<'a> From<Ptr<'a, f64>> for LRpnToken<'a>
{
    fn from(ptr: Ptr<'a, f64>) -> Self
    {
        LRpnToken::Var(ptr)
    }
}

impl<'a> From<Op> for LRpnToken<'a>
{
    fn from(op: Op) -> Self
    {
        LRpnToken::Op(op)
    }
}

impl<'a> From<(Ptr<'a, ExprFn>, usize)> for LRpnToken<'a>
{
    fn from((ptr, argc): (Ptr<'a, ExprFn>, usize)) -> Self
    {
        LRpnToken::Fn(ptr, argc)
    }
}

impl<'a, V, F> LRpnExpr<'a, V, F, LRpnToken<'a>> for Expr<LRpnToken<'a>>
where
    V: LockedResolver<f64>,
    F: LockedResolver<ExprFn>,
{
    fn eval_locked(
        &self,
        _ctx: &Context<Locked, V, F, V, F>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'a>>
    {
        if self.tokens.len() == 1 {
            if let LRpnToken::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                LRpnToken::Num(num) => stack.push(*num),
                LRpnToken::Var(ptr) => stack.push(ptr.get()),
                LRpnToken::Fn(ptr, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
                    let val = (ptr.get())(args);

                    stack.truncate(start);
                    stack.push(val);
                }
                LRpnToken::Op(op) => {
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

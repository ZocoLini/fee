use crate::{Error, EvalError, LRpnExpr, op::Op, prelude::*};

#[derive(Debug, PartialEq)]
pub enum LRpnToken
{
    Num(f64),
    Var(VarPtr),
    Fn(FnPtr, usize),
    Op(Op),
}

impl From<f64> for LRpnToken
{
    fn from(num: f64) -> Self
    {
        LRpnToken::Num(num)
    }
}

impl<'e> From<VarPtr> for LRpnToken
{
    fn from(ptr: VarPtr) -> Self
    {
        LRpnToken::Var(ptr)
    }
}

impl From<Op> for LRpnToken
{
    fn from(op: Op) -> Self
    {
        LRpnToken::Op(op)
    }
}

impl<'e> From<(FnPtr, usize)> for LRpnToken
{
    fn from((ptr, argc): (FnPtr, usize)) -> Self
    {
        LRpnToken::Fn(ptr, argc)
    }
}

impl<'e, V, F> LRpnExpr<'e, V, F, LRpnToken> for Expr<LRpnToken>
where
    V: Resolver<Locked, f64> + LockedResolver,
    F: Resolver<Locked, ExprFn> + LockedResolver,
{
    fn eval_locked(&self, stack: &mut Vec<f64>) -> Result<f64, Error<'e>>
    {
        if self.tokens.len() == 1 {
            if let LRpnToken::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                LRpnToken::Num(num) => stack.push(*num),
                LRpnToken::Var(ptr) => stack.push(unsafe { **ptr }),
                LRpnToken::Fn(ptr, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
                    let val = (unsafe { **ptr })(args);

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

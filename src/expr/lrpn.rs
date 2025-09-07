use crate::{
    Error, EvalError, Ptr,
    expr::ExprCompiler,
    op::Op,
    prelude::*,
    resolver::{Locked, LockedResolver},
};

#[derive(Debug, PartialEq)]
pub enum LRpn<'a>
{
    Num(f64),
    Var(Ptr<'a, f64>),
    Fn(Ptr<'a, ExprFn>, usize),
    Op(Op),
}

impl<'a> From<f64> for LRpn<'a>
{
    fn from(num: f64) -> Self
    {
        LRpn::Num(num)
    }
}

impl<'a> From<Ptr<'a, f64>> for LRpn<'a>
{
    fn from(ptr: Ptr<'a, f64>) -> Self
    {
        LRpn::Var(ptr)
    }
}

impl<'a> From<Op> for LRpn<'a>
{
    fn from(op: Op) -> Self
    {
        LRpn::Op(op)
    }
}

impl<'a> From<(Ptr<'a, ExprFn>, usize)> for LRpn<'a>
{
    fn from((ptr, argc): (Ptr<'a, ExprFn>, usize)) -> Self
    {
        LRpn::Fn(ptr, argc)
    }
}

impl<'e: 'c, 'c: 'e, V, F> ExprCompiler<'e, 'c, Locked, V, F, V, F, LRpn<'c>> for Expr<LRpn<'c>>
where
    V: LockedResolver<f64>,
    F: LockedResolver<ExprFn>,
{
    fn compile(
        expr: &'e str,
        ctx: &'c Context<Locked, V, F, V, F>,
    ) -> Result<Expr<LRpn<'c>>, Error<'e>>
    {
        Expr::try_from((expr, ctx))
    }
}

impl<'a, V, F> ExprEvaluator<'a, Locked, V, F, V, F> for Expr<LRpn<'a>>
where
    V: LockedResolver<f64>,
    F: LockedResolver<ExprFn>,
{
    fn eval(
        &self,
        _ctx: &Context<Locked, V, F, V, F>,
        stack: &mut Vec<f64>,
    ) -> Result<f64, Error<'a>>
    {
        if self.tokens.len() == 1 {
            if let LRpn::Num(num) = &self.tokens[0] {
                return Ok(*num);
            }
        }

        for tok in self.tokens.iter() {
            match tok {
                LRpn::Num(num) => stack.push(*num),
                LRpn::Var(ptr) => stack.push(ptr.get()),
                LRpn::Fn(ptr, argc) => {
                    if *argc > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - argc;
                    let args = &stack[start..];
                    let val = (ptr.get())(args);

                    stack.truncate(start);
                    stack.push(val);
                }
                LRpn::Op(op) => {
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

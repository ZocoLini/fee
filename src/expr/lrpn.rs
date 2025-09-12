use crate::{
    Error, EvalError, LContext, Ptr,
    expr::{Op, ParseableToken},
    prelude::*,
    resolver::LockedResolver,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LRpn<'a>
{
    Num(f64),
    Var(Ptr<'a, f64>),
    Fn(Ptr<'a, ExprFn>, usize),
    Op(Op),
}

impl<'a, 'c, V, F> ParseableToken<'a, 'c, Locked, V, F, V, F> for LRpn<'c>
where
    V: LockedResolver<f64>,
    F: LockedResolver<ExprFn>,
{
    #[inline]
    fn f64(num: f64) -> Self
    {
        LRpn::Num(num)
    }

    #[inline]
    fn i64(num: i64) -> Self
    {
        LRpn::Num(num as f64)
    }

    #[inline]
    fn bool(val: bool) -> Self
    {
        LRpn::Num(if val { 1.0 } else { 0.0 })
    }

    #[inline]
    fn op(op: Op) -> Self
    {
        LRpn::Op(op)
    }

    // TODO: Return an error manin
    #[inline]
    fn var(name: &'a str, ctx: &'c LContext<V, F>) -> Self
    {
        LRpn::Var(ctx.get_var_ptr(name).unwrap())
    }

    #[inline]
    fn fun(name: &'a str, argc: usize, ctx: &'c LContext<V, F>) -> Self
    {
        LRpn::Fn(ctx.get_fn_ptr(name).unwrap(), argc)
    }
}

impl<'e, 'c, V, F> ExprCompiler<'e, 'c, Locked, V, F, V, F, LRpn<'c>> for Expr<LRpn<'c>>
where
    V: LockedResolver<f64>,
    F: LockedResolver<ExprFn>,
{
    fn compile(expr: &'e str, ctx: &'c LContext<V, F>) -> Result<Expr<LRpn<'c>>, Error<'e>>
    {
        Expr::try_from((expr, ctx))
    }
}

impl<'a, V, F> ExprEvaluator<'a, Locked, V, F, V, F> for Expr<LRpn<'a>>
where
    V: LockedResolver<f64>,
    F: LockedResolver<ExprFn>,
{
    fn eval(&self, _ctx: &LContext<V, F>, stack: &mut Vec<f64>) -> Result<f64, Error<'a>>
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
                    let args = unsafe { stack.get_unchecked(start..) };
                    let val = (ptr.get())(args);

                    stack.truncate(start);
                    stack.push(val);
                }
                LRpn::Op(op) => {
                    if op.num_operands() > stack.len() {
                        return Err(Error::EvalError(EvalError::RPNStackUnderflow));
                    }

                    let start = stack.len() - op.num_operands();
                    let args = unsafe { stack.get_unchecked(start..) };
                    let res = op.apply(args);
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

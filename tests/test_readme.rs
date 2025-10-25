use fee::{prelude::*, DefaultResolver};

fn main()
{
    let expr = "max((2 + 4) * 6 / (p1 + 2), sqrt(p0^2 + 21)) + abs(-p1)";

    let mut var_resolver = DefaultResolver::empty();
    var_resolver.insert("p0", 10.0);
    var_resolver.insert("p1", 4.0);

    let mut fn_resolver = DefaultResolver::empty();
    fn_resolver.insert("abs", ExprFn::new(abs));
    fn_resolver.insert("max", ExprFn::new(max));
    fn_resolver.insert("sqrt", ExprFn::new(sqrt));

    let context = Context::new(var_resolver, fn_resolver);
    let mut stack = Vec::with_capacity(10);

    let expr = Expr::compile(expr, &context).unwrap();

    let result = expr.eval(&context, &mut stack).unwrap();
    assert_eq!(result, 15.0);
}

fn max(x: &[f64]) -> f64 {
    let mut max = x[0];
    for i in 1..x.len() {
        if x[i] > max {
            max = x[i];
        }
    }
    max
}

fn sqrt(x: &[f64]) -> f64 {
    x[0].sqrt()
}

fn abs(x: &[f64]) -> f64 {
   x[0].abs()
}
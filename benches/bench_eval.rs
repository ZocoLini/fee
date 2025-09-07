use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fee::{IndexedResolver, SmallResolver, prelude::*};

static EXPR: &str =
    "f0((2 * 21) + 3 - 35 - ((5 * 80) + 5) + p0, f0((2 * 21) + 3 - 35 - ((5 * 80) + 5) + p0))";

fn rpn_eval(c: &mut Criterion)
{
    let mut stack = Vec::with_capacity(EXPR.len() / 2);

    c.bench_function("internal/eval/rpn", |b| {
        let mut var_resolver = SmallResolver::new();
        var_resolver.insert("p0".to_string(), 10.0);

        let mut fn_resolver = SmallResolver::new();
        fn_resolver.insert("f0".to_string(), ExprFn::new(abs));

        let context = Context::new(var_resolver, fn_resolver);
        let expr = Expr::compile(EXPR, &context).unwrap();

        b.iter(|| {
            black_box(expr.eval(&context, &mut stack).unwrap());
        });
    });
}

fn irpn_eval(c: &mut Criterion)
{
    let mut stack = Vec::with_capacity(EXPR.len() / 2);

    c.bench_function("internal/eval/irpn", |b| {
        let mut var_resolver = IndexedResolver::new();
        var_resolver.add_id('p', 1);
        var_resolver.set('p', 0, 10.0);

        let mut fn_resolver = IndexedResolver::new();
        fn_resolver.add_id('f', 1);
        fn_resolver.set('f', 0, ExprFn::new(abs));

        let context = Context::new(var_resolver, fn_resolver);
        let expr = Expr::compile(EXPR, &context).unwrap();

        b.iter(|| {
            black_box(expr.eval(&context, &mut stack).unwrap());
        });
    });
}

fn lrpn_eval(c: &mut Criterion)
{
    let mut stack = Vec::with_capacity(EXPR.len() / 2);

    c.bench_function("internal/eval/lrpn", |b| {
        let mut var_resolver = IndexedResolver::new();
        var_resolver.add_id('p', 1);
        var_resolver.set('p', 0, 10.0);

        let mut fn_resolver = IndexedResolver::new();
        fn_resolver.add_id('f', 1);
        fn_resolver.set('f', 0, ExprFn::new(abs));

        let context = Context::new(var_resolver, fn_resolver).lock();
        let expr = Expr::compile(EXPR, &context).unwrap();

        b.iter(|| {
            black_box(expr.eval(&context, &mut stack).unwrap());
        });
    });
}

fn abs(x: &[f64]) -> f64
{
    x[0].abs()
}

criterion_group!(benches, rpn_eval, irpn_eval, lrpn_eval);
criterion_main!(benches);

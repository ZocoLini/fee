use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fee::{
    ConstantResolver, DefaultResolver, ExprFn, IndexedResolver, RpnEvaluator, SmallResolver,
    prelude::*,
};

fn rpn_evaluator(c: &mut Criterion)
{
    let expr = "abs((2 * 21) + 3 - 35 - ((5 * 80) + 5) + p0)";

    c.bench_function("internal/eval/rpn/default_r", |b| {
        let mut var_resolver = DefaultResolver::new_empty();
        var_resolver.insert("p0".to_string(), 10.0);

        let mut fn_resolver = DefaultResolver::new_empty();
        fn_resolver.insert("abs".to_string(), abs as ExprFn);

        let context = Context::new(var_resolver, fn_resolver);
        let mut stack = Vec::with_capacity(expr.len() / 2);
        let evaluator = RpnEvaluator::new(expr).unwrap();

        b.iter(|| {
            black_box(evaluator.eval_with_stack(&context, &mut stack).unwrap());
        });
    });

    c.bench_function("internal/eval/rpn/constant_r", |b| {
        let var_resolver = ConstantResolver::new(10.0);
        let fn_resolver = ConstantResolver::new(abs as ExprFn);

        let context = Context::new(var_resolver, fn_resolver);
        let mut stack = Vec::with_capacity(expr.len() / 2);
        let evaluator = RpnEvaluator::new(expr).unwrap();

        b.iter(|| {
            black_box(evaluator.eval_with_stack(&context, &mut stack).unwrap());
        });
    });

    c.bench_function("internal/eval/rpn/indexed_r", |b| {
        let expr = "p0((2 * 21) + 3 - 35 - ((5 * 80) + 5) + p0)";

        let mut var_resolver = IndexedResolver::new_var_resolver();
        var_resolver.add_var_identifier('p', 1);
        var_resolver.set('p', 0, 10.0);

        let mut fn_resolver = IndexedResolver::new_fn_resolver();
        fn_resolver.add_fn_identifier('p', 1);
        fn_resolver.set('p', 0, abs as ExprFn);

        let context = Context::new(var_resolver, fn_resolver);
        let mut stack = Vec::with_capacity(expr.len() / 2);
        let evaluator = RpnEvaluator::new(expr).unwrap();

        b.iter(|| {
            black_box(evaluator.eval_with_stack(&context, &mut stack).unwrap());
        });
    });

    c.bench_function("internal/eval/rpn/small_r", |b| {
        let mut var_resolver = SmallResolver::new();
        var_resolver.insert("p0".to_string(), 10.0);

        let mut fn_resolver = SmallResolver::new();
        fn_resolver.insert("abs".to_string(), abs as ExprFn);

        let context = Context::new(var_resolver, fn_resolver);
        let mut stack = Vec::with_capacity(expr.len() / 2);
        let evaluator = RpnEvaluator::new(expr).unwrap();

        b.iter(|| {
            black_box(evaluator.eval_with_stack(&context, &mut stack).unwrap());
        });
    });
}

fn abs(x: &[f64]) -> f64
{
    x[0].abs()
}

criterion_group!(benches, rpn_evaluator,);
criterion_main!(benches);

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fee::{
    DefaultContext, DefaultFnResolver, DefaultVarResolver, IndexedVarResolver, RpnEvaluator,
    prelude::*,
};

fn rpn_evaluator(c: &mut Criterion)
{
    let expr = "abs((2 * 21) + 3 - 35 - ((5 * 80) + 5) + p0)";

    c.bench_function("eval/rpn/default_vr", |b| {
        let mut var_resolver = DefaultVarResolver::new_empty();
        var_resolver.add_var("p0".to_string(), 10.0);

        let mut fn_resolver = DefaultFnResolver::new();
        fn_resolver.add_fn("abs".to_string(), |x| x[0].abs());

        let context = DefaultContext::new(var_resolver, fn_resolver);
        let evaluator = RpnEvaluator::new(expr).unwrap();

        b.iter(|| {
            black_box(evaluator.eval(&context).unwrap());
        });
    });

    c.bench_function("eval/rpn/indexed_vr", |b| {
        let mut var_resolver = IndexedVarResolver::new();
        var_resolver.add_identifier('p', 1);
        var_resolver.set('p', 0, 10.0);

        let mut fn_resolver = DefaultFnResolver::new();
        fn_resolver.add_fn("abs".to_string(), |x| x[0].abs());

        let context = DefaultContext::new(var_resolver, fn_resolver);
        let evaluator = RpnEvaluator::new(expr).unwrap();

        b.iter(|| {
            black_box(evaluator.eval(&context).unwrap());
        });
    });

    c.bench_function("eval/rpn/rust_native", |b| {
        b.iter(|| {
            black_box((((2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10) as i32).abs());
        });
    });
}

criterion_group!(benches, rpn_evaluator,);
criterion_main!(benches);

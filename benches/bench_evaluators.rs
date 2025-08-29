use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fee::{DefaultFnResolver, DefaultVarResolver, RPNEvaluator, prelude::*};

fn rpn_evaluator(c: &mut Criterion)
{
    c.bench_function("eval/rpn", |b| {
        let var_resolver = DefaultVarResolver::new();
        let fn_resolver = DefaultFnResolver::new();

        let expr = "(2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10";
        let context = Context::new(var_resolver, fn_resolver);
        let evaluator = RPNEvaluator::new(expr, &context).unwrap();

        b.iter(|| {
            black_box(evaluator.eval());
        });
    });

    c.bench_function("eval/rust_native", |b| {
        b.iter(|| {
            black_box((2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10);
        });
    });
}

criterion_group!(benches, rpn_evaluator,);
criterion_main!(benches);

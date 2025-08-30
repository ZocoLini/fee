use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fee::{DefaultFnResolver, DefaultVarResolver, RPNEvaluator, prelude::*};

fn evaluation(c: &mut Criterion)
{
    let expr = "(2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10"; // = -385

    c.bench_function("cmp/evalexpr/eval", |b| {
        use evalexpr::*;

        let precompiled = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();

        b.iter(|| black_box(precompiled.eval_int()));
    });

    c.bench_function("cmp/meval/eval", |b| {
        let expr: meval::Expr = expr.parse().unwrap();

        b.iter(|| {
            black_box(expr.eval().unwrap());
        });
    });

    c.bench_function("cmp/mexe/eval", |b| {
        b.iter(|| {
            black_box(mexe::eval(expr).unwrap());
        });
    });

    c.bench_function("cmp/fee/eval", |b| {
        let var_resolver = DefaultVarResolver::new();
        let fn_resolver = DefaultFnResolver::new();

        let mut context = Context::new(var_resolver, fn_resolver);
        let evaluator = RPNEvaluator::new(expr, &mut context).unwrap();

        b.iter(|| {
            black_box(evaluator.eval());
        });
    });
}

criterion_group!(benches, evaluation,);
criterion_main!(benches);

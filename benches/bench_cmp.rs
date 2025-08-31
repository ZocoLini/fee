use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use evalexpr::{DefaultNumericTypes, build_operator_tree};
use fee::{DefaultContext, DefaultFnResolver, DefaultVarResolver, RpnEvaluator, prelude::*};

fn evaluation(c: &mut Criterion)
{
    let expr = "(2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10"; // = -385

    c.bench_function("cmp/eval/evalexpr", |b| {
        use evalexpr::*;

        let precompiled = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();

        b.iter(|| black_box(precompiled.eval_int()));
    });

    c.bench_function("cmp/eval/meval", |b| {
        let expr: meval::Expr = expr.parse().unwrap();

        b.iter(|| {
            black_box(expr.eval().unwrap());
        });
    });

    c.bench_function("cmp/eval/mexe", |b| {
        b.iter(|| {
            black_box(mexe::eval(expr).unwrap());
        });
    });

    c.bench_function("cmp/eval/fee", |b| {
        let var_resolver = DefaultVarResolver::new();
        let fn_resolver = DefaultFnResolver::new();

        let context = DefaultContext::new(var_resolver, fn_resolver);
        let evaluator = RpnEvaluator::new(expr).unwrap();

        b.iter(|| {
            black_box(evaluator.eval(&context).unwrap());
        });
    });
}

fn parse(c: &mut Criterion)
{
    let expr = "(2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10"; // = -385

    c.bench_function("cmp/parse/evalexpr", |b| {
        b.iter(|| black_box(build_operator_tree::<DefaultNumericTypes>(expr).unwrap()));
    });

    c.bench_function("cmp/parse/meval", |b| {
        b.iter(|| {
            black_box(expr.parse::<meval::Expr>().unwrap());
        });
    });

    c.bench_function("cmp/parse/mexe", |b| {
        b.iter(|| {
            black_box(mexe::eval(expr).unwrap());
        });
    });

    c.bench_function("cmp/parse/fee", |b| {
        b.iter(|| {
            black_box(RpnEvaluator::new(expr).unwrap());
        });
    });
}

criterion_group!(benches, evaluation, parse);
criterion_main!(benches);

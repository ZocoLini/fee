use std::{collections::BTreeMap, hint::black_box};

use criterion::{Criterion, criterion_group, criterion_main};
use evalexpr::{DefaultNumericTypes, build_operator_tree};
use fasteval::{Compiler, Evaler};
use fee::{DefaultResolver, RpnEvaluator, prelude::*};

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

    c.bench_function("cmp/eval/fasteval", |b| {
        let parser = fasteval::Parser::new();
        let mut slab = fasteval::Slab::new();
        let mut map: BTreeMap<String, f64> = BTreeMap::new();

        let expr_ref = parser.parse(expr, &mut slab.ps).unwrap().from(&slab.ps);
        let compiled = expr_ref.compile(&slab.ps, &mut slab.cs);

        b.iter(|| black_box(compiled.eval(&slab, &mut map).unwrap()));
    });

    c.bench_function("cmp/eval/rust", |b| {
        b.iter(|| {
            black_box((2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10);
        });
    });

    c.bench_function("cmp/eval/fee", |b| {
        let var_resolver = DefaultResolver::new_var_resolver();
        let fn_resolver = DefaultResolver::new_fn_resolver();

        let context = Context::new(var_resolver, fn_resolver);
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

    c.bench_function("cmp/parse/fasteval", |b| {
        let parser = fasteval::Parser::new();

        b.iter_batched(
            || fasteval::Slab::new(),
            |mut slab| {
                let expr_ref = parser.parse(expr, &mut slab.ps).unwrap().from(&slab.ps);
                let _ = expr_ref.compile(&slab.ps, &mut slab.cs);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    c.bench_function("cmp/parse/fee", |b| {
        b.iter(|| {
            black_box(RpnEvaluator::new(expr).unwrap());
        });
    });
}

criterion_group!(benches, evaluation, parse);
criterion_main!(benches);

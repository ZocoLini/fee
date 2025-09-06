use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use evalexpr::{DefaultNumericTypes, build_operator_tree};
use fasteval::{CachedCallbackNamespace, Compiler, EmptyNamespace, Evaler};
use fee::{IndexedResolver, prelude::*};

fn evaluation(c: &mut Criterion)
{
    let expr = "(2 * 21) + 3 - p0 - f0((5 * p1) + 5) + y0"; // = -385

    let p0 = 35.0;
    let p1 = 80.0;
    let y0 = 10.0;

    c.bench_function("cmp/eval/evalexpr", |b| {
        use evalexpr::*;

        let precompiled = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();
        let context: HashMapContext<DefaultNumericTypes> = context_map! {
            "p0" => float p0,
            "p1" => float p1,
            "y0" => float y0,
            "f0" => Function::new(|argument| {
                if let Ok(int) = argument.as_int() {
                            Ok(Value::Int(int))
                        } else if let Ok(float) = argument.as_float() {
                            Ok(Value::Float(float))
                        } else {
                            Err(EvalexprError::expected_number(argument.clone()))
                        }
            }),
        }
        .unwrap();

        b.iter(|| black_box(precompiled.eval_int_with_context(&context)));
    });

    c.bench_function("cmp/eval/meval", |b| {
        let expr: meval::Expr = expr.parse().unwrap();
        let mut context = meval::Context::empty();

        context
            .var("y0", y0)
            .var("p0", p0)
            .var("p1", p1)
            .func("f0", |arg| arg);

        b.iter(|| {
            black_box(expr.eval_with_context(&context).unwrap());
        });
    });

    c.bench_function("cmp/eval/fasteval", |b| {
        let mut slab = fasteval::Slab::new();
        let parser = fasteval::Parser::new();
        let expr = parser.parse(expr, &mut slab.ps).unwrap();
        let compiled = expr.from(&slab.ps).compile(&slab.ps, &mut slab.cs);

        let mut ns = CachedCallbackNamespace::new(|name, args| match name {
            "f0" => Some(args[0]),
            "p0" => Some(p0),
            "p1" => Some(p1),
            "y0" => Some(y0),
            _ => unreachable!("Unknown function: {}", name),
        });

        b.iter(|| {
            black_box(compiled.eval(&slab, &mut ns).unwrap());
        });
    });

    c.bench_function("cmp/eval/rust", |b| {
        b.iter(|| {
            black_box((2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10);
        });
    });

    c.bench_function("cmp/eval/fee", |b| {
        let mut var_resolver = IndexedResolver::new();
        var_resolver.add_id('p', 2);
        var_resolver.set('p', 0, p0);
        var_resolver.set('p', 1, p1);
        var_resolver.add_id('y', 1);
        var_resolver.set('y', 0, y0);

        let mut fn_resolver = IndexedResolver::new();
        fn_resolver.add_id('f', 1);
        fn_resolver.set('f', 0, ExprFn::new(|args| args[0]));

        let context = Context::new(var_resolver, fn_resolver).lock();
        let mut stack = Vec::with_capacity(expr.len() / 2);

        let expr = Expr::compile_locked(expr, &context).unwrap();

        b.iter(|| {
            black_box(expr.eval_locked(&context, &mut stack).unwrap());
        });
    });
}

fn evaluation2(c: &mut Criterion)
{
    let expr = "(2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10"; // = -385

    c.bench_function("cmp/eval2/evalexpr", |b| {
        use evalexpr::*;

        let precompiled = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();
        let context: HashMapContext<DefaultNumericTypes> = HashMapContext::new();

        b.iter(|| black_box(precompiled.eval_int_with_context(&context)));
    });

    c.bench_function("cmp/eval2/meval", |b| {
        let expr: meval::Expr = expr.parse().unwrap();
        let context = meval::Context::empty();

        b.iter(|| {
            black_box(expr.eval_with_context(&context).unwrap());
        });
    });

    c.bench_function("cmp/eval2/fasteval", |b| {
        let mut slab = fasteval::Slab::new();
        let parser = fasteval::Parser::new();
        let expr = parser.parse(expr, &mut slab.ps).unwrap();
        let compiled = expr.from(&slab.ps).compile(&slab.ps, &mut slab.cs);

        let mut ns = EmptyNamespace;

        b.iter(|| {
            black_box(compiled.eval(&slab, &mut ns).unwrap());
        });
    });

    c.bench_function("cmp/eval2/rust", |b| {
        b.iter(|| {
            black_box((2 * 21) + 3 - 35 - ((5 * 80) + 5) + 10);
        });
    });

    c.bench_function("cmp/eval2/fee", |b| {
        let context = Context::empty().lock();
        let mut stack = Vec::with_capacity(expr.len() / 2);

        let expr = Expr::compile_locked(expr, &context).unwrap();

        b.iter(|| {
            black_box(expr.eval_locked(&context, &mut stack).unwrap());
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

    c.bench_function("cmp/parse/fee/rpn", |b| {
        let context = Context::empty();
        b.iter(|| {
            black_box(Expr::compile_unlocked(expr, &context).unwrap());
        });
    });

    c.bench_function("cmp/parse/fee/irpn", |b| {
        let v_resolver = IndexedResolver::new();
        let f_resolver = IndexedResolver::new();
        let context = Context::new(v_resolver, f_resolver);
        b.iter(|| {
            black_box(Expr::compile_unlocked(expr, &context).unwrap());
        });
    });

    c.bench_function("cmp/parse/fee/lrpn", |b| {
        let context = Context::empty().lock();
        b.iter(|| {
            black_box(Expr::compile_locked(expr, &context).unwrap());
        });
    });
}

criterion_group!(benches, evaluation, evaluation2, parse);
criterion_main!(benches);

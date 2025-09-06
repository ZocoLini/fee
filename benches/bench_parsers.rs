use criterion::{Criterion, criterion_group, criterion_main};
use fee::{IndexedResolver, prelude::*};
use std::hint::black_box;

fn parsers(c: &mut Criterion)
{
    let expr = "(2 * 21)
        + abs((2 + 3) * 4, sqrt(5))
        + 3 - 35 - ((5 * 80) + 5)
        + 10
        - abs((2 + 3) * 4, sqrt(5))";

    c.bench_function("internal/parse/rpn", |b| {
        let context = Context::empty();
        b.iter(|| {
            black_box(Expr::compile(expr, &context).unwrap());
        });
    });

    c.bench_function("internal/parse/irpn", |b| {
        let v_resolver = IndexedResolver::new();
        let f_resolver = IndexedResolver::new();
        let context = Context::new(v_resolver, f_resolver);
        b.iter(|| {
            black_box(Expr::compile(expr, &context).unwrap());
        });
    });

    c.bench_function("internal/parse/lrpn", |b| {
        let context = Context::empty().lock();
        b.iter(|| {
            black_box(Expr::compile(expr, &context).unwrap());
        });
    });
}

criterion_group!(benches, parsers);
criterion_main!(benches);

use criterion::{Criterion, criterion_group, criterion_main};
use fee::RpnToken;
use fee::{IRpnToken, prelude::*};
use std::hint::black_box;

fn parsers(c: &mut Criterion)
{
    let expr = "(2 * 21)
        + abs((2 + 3) * 4, sqrt(5))
        + 3 - 35 - ((5 * 80) + 5)
        + 10
        - abs((2 + 3) * 4, sqrt(5))";

    c.bench_function("internal/parse/rpn", |b| {
        b.iter(|| {
            black_box(Expr::<RpnToken>::try_from(expr).unwrap());
        });
    });

    c.bench_function("internal/parse/irpn", |b| {
        b.iter(|| {
            black_box(Expr::<IRpnToken>::try_from(expr).unwrap());
        });
    });
}

criterion_group!(benches, parsers);
criterion_main!(benches);

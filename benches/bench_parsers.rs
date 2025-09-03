#[cfg(not(feature = "bench-internal"))]
fn main() {}

#[cfg(feature = "bench-internal")]
use criterion::{Criterion, criterion_group, criterion_main};
#[cfg(feature = "bench-internal")]
use fee::benches;
#[cfg(feature = "bench-internal")]
use std::hint::black_box;

#[cfg(feature = "bench-internal")]
fn parsers(c: &mut Criterion)
{
    let expr = "(2 * 21)
        + abs((2 + 3) * 4, sqrt(5))
        + 3 - 35 - ((5 * 80) + 5)
        + 10
        - abs((2 + 3) * 4, sqrt(5))";

    c.bench_function("internal/parse/infix", |b| {
        b.iter(|| {
            black_box(benches::parse_infix(expr).unwrap());
        });
    });

    c.bench_function("internal/parse/rpn", |b| {
        b.iter(|| {
            black_box(benches::parse_rpn(expr).unwrap());
        });
    });

    c.bench_function("internal/parse/irpn", |b| {
        b.iter(|| {
            black_box(benches::parse_irpn(expr).unwrap());
        });
    });
}

#[cfg(feature = "bench-internal")]
criterion_group!(benches, parsers);
#[cfg(feature = "bench-internal")]
criterion_main!(benches);

#[cfg(feature = "bench-internal")]
use criterion::{criterion_group, criterion_main, Criterion};

#[cfg(feature = "bench-internal")]
mod internal {
    use super::*;
    use std::hint::black_box;

    fn parsers(c: &mut Criterion) {
        c.bench_function("parse/infix", |b| {
            let expr = "(2 * 21)
                + abs((2 + 3) * 4, sqrt(5))
                + 3 - 35 - ((5 * 80) + 5)
                + 10
                - abs((2 + 3) * 4, sqrt(5))";

            b.iter(|| {
                black_box(fee::lexer::Expr::try_from(expr).unwrap());
            });
        });
    }

    criterion_group!(benches, parsers);
}

#[cfg(feature = "bench-internal")]
criterion_main!(internal::benches);

#[cfg(not(feature = "bench-internal"))]
fn main() {}

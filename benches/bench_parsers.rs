use criterion::{Criterion, criterion_group, criterion_main};
use fee::benches;
use std::hint::black_box;

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
        b.iter_batched(
            || benches::parse_infix(expr).unwrap(),
            |expr| {
                black_box(benches::parse_rpn(expr).unwrap());
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, parsers);
criterion_main!(benches);

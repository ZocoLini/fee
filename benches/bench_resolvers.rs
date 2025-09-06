use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fee::{ConstantResolver, DefaultResolver, IndexedResolver, SmallResolver, prelude::*};

fn var_resolver(c: &mut Criterion)
{
    c.bench_function("internal/resolver/default", |b| {
        let mut resolver = DefaultResolver::new_empty();
        for i in 0..100 {
            resolver.insert(format!("p{}", i), 2.0);
        }

        b.iter(|| {
            black_box(resolver.resolve("p99").unwrap());
            black_box(resolver.resolve("p50").unwrap());
            black_box(resolver.resolve("p1").unwrap());
        });
    });

    c.bench_function("internal/resolver/indexed", |b| {
        let mut resolver = IndexedResolver::new_var_resolver();
        resolver.add_var_identifier('p', 10);
        for i in 0..10 {
            resolver.set('p', i, 2.0);
        }

        b.iter(|| {
            black_box(resolver.resolve("p9").unwrap());
            black_box(resolver.resolve("p5").unwrap());
            black_box(resolver.resolve("p1").unwrap());
        });
    });

    c.bench_function("internal/resolver/small", |b| {
        let mut resolver = SmallResolver::new();
        for i in 0..10 {
            resolver.insert(format!("p{}", i), 2.0);
        }

        b.iter(|| {
            black_box(resolver.resolve("p9").unwrap());
            black_box(resolver.resolve("p5").unwrap());
            black_box(resolver.resolve("p1").unwrap());
        });
    });

    c.bench_function("internal/resolver/constant", |b| {
        let resolver = ConstantResolver::new(2.0);

        b.iter(|| {
            black_box(resolver.resolve("p9").unwrap());
            black_box(resolver.resolve("p5").unwrap());
            black_box(resolver.resolve("p1").unwrap());
        });
    });

    c.bench_function("internal/resolver/rust", |b| {
        let resolver = vec![0.0; 100];

        b.iter(|| {
            black_box(resolver[99]);
            black_box(resolver[50]);
            black_box(resolver[1]);
        });
    });
}

criterion_group!(benches, var_resolver,);
criterion_main!(benches);

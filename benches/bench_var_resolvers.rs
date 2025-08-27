use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use fee::{DefaultVarResolver, IndexedVarResolver, prelude::*};

fn default_var_resolver(c: &mut Criterion)
{
    c.bench_function("default var resolver", |b| {
        b.iter_batched(
            || {
                let mut resolver = DefaultVarResolver::new();
                for i in 0..100 {
                    resolver.add_var(format!("p{}", i), 2.0);
                }
                resolver
            },
            |resolver| {
                black_box(resolver.get("p99").unwrap());
                black_box(resolver.get("p50").unwrap());
                black_box(resolver.get("p1").unwrap());
            },
            BatchSize::SmallInput,
        )
    });
}

fn indexed_var_resolver(c: &mut Criterion)
{
    c.bench_function("indexed var resolver", |b| {
        b.iter_batched(
            || {
                let mut resolver = IndexedVarResolver::new(100);
                for i in 0..100 {
                    resolver.set(i, 2.0);
                }
                resolver
            },
            |resolver| {
                black_box(resolver.get("_99").unwrap());
                black_box(resolver.get("_50").unwrap());
                black_box(resolver.get("_1").unwrap());
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, default_var_resolver, indexed_var_resolver);
criterion_main!(benches);

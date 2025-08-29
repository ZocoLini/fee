use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fee::{DefaultVarResolver, IndexedVarResolver, prelude::*};

fn default_var_resolver(c: &mut Criterion)
{
    c.bench_function("var_resolver/default", |b| {
        let mut resolver = DefaultVarResolver::new();
        for i in 0..100 {
            resolver.add_var(format!("p{}", i), 2.0);
        }

        b.iter(|| {
            black_box(resolver.get_var("p99").unwrap());
            black_box(resolver.get_var("p50").unwrap());
            black_box(resolver.get_var("p1").unwrap());
        });
    });
}

fn indexed_var_resolver(c: &mut Criterion)
{
    c.bench_function("var_resolver/indexed", |b| {
        let mut resolver = IndexedVarResolver::new();
        resolver.add_identifier('p', 100);
        for i in 0..100 {
            resolver.set('p', i, 2.0);
        }

        b.iter(|| {
            black_box(resolver.get_var("p99").unwrap());
            black_box(resolver.get_var("p50").unwrap());
            black_box(resolver.get_var("p1").unwrap());
        });
    });
}

fn target_speed(c: &mut Criterion)
{
    c.bench_function("var_resolver/rust_native", |b| {
        let resolver = vec![0.0; 100];

        b.iter(|| {
            black_box(resolver[99]);
            black_box(resolver[50]);
            black_box(resolver[1]);
        });
    });
}

criterion_group!(
    benches,
    default_var_resolver,
    indexed_var_resolver,
    target_speed
);
criterion_main!(benches);

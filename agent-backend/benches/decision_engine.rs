//! Decision-engine benchmark scaffold.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_decision_eval(c: &mut Criterion) {
    let mut group = c.benchmark_group("decision_engine");

    group.bench_function("decision_eval_noop", |b| {
        b.iter(|| {
            black_box(true)
        });
    });

    group.finish();
}

criterion_group!(benches, bench_decision_eval);
criterion_main!(benches);

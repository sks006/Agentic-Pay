//! Microbenchmarks for state.rs measuring CPU contention and DashMap concurrency.

use agent_backend::state::{EngineConfig, NoncePhase, ServerState};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn bench_nonce_lifecycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_machine");

    let config = EngineConfig {
        max_in_flight_per_agent: 1_000_000,
        nonce_ttl: Duration::from_secs(600),
        max_rate_limit_units: 10_000_000,
        rate_limit_per_sec: 10_000_000.0,
    };
    let state = Arc::new(ServerState::new(Some(config)));
    let agent = [1u8; 32];
    let dummy_sig = [42u8; 64];
    let nonce_counter = AtomicU64::new(1);

    group.bench_function("nonce_acquire_and_commit", |b| {
        b.iter(|| {
            let nonce = nonce_counter.fetch_add(1, Ordering::Relaxed);
            let guard = state
                .try_acquire_nonce_sync(black_box(agent), black_box(nonce))
                .unwrap();
            guard
                .commit(black_box(NoncePhase::IntentSigned(dummy_sig)))
                .unwrap();
        });
    });

    // Pre-insert a permanent nonce outside the benchmark iterations
    let dup_nonce = 999_999_999u64;
    let _ = state
        .try_acquire_nonce_sync(agent, dup_nonce)
        .map(|guard| guard.commit(NoncePhase::IntentSigned(dummy_sig)));

    group.bench_function("nonce_duplicate_rejection", |b| {
        b.iter(|| {
            let res = state.try_acquire_nonce_sync(black_box(agent), black_box(dup_nonce));
            assert!(res.is_err());
        });
    });

    group.finish();
}

criterion_group!(benches, bench_nonce_lifecycle);
criterion_main!(benches);

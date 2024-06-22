use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;

use ws_replier::{find_nonce, find_nonce_parallel};

fn benchmark_find_nonce(c: &mut Criterion) {
    let token = CancellationToken::new();

    c.bench_function("find_nonce", |b| {
        b.iter_batched(
            || (rand::random(), rand::random::<u8>() % 16),
            |(rand, zeros)| find_nonce(rand, zeros, token.clone()),
            BatchSize::SmallInput,
        )
    });
}

fn benchmark_find_nonce_parallel(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("find_nonce_parallel", |b| {
        b.to_async(&rt).iter_batched(
            || {
                (
                    rand::random(),
                    rand::random::<u8>() % 16,
                    CancellationToken::new(),
                )
            },
            |(rand, zeros, token)| find_nonce_parallel(rand, zeros, token),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, benchmark_find_nonce, benchmark_find_nonce_parallel);
criterion_main!(benches);

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use tokio_util::sync::CancellationToken;

use ws_replyer::find_nonce;

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

criterion_group!(benches, benchmark_find_nonce);
criterion_main!(benches);

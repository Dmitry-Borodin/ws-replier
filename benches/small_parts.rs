use criterion::{Criterion, criterion_group, criterion_main};

fn simple_benchmark(c: &mut Criterion) {
    c.bench_function("simple_benchmark", |b| {
        b.iter(|| {
            // Simulate some work here
        })
    });
}

criterion_group!(benches, simple_benchmark);
criterion_main!(benches);

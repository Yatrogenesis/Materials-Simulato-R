use criterion::{black_box, criterion_group, criterion_main, Criterion};
use compression_experiment::methods::*;

fn bench_gzip(c: &mut Criterion) {
    let vectors: Vec<Vec<f32>> = vec![vec![1.0; 768]; 100];
    c.bench_function("gzip_compress", |b| b.iter(|| {
        black_box(gzip_compress(&vectors))
    }));
}

criterion_group!(benches, bench_gzip);
criterion_main!(benches);

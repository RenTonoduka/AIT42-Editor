//! Benchmark Tests for Buffer Operations
//!
//! Measures performance of core buffer operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ait42_core::Buffer;

fn buffer_creation_benchmark(c: &mut Criterion) {
    c.bench_function("buffer_new", |b| {
        b.iter(|| {
            black_box(Buffer::new());
        });
    });
}

fn buffer_content_benchmark(c: &mut Criterion) {
    let buffer = Buffer::new();
    c.bench_function("buffer_content", |b| {
        b.iter(|| {
            black_box(buffer.content());
        });
    });
}

criterion_group!(benches, buffer_creation_benchmark, buffer_content_benchmark);
criterion_main!(benches);

// Add this to Cargo.toml in benches section:
// [[bench]]
// name = "buffer_bench"
// harness = false

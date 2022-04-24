//! Benchmarks the layout of simple text strings.
use criterion::{criterion_group, criterion_main, Criterion};
use ugly::{font::layout, metrics::Point};

fn criterion_benchmark(c: &mut Criterion) {
    let font = ugly::Font::from_dir("assets/fonts/medium");
    let metrics = font.metrics().expect("couldn't load font metrics");
    c.bench_function("prop-kerned", |b| {
        b.iter(|| {
            layout::String::layout(
                &metrics,
                "The quick brown fox jumps over the lazy dog".to_string(),
                Point::default(),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

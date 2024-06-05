//! Benchmarks the layout of simple text strings.
use criterion::{criterion_group, criterion_main, Criterion};
use ugly::font::layout;

const EXAMPLE_STR: &str = "The quick brown fox\njumps over\nthe lazy dog.";

fn criterion_benchmark(c: &mut Criterion) {
    let font = ugly::Font::from_dir("assets/fonts/medium");
    let metrics = font.metrics().expect("couldn't load font metrics");
    c.bench_function("prop-kerned", |b| {
        b.iter(|| layout::Builder::new(&metrics).build(EXAMPLE_STR.to_owned()))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

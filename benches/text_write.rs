//! Benchmarks the repeated writing of simple text strings.
use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, Criterion};

use ugly::{font, render, resource, text};

/// Benchmarks writing the same string several times in a row without changing anything.
fn write_same(c: &mut Criterion) {
    let font = ugly::Font::from_dir("assets/fonts/medium");
    let metrics = font.metrics().expect("couldn't load font metrics");
    let mmap: resource::DefaultingHashMap<(), font::Metrics> =
        resource::DefaultingHashMap::new(HashMap::default(), metrics);
    c.bench_function("write same thing repeatedly", |b| {
        b.iter_batched(
            setup_write,
            |(logger, writer)| {
                write_repeatedly(
                    logger,
                    writer,
                    &mmap,
                    &["the quick brown fox jumps over the lazy dog"],
                )
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

/// Benchmarks writing the same string several times in a row without changing anything.
fn write_alternating(c: &mut Criterion) {
    let font = ugly::Font::from_dir("assets/fonts/medium");
    let metrics = font.metrics().expect("couldn't load font metrics");
    let mmap: resource::DefaultingHashMap<(), font::Metrics> =
        resource::DefaultingHashMap::new(HashMap::default(), metrics);
    c.bench_function("write alternating strings repeatedly", |b| {
        b.iter_batched(
            setup_write,
            |(logger, writer)| {
                write_repeatedly(
                    logger,
                    writer,
                    &mmap,
                    &[
                        "the quick brown fox jumps over the lazy dog",
                        "JACKDAWS LOVE MY GIANT SPHINX OF QUARTZ",
                    ],
                )
            },
            criterion::BatchSize::SmallInput,
        )
    });
}

// TODO(@MattWindsor91): test moving the position and font of the string

fn setup_write() -> (render::logger::Logger<(), (), ()>, text::Writer<(), ()>) {
    let logger = render::logger::Logger::default();
    let writer = text::Writer::new((), ());
    (logger, writer)
}

#[inline]
fn write_repeatedly(
    mut logger: render::logger::Logger<(), (), ()>,
    mut writer: text::Writer<(), ()>,
    mmap: &resource::DefaultingHashMap<(), font::Metrics>,
    things_to_write: &[&str],
) {
    for i in 0..things_to_write.len() {
        writer.set_string(things_to_write[i % things_to_write.len()]);
        writer.layout(mmap);
        writer
            .render(&mut logger)
            .expect("should not fail to render");
    }
}

criterion_group!(benches, write_same, write_alternating);
criterion_main!(benches);

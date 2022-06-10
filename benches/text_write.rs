//! Benchmarks the repeated writing of simple text strings.
use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use ugly::{colour, font, render, resource, text, Renderer};

/// Benchmarks writing the same string several times in a row without changing anything.
fn write_same(c: &mut Criterion) {
    let font = ugly::Font::from_dir("assets/fonts/medium");
    let metrics = font.metrics().expect("couldn't load font metrics");
    c.bench_function("write same thing repeatedly", |b| {
        b.iter_batched(
            || setup_write(metrics.clone()),
            |(logger, writer)| {
                write_repeatedly(
                    logger,
                    writer,
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
    c.bench_function("write alternating strings repeatedly", |b| {
        b.iter_batched(
            || setup_write(metrics.clone()),
            |(logger, writer)| {
                write_repeatedly(
                    logger,
                    writer,
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

fn setup_write(
    metrics: font::Metrics,
) -> (
    render::logger::Logger<Font, Fg, Fg>,
    text::Writer<Font, Fg, Fg>,
) {
    let mmap: resource::DefaultingHashMap<(), font::Metrics> =
        resource::DefaultingHashMap::new(HashMap::default(), metrics);

    let logger = render::logger::Logger::new(mmap);
    let mut writer = text::Writer::new();
    writer.set_font_spec(font::Spec::default());
    (logger, writer)
}

type Font = resource::DefaultingHashMap<(), font::Font>;
type Fg = resource::DefaultingHashMap<(), colour::Definition>;

#[inline]
fn write_repeatedly(
    mut logger: render::logger::Logger<Font, Fg, Fg>,
    mut writer: text::Writer<Font, Fg, Fg>,
    things_to_write: &[&str],
) {
    for i in 0..things_to_write.len() {
        writer.set_string(things_to_write[i % things_to_write.len()]);
        writer.layout(logger.font_metrics());
        writer
            .render(&mut logger)
            .expect("should not fail to render");
    }
}

criterion_group!(benches, write_same, write_alternating);
criterion_main!(benches);

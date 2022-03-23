//! Checks the pack-in assets to make sure they are present and functional.

use std::path::PathBuf;

/// Tests to make sure the 'large' font metrics file is present and parses correctly.
#[test]
fn test_large_font_metrics() {
    test_font("large")
}

/// Tests to make sure the 'medium' font metrics file is present and parses correctly.
#[test]
fn test_medium_font_metrics() {
    test_font("medium")
}

/// Tests to make sure the 'small' font metrics file is present and parses correctly.
#[test]
fn test_small_font_metrics() {
    test_font("small")
}

fn test_font(name: &'static str) {
    let font = font(name);
    let m = font.metrics().expect("font must have metrics present");
    assert!(m.char.is_normal(), "character size must be normal");
    assert!(!m.char.is_zero(), "character size must be nonzero");
    assert!(m.pad.is_normal(), "padding must be normal");
}

fn font(name: &'static str) -> ugly::Font {
    let path: PathBuf = ["assets", "fonts", name].iter().collect();
    ugly::Font::from_dir(path)
}

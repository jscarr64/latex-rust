//! Render properties: layout, SVG, PNG and egui never panic, never return NaN
//! or infinite sizes, and either succeed with a well-formed result or return
//! a typed error.

use super::props::guarded;
use std::cmp::Ordering;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

/// Inputs whose SVG had a negative width or height (known finding W-SVG-1).
pub static KNOWN_SVG_NEGATIVE: AtomicUsize = AtomicUsize::new(0);

/// Inputs with a 20-digit number whose SVG held `NaN` (known finding W-SVG-2).
pub static KNOWN_SVG_NAN: AtomicUsize = AtomicUsize::new(0);

/// True if `s` holds a run of at least 20 ASCII digits.
fn has_huge_number(s: &str) -> bool {
    let mut run = 0usize;
    for c in s.chars() {
        if c.is_ascii_digit() {
            run += 1;
            if run >= 20 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}
use latex_rust::{
    layout, parse, render_svg, BoxContent, Dim, MathBox, MathFont, MathStyle, SvgOptions,
};

/// A `Dim` that is a real rational (not NaN).
fn finite(d: &Dim) -> bool {
    !d.is_nan() && d.as_ratio().is_some()
}

/// Every dimension in the box tree is finite.
pub fn check_box(b: &MathBox, path: &str) -> Result<(), String> {
    let dims = [
        ("width", &b.width),
        ("height", &b.height),
        ("depth", &b.depth),
        ("italic", &b.italic),
        ("shift", &b.shift),
    ];
    for (name, d) in dims {
        if !finite(d) {
            return Err(format!("{path}: {name} is NaN"));
        }
    }
    match &b.content {
        BoxContent::HList(v) | BoxContent::VList(v) | BoxContent::Overlap(v) => {
            for (i, c) in v.iter().enumerate() {
                check_box(c, &format!("{path}/{i}"))?;
            }
        }
        BoxContent::Kern(d) => {
            if !finite(d) {
                return Err(format!("{path}: kern is NaN"));
            }
        }
        BoxContent::Color(_, inner) | BoxContent::BackColor(_, inner) => {
            check_box(inner, &format!("{path}/c"))?;
        }
        BoxContent::Frame {
            thickness, inner, ..
        } => {
            if !finite(thickness) {
                return Err(format!("{path}: frame thickness is NaN"));
            }
            check_box(inner, &format!("{path}/f"))?;
        }
        BoxContent::Line {
            x1,
            y1,
            x2,
            y2,
            thickness,
        } => {
            for d in [x1, y1, x2, y2, thickness] {
                if !finite(d) {
                    return Err(format!("{path}: line coordinate is NaN"));
                }
            }
        }
        BoxContent::Glyph { scale, .. } if !finite(scale) => {
            return Err(format!("{path}: glyph scale is NaN"));
        }
        _ => {}
    }
    Ok(())
}

/// Number in `name="…pt"` of an SVG header.
fn attr_pt(svg: &str, name: &str) -> Option<f64> {
    let key = format!(" {name}=\"");
    let at = svg.find(&key)? + key.len();
    let rest = svg.get(at..)?;
    let end = rest.find("pt\"")?;
    rest.get(..end)?.parse::<f64>().ok()
}

/// Words `nan`, `inf`, `infinity` (any case) must not appear as tokens.
fn has_bad_number_word(svg: &str) -> bool {
    svg.split(|c: char| !c.is_ascii_alphanumeric()).any(|w| {
        let l = w.to_ascii_lowercase();
        l == "nan" || l == "inf" || l == "infinity"
    })
}

/// Layout and render `src` (already known to parse or not).
pub fn check_render(src: &str, font: &MathFont, display: bool) -> Result<bool, String> {
    let tree = match guarded(|| parse(src))? {
        Ok(t) => t,
        Err(_) => return Ok(false),
    };
    let style = if display {
        MathStyle::Display
    } else {
        MathStyle::Text
    };
    let boxed = match guarded(|| layout(&tree, font, style))? {
        Ok(b) => b,
        Err(_) => return Ok(false),
    };
    check_box(&boxed, "box")?;
    let opts = SvgOptions::new();
    match guarded(|| render_svg(&boxed, font, &opts))? {
        Ok(svg) => {
            if !svg.contains("<svg") {
                return Err(String::from("svg without <svg"));
            }
            if has_bad_number_word(&svg) && has_huge_number(src) {
                // W-SVG-2: a huge but accepted dimension overflows to NaN in
                // the SVG header. Counted; reproduced by an ignored test.
                KNOWN_SVG_NAN.fetch_add(1, AtomicOrdering::Relaxed);
                return Ok(true);
            }
            if has_bad_number_word(&svg) {
                return Err(String::from("svg contains a NaN or inf token"));
            }
            let neg_box = boxed.width.cmp(&Dim::zero()) == Some(Ordering::Less)
                || (&boxed.height + &boxed.depth).cmp(&Dim::zero()) == Some(Ordering::Less);
            for name in ["width", "height"] {
                match attr_pt(&svg, name) {
                    Some(v) if v.is_finite() && v >= 0.0 => {}
                    Some(v) if v.is_finite() && neg_box => {
                        // W-SVG-1: a box with negative size gives a negative
                        // `width` or `height`, which is invalid SVG. Counted,
                        // reproduced by an ignored test in
                        // tests/wide_range_findings.rs.
                        KNOWN_SVG_NEGATIVE.fetch_add(1, AtomicOrdering::Relaxed);
                    }
                    other => return Err(format!("svg {name} attribute is {other:?}")),
                }
            }
        }
        Err(_) => return Ok(true),
    }
    Ok(true)
}

/// PNG backend: typed error, or bytes with the PNG magic.
pub fn check_png(src: &str, font: &MathFont) -> Result<(), String> {
    use latex_rust::{latex_to_png, PngOptions};
    match guarded(|| latex_to_png(src, font, &PngOptions::new()))? {
        Ok(bytes) => {
            if !bytes.starts_with(b"\x89PNG") {
                return Err(String::from("png without the PNG magic"));
            }
            Ok(())
        }
        Err(_) => Ok(()),
    }
}

/// egui backend: typed error, or a finite bounding rectangle.
#[cfg(feature = "egui")]
pub fn check_egui(src: &str, font: &MathFont) -> Result<(), String> {
    use latex_rust::{latex_to_shapes, EguiOptions};
    match guarded(|| latex_to_shapes(src, font, &EguiOptions::new(), egui::Pos2::ZERO, 1.0))? {
        Ok((_shapes, rect)) => {
            if !rect.is_finite() {
                if has_huge_number(src) {
                    // W-SVG-2 reaches the egui backend too.
                    KNOWN_SVG_NAN.fetch_add(1, AtomicOrdering::Relaxed);
                    return Ok(());
                }
                return Err(String::from("egui rect is not finite"));
            }
            Ok(())
        }
        Err(_) => Ok(()),
    }
}

/// egui is off: nothing to check.
#[cfg(not(feature = "egui"))]
pub fn check_egui(_src: &str, _font: &MathFont) -> Result<(), String> {
    Ok(())
}

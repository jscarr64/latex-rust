//! Hand-derived semantic oracles: widths and rule counts that follow from
//! TeX's rules and the crate's documented units, independent of the code
//! under test. Units: `1 em = 1`, `1pt = 1/10 em`, `1mu = 1/18 em`, `bp` reads
//! as `pt`, no unit reads as `em` (`Dim::in_em`).

use latex_rust::{latex_to_svg, layout, parse, MathFont, MathStyle, SvgOptions};

/// Width of `src` in em as an exact ratio.
pub fn width_of(src: &str, font: &MathFont) -> Result<(i128, i128), String> {
    let tree = parse(src).map_err(|e| format!("{src:?}: {e}"))?;
    let b = layout(&tree, font, MathStyle::Text).map_err(|e| format!("{src:?}: {e}"))?;
    b.width
        .as_ratio()
        .ok_or_else(|| format!("{src:?}: NaN width"))
}

fn add(a: (i128, i128), b: (i128, i128)) -> Option<(i128, i128)> {
    let num = a.0.checked_mul(b.1)?.checked_add(b.0.checked_mul(a.1)?)?;
    let den = a.1.checked_mul(b.1)?;
    let (mut x, mut y) = (num.unsigned_abs(), den.unsigned_abs());
    while y != 0 {
        let t = x % y;
        x = y;
        y = t;
    }
    let g = i128::try_from(x).ok()?.max(1);
    Some((num / g, den / g))
}

/// `\hspace` and the named spaces have exact widths.
pub fn spacing_widths(font: &MathFont) -> Result<usize, String> {
    // (input, numerator, denominator) in em. Derived by hand:
    // 1pt = 1/10 em, 1mu = 1/18 em; thin = 3mu, medium = 4mu, thick = 5mu.
    let table: &[(&str, i128, i128)] = &[
        ("\\hspace{1em}", 1, 1),
        ("\\hspace{0em}", 0, 1),
        ("\\hspace{-1em}", -1, 1),
        ("\\hspace{.5em}", 1, 2),
        ("\\hspace{1.5em}", 3, 2),
        ("\\hspace{+1.5em}", 3, 2),
        ("\\hspace{10pt}", 1, 1),
        ("\\hspace{1pt}", 1, 10),
        ("\\hspace{-3pt}", -3, 10),
        ("\\hspace{18mu}", 1, 1),
        ("\\hspace{1mu}", 1, 18),
        ("\\hspace{3.25mu}", 13, 72),
        ("\\hspace{-18mu}", -1, 1),
        ("\\hspace{2bp}", 1, 5),
        ("\\hspace{1}", 1, 1),
        ("\\hspace{0.0001em}", 1, 10_000),
        (
            "\\hspace{99999999999999999999em}",
            99_999_999_999_999_999_999,
            1,
        ),
        (
            "\\hspace{0.00000000000000000000001em}",
            1,
            100_000_000_000_000_000_000_000,
        ),
        ("\\quad", 1, 1),
        ("\\qquad", 2, 1),
        ("\\,", 1, 6),
        ("\\:", 2, 9),
        ("\\;", 5, 18),
        ("\\!", -1, 6),
        ("\\ ", 1, 3),
    ];
    for (src, n, d) in table {
        let got = width_of(src, font)?;
        if got != (*n, *d) {
            return Err(format!("{src:?}: width {got:?} em, expected {n}/{d}"));
        }
    }
    Ok(table.len())
}

/// Between two ordinary atoms there is no glue, so widths add exactly.
pub fn widths_add(font: &MathFont) -> Result<usize, String> {
    let mut n = 0;
    // Digits and fences carry no italic correction, so nothing but the kern
    // sits between them (an italic letter adds its correction, Appendix G).
    for pair in [
        ("1", "2"),
        ("2", "1"),
        ("1", "1"),
        ("(", ")"),
        ("[", "]"),
        ("9", "0"),
    ] {
        for gap in ["1em", "-1em", "0.5em", "3pt", "7mu", "-0.25em", "123.456em"] {
            let (a, b) = (width_of(pair.0, font)?, width_of(pair.1, font)?);
            let h = width_of(&format!("\\hspace{{{gap}}}"), font)?;
            let want = add(add(a, h).ok_or("overflow")?, b).ok_or("overflow")?;
            let got = width_of(&format!("{}\\hspace{{{gap}}}{}", pair.0, pair.1), font)?;
            if got != want {
                return Err(format!(
                    "{}\\hspace{{{gap}}}{}: width {got:?}, expected {want:?}",
                    pair.0, pair.1
                ));
            }
            n += 1;
        }
    }
    Ok(n)
}

/// Constructs that draw exactly one rule in TeX.
pub fn rule_counts(font: &MathFont) -> Result<usize, String> {
    let table: &[(&str, usize)] = &[
        ("x", 0),
        ("x^2_3", 0),
        ("\\frac{a}{b}", 1),
        ("\\dfrac{a}{b}", 1),
        ("\\tfrac{a}{b}", 1),
        ("{a\\over b}", 1),
        ("\\sqrt{x}", 1),
        ("\\sqrt[3]{x}", 1),
        ("\\overline{x}", 1),
        ("\\underline{x}", 1),
        ("\\frac{\\frac{a}{b}}{c}", 2),
        ("\\frac{a}{b}+\\frac{c}{d}", 2),
        ("\\sqrt{\\sqrt{x}}", 2),
        ("\\sum_{i=1}^{n}i", 0),
    ];
    for (src, want) in table {
        let svg =
            latex_to_svg(src, font, &SvgOptions::new()).map_err(|e| format!("{src:?}: {e}"))?;
        let got = svg.matches("<rect").count();
        if got != *want {
            return Err(format!("{src:?}: {got} rule(s), expected {want}"));
        }
    }
    Ok(table.len())
}

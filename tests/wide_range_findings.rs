//! Red tests for the findings of the wide-range layer. Every test here
//! currently FAILS, so each is `#[ignore]`d with the finding id; run them with
//! `cargo test --test wide_range_findings -- --ignored` to see the red state.
//! Remove the `#[ignore]` in the commit that fixes the root cause.
//!
//! Ids are the rows of `documents/FUZZ_LEDGER.md` (section "Wide-range
//! findings"). Goldens are hand-derived (TeX rules), never taken from the code
//! under test.

use latex_rust::{
    latex_to_svg, layout, parse, parse_with_options, MathFont, MathStyle, ParseOptions, SvgOptions,
};

fn font() -> Result<MathFont, String> {
    MathFont::stix_two_math().map_err(|e| format!("font: {e}"))
}

fn rects(src: &str, f: &MathFont) -> Result<usize, String> {
    let svg = latex_to_svg(src, f, &SvgOptions::new()).map_err(|e| format!("{src:?}: {e}"))?;
    Ok(svg.matches("<rect").count())
}

/// Largest `k <= 40` such that `x^{`*k + min + `}`*k parses by default must
/// print to something that parses by default.
fn boundary(min: &str, options: &ParseOptions) -> Result<(), String> {
    for k in (0..40usize).rev() {
        let src = format!("{}{}{}", "x^{".repeat(k), min, "}".repeat(k));
        let Ok((tree, _)) = parse_with_options(&src, options) else {
            continue;
        };
        let printed = tree.to_latex_with(options);
        return match parse_with_options(&printed, options) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!(
                "{src:?} is accepted (k={k}) but its print is rejected: {e}\n  print {printed:?}"
            )),
        };
    }
    Err(format!("{min:?} never parses"))
}

/// W-SVG-1: a box with negative width gives `width="-12.0pt"`, invalid SVG.
#[test]
#[ignore = "W-SVG-1 negative svg width"]
fn svg_width_is_never_negative() -> Result<(), String> {
    let f = font()?;
    for src in ["\\hspace{-1em}", "x\\hspace{-3em}", "\\!\\!\\!"] {
        let svg = latex_to_svg(src, &f, &SvgOptions::new()).map_err(|e| e.to_string())?;
        if svg.contains("width=\"-") || svg.contains("height=\"-") {
            return Err(format!(
                "{src:?} gives a negative size: {}",
                svg.lines().nth(1).unwrap_or("")
            ));
        }
    }
    Ok(())
}

/// W-BINOM-1: TeX draws no rule in `\binom` / `\choose` (`\genfrac` with a
/// 0pt bar). The renderer draws one.
#[test]
#[ignore = "W-BINOM-1 binomial drawn with a fraction bar"]
fn binomials_have_no_rule() -> Result<(), String> {
    let f = font()?;
    for src in [
        "\\binom{n}{k}",
        "{n\\choose k}",
        "\\dbinom{n}{k}",
        "\\tbinom{n}{k}",
    ] {
        let n = rects(src, &f)?;
        if n != 0 {
            return Err(format!("{src:?} draws {n} rule(s), TeX draws none"));
        }
    }
    Ok(())
}

/// W-GENFRAC-1: the thickness and style arguments of `\genfrac` are read and
/// dropped, so `0pt` still draws a bar. A zero thickness is a barless stack.
#[test]
#[ignore = "W-GENFRAC-1 genfrac thickness ignored"]
fn genfrac_zero_thickness_has_no_rule() -> Result<(), String> {
    let f = font()?;
    for src in ["\\genfrac{}{}{0pt}{}{a}{b}", "\\genfrac{(}{)}{0pt}{}{a}{b}"] {
        // Typed rejection is acceptable too; a bar is not.
        if let Ok(n) = rects(src, &f) {
            if n != 0 {
                return Err(format!("{src:?} draws {n} rule(s) for thickness 0pt"));
            }
        }
    }
    Ok(())
}

/// W-RULE-1: `\rule{w}{h}` keeps only the height and renders an invisible
/// strut of width 0. TeX draws a w by h black box.
#[test]
#[ignore = "W-RULE-1 rule width dropped"]
fn rule_keeps_its_width() -> Result<(), String> {
    let f = font()?;
    for (src, num, den) in [
        ("\\rule{1em}{1em}", 1i128, 1i128),
        ("\\rule{2em}{.5em}", 2, 1),
        ("\\rule{10pt}{1pt}", 1, 1),
    ] {
        let tree = parse(src).map_err(|e| e.to_string())?;
        let Ok(b) = layout(&tree, &f, MathStyle::Text) else {
            continue; // a typed error is acceptable
        };
        if b.width.as_ratio() != Some((num, den)) {
            return Err(format!(
                "{src:?}: width {:?} em, expected {num}/{den}",
                b.width.as_ratio()
            ));
        }
    }
    if rects("\\rule{1em}{1em}", &f)? != 1 {
        return Err(String::from("\\rule{1em}{1em} draws no rectangle"));
    }
    Ok(())
}

/// W-RT-1: `\genfrac` accepts empty numerator and denominator; its print is
/// `\frac{}{}` or `\binom{}{}`, which the parser rejects.
#[test]
#[ignore = "W-RT-1 genfrac empty args do not round-trip"]
fn genfrac_with_empty_arguments_round_trips() -> Result<(), String> {
    for src in [
        "\\genfrac{(}{)}{}{}{}{}",
        "\\genfrac{}{}{}{}{}{}",
        "\\genfrac{}{rangle}{}{}{}{}",
    ] {
        let Ok(tree) = parse(src) else {
            continue; // rejecting it up front is the consistent answer
        };
        let printed = tree.to_latex();
        if let Err(e) = parse(&printed) {
            return Err(format!(
                "{src:?} parses, its print {printed:?} does not: {e}"
            ));
        }
    }
    Ok(())
}

/// W-GENFRAC-2: the `{[}` / `{]}` preprocessor shorthand breaks the standard
/// `\genfrac{[}{]}{…}` delimiters (typed error `expected '{'`).
#[test]
#[ignore = "W-GENFRAC-2 genfrac square-bracket delimiters rejected"]
fn genfrac_accepts_square_bracket_delimiters() -> Result<(), String> {
    for src in [
        "\\genfrac{[}{]}{}{}{a}{b}",
        "\\genfrac{(}{]}{}{}{a}{b}",
        "\\genfrac{[}{)}{}{}{a}{b}",
    ] {
        parse(src).map_err(|e| format!("{src:?}: {e}"))?;
    }
    Ok(())
}

/// W-DEPTH-1: an empty row, cell or body prints as `{}`, one level deeper than
/// the source, so a source at the depth limit prints to something rejected.
#[test]
#[ignore = "W-DEPTH-1 empty node printed as {}"]
fn depth_boundary_empty_nodes() -> Result<(), String> {
    for min in [
        "\\left(\\right)",
        "\\begin{matrix}&\\end{matrix}",
        "\\tag{}",
    ] {
        boundary(min, &ParseOptions::new())?;
    }
    Ok(())
}

#[test]
#[ignore = "W-DEPTH-2 \\not X printed as \\not {X}"]
fn depth_boundary_not() -> Result<(), String> {
    boundary("\\not\\alpha", &ParseOptions::new())
}

#[test]
#[ignore = "W-DEPTH-3 unicode script char printed as ^{..}"]
fn depth_boundary_unicode_script() -> Result<(), String> {
    boundary("x²", &ParseOptions::new().with_unicode_symbols(true))
}

#[test]
#[ignore = "W-DEPTH-4 prime_symbol: x' printed as x^\\prime"]
fn depth_boundary_prime_symbol() -> Result<(), String> {
    boundary("x'", &ParseOptions::new().with_prime_symbol(true))
}

/// W-DEPTH-5: a run of `\color` switches is flat in the source and nested in
/// the print: 16 of them print deeper than 32.
#[test]
#[ignore = "W-DEPTH-5 \\color switches printed nested"]
fn color_run_round_trips() -> Result<(), String> {
    let src = format!("{}x", "\\color{red}".repeat(16));
    let tree = parse(&src).map_err(|e| e.to_string())?;
    let printed = tree.to_latex();
    parse(&printed)
        .map(|_| ())
        .map_err(|e| format!("16 colour switches: print rejected: {e}"))
}

#[test]
#[ignore = "W-DEPTH-6 font body of several nodes printed in a group"]
fn depth_boundary_font_body() -> Result<(), String> {
    boundary("=\\mathrm{-e}", &ParseOptions::new())
}

#[test]
#[ignore = "W-DEPTH-11 named operator with both scripts printed as {\\op _b}^a"]
fn depth_boundary_limits() -> Result<(), String> {
    boundary("\\lim^a_2", &ParseOptions::new())
}

#[test]
#[ignore = "W-DEPTH-12 prime followed by a second script printed as {x^x'}^y"]
fn depth_boundary_prime_then_script() -> Result<(), String> {
    boundary("x^x'^x", &ParseOptions::new())
}

/// W-SVG-2: a 39-digit dimension is accepted, and the SVG header then holds
/// `NaN` (`height="NaNpt"`). An invalid size is never valid SVG; a typed
/// error (as for 1e38em, which is rejected) is the honest answer.
#[test]
#[ignore = "W-SVG-2 NaN in svg for a huge accepted dimension"]
fn svg_has_no_nan_for_huge_dimensions() -> Result<(), String> {
    let f = font()?;
    let nines = "9".repeat(39);
    for src in [
        format!("\\hspace{{{nines}em}}"),
        format!("\\rule{{1em}}{{1{}em}}", "0".repeat(38)),
    ] {
        if let Ok(svg) = latex_to_svg(&src, &f, &SvgOptions::new()) {
            if svg.contains("NaN") || svg.contains("inf") {
                return Err(format!(
                    "{src:?} gives {}",
                    svg.lines().nth(1).unwrap_or("")
                ));
            }
        }
    }
    Ok(())
}

/// W-OPNAME-1: the body of `\operatorname` is read as a literal name, so a
/// command in it (`\alpha`, `\sin`, `\frac{1}{2}`, `\left(`) is shown as the
/// text `\alpha`. TeX reads the body as math (`\alpha` is alpha). A typed
/// error for a command in the body would also be honest; silent literal text
/// is not.
#[test]
#[ignore = "W-OPNAME-1 operatorname body is literal text, commands are not read"]
fn operatorname_does_not_show_commands_as_text() -> Result<(), String> {
    for src in [
        "\\operatorname{\\alpha}",
        "\\operatorname{\\sin}x",
        "\\operatorname{\\frac{1}{2}}",
        "\\operatorname{\\left(x}",
    ] {
        if let Ok(tree) = parse(src) {
            let gold = tree.gold();
            if gold.contains("(op \\") {
                return Err(format!("{src:?} is accepted as the literal text: {gold}"));
            }
        }
    }
    Ok(())
}

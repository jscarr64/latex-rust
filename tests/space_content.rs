//! Explicit spacing is content, including inside environment cells.
use latex_rust::{latex_to_svg, parse, MathFont, SvgOptions};

fn assert_renders(input: &str) {
    let font = MathFont::stix_two_math().expect("STIX Two Math");
    let svg = latex_to_svg(input, &font, &SvgOptions::new())
        .unwrap_or_else(|error| panic!("{input}: {error}"));
    assert!(svg.contains("<svg"), "{input}: missing SVG root");
}

fn assert_cell_spacing(environment: &str, spacing: &str) {
    let input = format!(r"\begin{{{environment}}}a{spacing} b&c\end{{{environment}}}");
    let node = parse(&input).expect("spaced cells parse");
    let space_gold = parse(spacing).expect("spacing parses").gold();
    assert!(
        node.gold().contains(&space_gold),
        "{input}: cell lost {space_gold}: {}",
        node.gold()
    );
    assert_renders(&input);
}

#[test]
fn align_thin_space_is_preserved() {
    assert_cell_spacing("align", r"\,");
}

#[test]
fn align_quad_is_preserved() {
    assert_cell_spacing("align", r"\quad");
}

#[test]
fn matrix_thin_space_is_preserved() {
    assert_cell_spacing("matrix", r"\,");
}

#[test]
fn matrix_quad_is_preserved() {
    assert_cell_spacing("matrix", r"\quad");
}

#[test]
fn cases_thin_space_is_preserved() {
    assert_cell_spacing("cases", r"\,");
}

#[test]
fn cases_quad_is_preserved() {
    assert_cell_spacing("cases", r"\quad");
}

#[test]
fn underline_space_is_content() {
    assert_renders(r"\underline{\quad}");
}

#[test]
fn fraction_space_numerator_is_content() {
    assert_renders(r"\frac{\,}{2}");
}

#[test]
fn empty_hat_still_errors() {
    assert!(parse(r"\hat{}").is_err());
    let font = MathFont::stix_two_math().expect("STIX Two Math");
    assert!(latex_to_svg(r"\hat{}", &font, &SvgOptions::new()).is_err());
}

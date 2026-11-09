use latex_rust::{layout, parse, BoxContent, Dim, MathBox, MathFont, MathParams, MathStyle};

#[test]
fn gathered_is_unnumbered_displaystyle_centered_on_the_math_axis() {
    let font = MathFont::stix_two_math().unwrap();
    let ast = parse(r"\begin{gathered}\frac12\\x+1=2\end{gathered}").unwrap();
    assert!(ast.gold().contains("gathered"));
    let tree = layout(&ast, &font, MathStyle::Text).unwrap();
    let BoxContent::VList(rows) = &tree.content else {
        panic!("stack")
    };
    assert_eq!(rows.len(), 3);
    let axis = MathParams::from_font(&font).unwrap().axis_height;
    assert_eq!(
        (&tree.height - &tree.depth) / Dim::from_i64(2) + &tree.shift,
        axis
    );
    assert_eq!(rows[0].width, rows[2].width);
    let fraction = layout(&parse(r"\frac12").unwrap(), &font, MathStyle::Display).unwrap();
    let BoxContent::HList(parts) = &rows[0].content else {
        panic!("row")
    };
    let BoxContent::HList(cell) = &parts[1].content else {
        panic!("centered cell")
    };
    assert!(cell
        .iter()
        .any(|b| b.height == fraction.height && b.depth == fraction.depth));
}

#[test]
fn gathered_still_rejects_alignment_tabs() {
    let font = MathFont::stix_two_math().unwrap();
    assert!(layout(
        &parse(r"\begin{gathered}x&y\end{gathered}").unwrap(),
        &font,
        MathStyle::Display
    )
    .is_err());
}

#[test]
fn gathered_accepts_hline_and_grows_by_exactly_the_rule_thickness() {
    let font = MathFont::stix_two_math().unwrap();
    let plain = layout(
        &parse(r"\begin{gathered}x\end{gathered}").unwrap(),
        &font,
        MathStyle::Display,
    )
    .unwrap();
    let ruled = layout(
        &parse(r"\begin{gathered}\hline x\end{gathered}").unwrap(),
        &font,
        MathStyle::Display,
    )
    .unwrap();
    // TeX accepts `\hline` in gathered and draws the rule: the box grows
    // by exactly the rule thickness (0.4 pt = 0.04 em), half in height and
    // half in depth, and the width does not change.
    assert_eq!(ruled.width, plain.width);
    assert_eq!(
        &ruled.height + &ruled.depth,
        &(&plain.height + &plain.depth) + &Dim::ratio(1, 25)
    );
    let eff_height = |b: &MathBox| &b.height + &b.shift;
    let eff_depth = |b: &MathBox| &b.depth - &b.shift;
    assert_eq!(eff_height(&ruled), &eff_height(&plain) + &Dim::ratio(1, 50));
    assert_eq!(eff_depth(&ruled), &eff_depth(&plain) + &Dim::ratio(1, 50));
    // Still centred on the math axis.
    let axis = MathParams::from_font(&font).unwrap().axis_height;
    assert_eq!(
        (&ruled.height - &ruled.depth) / Dim::from_i64(2) + &ruled.shift,
        axis
    );
}

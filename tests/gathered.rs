use latex_rust::{layout, parse, BoxContent, Dim, MathFont, MathParams, MathStyle};

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
fn gathered_rejects_alignment_tabs_and_rules() {
    let font = MathFont::stix_two_math().unwrap();
    for source in [
        r"\begin{gathered}x&y\end{gathered}",
        r"\begin{gathered}\hline x\end{gathered}",
    ] {
        assert!(layout(&parse(source).unwrap(), &font, MathStyle::Display).is_err());
    }
}

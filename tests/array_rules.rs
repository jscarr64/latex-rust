// SPDX-License-Identifier: MIT OR Apache-2.0
use latex_rust::{layout, parse, BoxContent, Dim, MathBox, MathFont, MathStyle};

fn array(spec: &str, body: &str) -> MathBox {
    let font = MathFont::stix_two_math().unwrap();
    layout(
        &parse(&format!(r"\begin{{array}}{{{spec}}}{body}\end{{array}}")).unwrap(),
        &font,
        MathStyle::Display,
    )
    .unwrap()
}

fn children(b: &MathBox) -> &[MathBox] {
    match &b.content {
        BoxContent::HList(c) | BoxContent::VList(c) | BoxContent::Overlap(c) => c,
        _ => panic!("expected list"),
    }
}

fn total(b: &MathBox) -> Dim {
    &b.height + &b.depth
}

#[test]
fn vertical_rules_add_no_width_and_span_every_row() {
    let plain = array("cc", r"x&1\\y&2");
    let ruled = array("c|c", r"x&1\\\hline y&2");
    let layers = children(&ruled);
    assert_eq!(layers.len(), 2);
    let border = &children(&layers[1])[1];
    assert!(matches!(border.content, BoxContent::Rule));
    assert_eq!(border.height, ruled.height);
    assert_eq!(border.depth, ruled.depth);
    // TeX draws a vertical rule centred in the intercolumn gap, so ruled
    // minus plain is exactly zero: the rule adds no width at all, and the
    // full column gap is preserved around it. It must never squeeze the
    // glyphs together, nor widen the table.
    assert_eq!(ruled.width, plain.width);
    assert_eq!(border.width, Dim::ratio(1, 25));
    let rows = children(&layers[0]);
    assert_eq!(rows.len(), 4, "row, rule, padding, row");
    assert!(matches!(rows[1].content, BoxContent::Rule));
    assert_eq!(rows[1].width, ruled.width);
    assert!(matches!(rows[2].content, BoxContent::Empty));
    assert_eq!(rows[2].depth, Dim::ratio(1, 5));
}

#[test]
fn outer_rules_add_no_width_and_double_rules_add_doublerulesep() {
    let tree = array("|l||r|", "x&123");
    let layers = children(&tree);
    assert_eq!(layers.len(), 5, "table and four continuous borders");
    let row = &children(&layers[0])[0];
    let parts = children(row);
    assert_eq!(parts.len(), 11);
    // Edge rules add no gap; the single rule between the columns splits
    // the column gap in half on each side; the `||` group adds exactly
    // \doublerulesep (0.2 em) between its rules.
    assert_eq!(parts[1].width, Dim::zero());
    assert_eq!(parts[3].width, Dim::ratio(5, 18));
    assert_eq!(parts[5].width, Dim::ratio(1, 5));
    assert_eq!(parts[7].width, Dim::ratio(5, 18));
    assert_eq!(parts[9].width, Dim::zero());
    assert_eq!(row.width, tree.width);
    // Ruled minus plain is exactly the one `||` group: the single `|`
    // rules on the edges contribute nothing.
    let plain = array("lr", "x&123");
    assert_eq!(tree.width, &plain.width + &Dim::ratio(1, 5));
}

#[test]
fn horizontal_rules_add_only_their_thickness_and_the_double_rule_gap() {
    let tree = array("cc", r"\hline x&y\\\hline\hline");
    let rows = children(&tree);
    assert_eq!(rows.len(), 5, "rule, row, rule, gap, rule");
    for i in [0, 2, 4] {
        assert!(matches!(rows[i].content, BoxContent::Rule));
        assert_eq!(rows[i].width, tree.width);
    }
    // No padding around the isolated leading rule; between the two
    // adjacent trailing rules, exactly \doublerulesep minus one rule
    // thickness (0.2 - 0.04 = 0.16 em).
    assert!(matches!(rows[3].content, BoxContent::Empty));
    assert_eq!(rows[3].depth, Dim::ratio(4, 25));
    // Totals: the leading rule adds its thickness (0.04) and the trailing
    // double rule adds two thicknesses plus the double gap (0.24).
    let plain = array("cc", r"x&y");
    assert_eq!(total(&tree), &total(&plain) + &Dim::ratio(7, 25));
    let single = array("c", r"a\\\hline b");
    let plain_single = array("c", r"a\\b");
    assert_eq!(total(&single), &total(&plain_single) + &Dim::ratio(1, 25));
    let doubled = array("cc", r"a & b \\\hline\hline c & d");
    let plain_double = array("cc", r"a & b \\ c & d");
    assert_eq!(total(&doubled), &total(&plain_double) + &Dim::ratio(6, 25));
}

#[test]
fn ordinary_arrays_preserve_row_spacing_and_mixed_alignment() {
    let tree = array("lcr", r"x&y&z\\123&456&789");
    let rows = children(&tree);
    assert_eq!(rows.len(), 3);
    assert!(matches!(rows[1].content, BoxContent::Empty));
    let top = children(&rows[0]);
    let bottom = children(&rows[2]);
    assert_eq!(top.len(), 5);
    for i in [0, 2, 4] {
        assert_eq!(top[i].width, bottom[i].width);
    }
    let left = children(&top[0]);
    let center = children(&top[2]);
    let right = children(&top[4]);
    assert!(matches!(
        left[0].content,
        BoxContent::Glyph { .. } | BoxContent::HList(_)
    ));
    assert!(matches!(center[0].content, BoxContent::Kern(_)));
    assert!(matches!(right[0].content, BoxContent::Kern(_)));
}

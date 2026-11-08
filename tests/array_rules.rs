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

#[test]
fn vertical_rules_keep_the_full_column_gap_and_span_every_row() {
    let plain = array("cc", r"x&1\\y&2");
    let ruled = array("c|c", r"x&1\\\hline y&2");
    let layers = children(&ruled);
    assert_eq!(layers.len(), 2);
    let border = &children(&layers[1])[1];
    assert!(matches!(border.content, BoxContent::Rule));
    assert_eq!(border.height, ruled.height);
    assert_eq!(border.depth, ruled.depth);
    // A vertical rule adds only its thickness; both half-gaps sum to the
    // ordinary column gap. It must never squeeze the glyphs together.
    assert_eq!(ruled.width, &plain.width + &border.width);
    let rows = children(&layers[0]);
    assert_eq!(rows.len(), 5, "row, padding, rule, padding, row");
    for gap in [&rows[1], &rows[3]] {
        assert!(matches!(gap.content, BoxContent::Empty));
        assert!(gap.depth > Dim::zero());
    }
    assert_eq!(rows[2].width, ruled.width);
}

#[test]
fn outer_and_double_rules_have_visible_gaps() {
    let tree = array("|l||r|", "x&123");
    let layers = children(&tree);
    assert_eq!(layers.len(), 5, "table and four continuous borders");
    let row = &children(&layers[0])[0];
    let parts = children(row);
    // Every pair of tracks has positive glue, including adjacent rules.
    for gap in parts.iter().skip(1).step_by(2) {
        assert!(gap.width > Dim::zero());
    }
    assert_eq!(row.width, tree.width);
}

#[test]
fn horizontal_rules_have_padding_at_top_bottom_and_between_rules() {
    let tree = array("cc", r"\hline x&y\\\hline\hline");
    let rows = children(&tree);
    assert_eq!(rows.len(), 7);
    for i in [0, 4, 6] {
        assert!(matches!(rows[i].content, BoxContent::Rule));
        assert_eq!(rows[i].width, tree.width);
    }
    for i in [1, 3, 5] {
        assert!(rows[i].depth > Dim::zero());
    }
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

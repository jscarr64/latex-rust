use latex_rust::{layout, parse, render_svg_with_metrics, Dim, MathFont, MathStyle, SvgOptions};

#[test]
fn viewport_contains_negative_glyph_bearings_and_reports_real_baseline() {
    let font = MathFont::stix_two_math().unwrap();
    for source in [
        "x",
        r"\begin{aligned}y&=\tfrac12gt^2,\\y&=176.4\,\text{m}.\end{aligned}",
        r"\begin{array}{l||r}x&1\\\hline\hline y&2\end{array}",
    ] {
        let tree = layout(&parse(source).unwrap(), &font, MathStyle::Display).unwrap();
        let opt = SvgOptions::new();
        let image = render_svg_with_metrics(&tree, &font, &opt).unwrap();
        assert!(image.width_pt >= &tree.width * &opt.font_size_pt);
        assert!(image.height_pt >= (&tree.height + &tree.depth) * &opt.font_size_pt);
        assert!(image.baseline_pt > Dim::zero());
        assert!(image.baseline_pt <= image.height_pt);
        // The mathematical baseline includes the root stack's shift.
        let view = image
            .svg
            .split("viewBox=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let origin_y = view
            .split_whitespace()
            .nth(1)
            .unwrap()
            .parse::<f64>()
            .unwrap();
        let baseline = image.baseline_pt.to_svg_string().parse::<f64>().unwrap();
        let expected = (&tree.height * &opt.font_size_pt)
            .to_svg_string()
            .parse::<f64>()
            .unwrap();
        assert!((baseline + origin_y - expected).abs() < 0.00001);
    }
}

//! Fixed adversarial inputs and size-parameterised families.

/// Named fixed inputs: empty, one byte, whitespace, unicode, unbalanced,
/// unknown commands and the classic TeX traps.
pub fn fixed_inputs() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = Vec::new();
    let mut add = |name: &str, s: &str| v.push((String::from(name), String::from(s)));
    add("empty", "");
    add("space", " ");
    add("newline", "\n");
    add("crlf", "\r\n");
    add("nul", "\0");
    add("bom", "\u{feff}");
    add("bom-x", "\u{feff}x");
    add("x-bom", "x\u{feff}");
    add("rtl-mark", "\u{202e}x\u{202c}");
    add("combining-alone", "\u{301}");
    add("x-combining", "x\u{301}\u{302}\u{303}\u{304}\u{305}\u{306}");
    add("astral", "\u{1f600}");
    add("astral-math", "\u{1d538}\u{1d539}");
    add("max-char", "\u{10ffff}");
    add("hebrew", "\u{5d0}\u{5d1}\u{5d2}");
    add("arabic", "\u{627}\u{644}");
    add("backslash", "\\");
    add("two-backslash", "\\\\");
    add("three-backslash", "\\\\\\");
    add("backslash-space", "\\ ");
    add("backslash-nul", "\\\0");
    add("backslash-astral", "\\\u{1f600}");
    add("backslash-combining", "\\\u{301}");
    add("brace-open", "{");
    add("brace-close", "}");
    add("brace-pair", "{}");
    add("brace-nest-empty", "{{{{}}}}");
    add("caret", "^");
    add("underscore", "_");
    add("caret-underscore", "^_");
    add("script-only", "_1^2");
    add("double-sup", "x^1^2");
    add("double-sub", "x_1_2");
    add("prime-only", "'");
    add("amp", "&");
    add("percent", "%");
    add("percent-nl", "%\n");
    add("dollar", "$");
    add("dollar2", "$$");
    add("dollar-pair", "$x$");
    add("paren-math", "\\(x\\)");
    add("bracket-math", "\\[x\\]");
    add("left-only", "\\left");
    add("left-paren", "\\left(");
    add("right-only", "\\right)");
    add("left-right-dot", "\\left.\\right.");
    add("left-bad", "\\left x\\right y");
    add("frac-none", "\\frac");
    add("frac-one", "\\frac{1}");
    add("frac-bare", "\\frac12");
    add("frac-bare-bs", "\\frac\\alpha\\beta");
    add("sqrt-none", "\\sqrt");
    add("sqrt-opt-open", "\\sqrt[");
    add("sqrt-opt-nested", "\\sqrt[[x]]{y}");
    add("begin-only", "\\begin");
    add("begin-open", "\\begin{");
    add("begin-name", "\\begin{matrix}");
    add("end-only", "\\end{matrix}");
    add("env-mismatch", "\\begin{matrix}x\\end{pmatrix}");
    add(
        "env-nest-mismatch",
        "\\begin{matrix}\\begin{pmatrix}x\\end{matrix}\\end{pmatrix}",
    );
    add("env-empty", "\\begin{matrix}\\end{matrix}");
    add("env-empty-rows", "\\begin{matrix}\\\\\\\\\\\\\\end{matrix}");
    add("env-amp-only", "\\begin{matrix}&&&\\end{matrix}");
    add(
        "env-ragged",
        "\\begin{matrix}1&2&3\\\\4\\\\5&6\\end{matrix}",
    );
    add("array-nospec", "\\begin{array}x\\end{array}");
    add("array-empty-spec", "\\begin{array}{}x\\end{array}");
    add("array-wide-spec", "\\begin{array}{cccccccccc}1\\end{array}");
    add("array-bad-spec", "\\begin{array}{z}1\\end{array}");
    add("cases-empty", "\\begin{cases}\\end{cases}");
    add("row-outside", "a\\\\b");
    add("cr-outside", "a\\cr b");
    add("hline-outside", "\\hline");
    add("unknown", "\\notacommandxyz");
    add("unknown-long", &format!("\\{}", "a".repeat(5000)));
    add("unknown-nul", "\\foo\0bar");
    add("text-open", "\\text{");
    add("text-dollar-open", "\\text{$}");
    add("text-nested", "\\text{\\text{\\text{x}}}");
    add("text-math-text", "\\text{$\\text{$x$}$}");
    add("text-backslash", "\\text{\\}");
    add("text-escape-end", "\\text{a\\");
    add("mbox-nul", "\\mbox{\0}");
    add("hspace-nodim", "\\hspace{}");
    add(
        "hspace-huge",
        "\\hspace{99999999999999999999999999999999999999999999em}",
    );
    add(
        "hspace-tiny",
        "\\hspace{0.000000000000000000000000000000000000000001em}",
    );
    add("hspace-neg", "\\hspace{-1em}");
    add("rule-neg", "\\rule{-1em}{-1em}");
    add("rule-huge", "\\rule{1000000em}{1000000em}");
    add("rule-zero", "\\rule{0em}{0em}");
    add("color-nodef", "\\color{nope}");
    add("color-rgb-bad", "\\color[rgb]{2,3,4}x");
    add("color-html-bad", "\\color[HTML]{xyz}x");
    add(
        "definecolor-twice",
        "\\definecolor{c}{rgb}{1,0,0}\\definecolor{c}{rgb}{0,1,0}\\textcolor{c}{x}",
    );
    add("xarrow-opt-only", "\\xrightarrow[x]");
    add("limits-alone", "\\limits");
    add("limits-twice", "\\sum\\limits\\limits_a^b");
    add("not-end", "\\not");
    add("middle-alone", "\\middle|");
    add("tag-open", "\\tag{");
    add("label-nul", "\\label{\0}");
    add("ref-empty", "\\ref{}");
    add("over-twice", "{a\\over b\\over c}");
    add("over-top", "a\\over b");
    add("choose-top", "n\\choose k");
    add("over-empty", "{\\over}");
    add("prime-run", &"'".repeat(300));
    add("prime-then-script", "f'''^2_3");
    add("all-ascii-punct", "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~");
    add("digits-huge", &"9".repeat(500));
    add("digits-fraction", &format!("0.{}", "0".repeat(400)));
    add("spaces-many", &" ".repeat(10000));
    add("newlines-many", &"\n".repeat(10000));
    add("tabs-x-tabs", "\t\t\tx\t\t\t");
    add("comment-only", "% only a comment");
    add("comment-crlf", "x%c\r\ny");
    add("comment-then-brace", "{%\n}");
    add("comment-in-cmd", "\\fr%\nac{1}{2}");
    add("cmd-then-comment", "\\alpha%\nx");
    add("surrogate-adjacent", "\u{d7ff}\u{e000}");
    add("noncharacters", "\u{fdd0}\u{ffff}");
    add("line-sep", "x\u{2028}y\u{2029}z");
    add("vt-ff", "x\u{b}y\u{c}z");
    add("zero-width", "x\u{200b}y\u{200c}z\u{200d}w");
    v
}

/// Size-parameterised families, `(name, builder)`. `n` is the size knob.
/// A named size-parameterised builder.
pub type Family = (&'static str, fn(usize) -> String);

/// Size-parameterised families.
pub fn families() -> Vec<Family> {
    vec![
        ("x-run", |n| "x".repeat(n)),
        ("sum-terms", |n| "a+".repeat(n)),
        ("braces-nest", |n| {
            format!("{}{}", "{".repeat(n), "}".repeat(n))
        }),
        ("braces-open", |n| "{".repeat(n)),
        ("braces-close", |n| "}".repeat(n)),
        ("sup-nest", |n| {
            format!("{}x{}", "x^{".repeat(n), "}".repeat(n))
        }),
        ("sub-chain", |n| "x_".repeat(n)),
        ("caret-chain", |n| "^".repeat(n)),
        ("frac-nest", |n| {
            format!("{}1{}", "\\frac{1}{".repeat(n), "}".repeat(n))
        }),
        ("sqrt-nest", |n| {
            format!("{}x{}", "\\sqrt{".repeat(n), "}".repeat(n))
        }),
        ("left-nest", |n| {
            format!("{}x{}", "\\left(".repeat(n), "\\right)".repeat(n))
        }),
        ("left-open", |n| "\\left(".repeat(n)),
        ("paren-open", |n| "(".repeat(n)),
        ("paren-row-nest", |n| {
            format!("{}{}", "{(".repeat(n), ")".repeat(n))
        }),
        ("bracket-opt", |n| {
            format!("{}x{}", "\\sqrt[".repeat(n), "]{y}".repeat(n))
        }),
        ("text-long", |n| format!("\\text{{{}}}", "a b ".repeat(n))),
        ("text-dollar", |n| format!("\\text{{{}}}", "$x$ ".repeat(n))),
        ("prime-run", |n| format!("f{}", "'".repeat(n))),
        ("matrix-rows", |n| {
            format!("\\begin{{matrix}}{}\\end{{matrix}}", "a&b\\\\".repeat(n))
        }),
        ("matrix-cols", |n| {
            format!("\\begin{{matrix}}{}\\end{{matrix}}", "a&".repeat(n))
        }),
        ("matrix-nest", |n| {
            format!(
                "{}x{}",
                "\\begin{matrix}".repeat(n),
                "\\end{matrix}".repeat(n)
            )
        }),
        ("align-rows", |n| {
            format!("\\begin{{align}}{}\\end{{align}}", "a&=b\\\\".repeat(n))
        }),
        ("thin-spaces", |n| "\\,".repeat(n)),
        ("spaces", |n| " ".repeat(n)),
        ("comments", |n| "%\n".repeat(n)),
        ("row-breaks", |n| "\\\\".repeat(n)),
        ("over-chain", |n| format!("{{{}1}}", "1\\over ".repeat(n))),
        ("mathrm-nest", |n| {
            format!("{}x{}", "\\mathrm{".repeat(n), "}".repeat(n))
        }),
        ("color-run", |n| format!("{}x", "\\color{red}".repeat(n))),
        ("group-run", |n| "{x}".repeat(n)),
        ("unicode-run", |n| "é\u{301}\u{1f600}\u{5d0}".repeat(n)),
        ("bs-space-run", |n| "\\ ".repeat(n)),
        ("big-run", |n| "\\bigl(".repeat(n)),
        ("overset-nest", |n| {
            format!("{}x{}", "\\overset{a}{".repeat(n), "}".repeat(n))
        }),
        ("limits-run", |n| format!("\\sum{}", "_a^b".repeat(n))),
        ("number-long", |n| "7".repeat(n)),
        ("dimension-long", |n| {
            format!("\\hspace{{{}em}}", "9".repeat(n))
        }),
    ]
}

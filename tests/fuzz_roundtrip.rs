//! Fuzz round-trip regressions. Each input is a minimized crash of the
//! `parse` fuzz target: parse → `to_latex` → parse must give the identical
//! tree and a stable print, or the input must be rejected outright.

use latex_rust::parse;

fn round_trip(input: &str) {
    let node = match parse(input) {
        Ok(node) => node,
        Err(err) => panic!("{input:?} did not parse: {err}"),
    };
    let printed = node.to_latex();
    let again = match parse(&printed) {
        Ok(again) => again,
        Err(err) => panic!("reprint {printed:?} of {input:?}: {err}"),
    };
    assert_eq!(again, node, "input {input:?} printed {printed:?}");
    assert_eq!(again.to_latex(), printed, "reprint of {input:?} not stable");
}

/// `$ & _ ^` in text must be escaped, or the reprint stops at the token.
#[test]
fn text_specials_are_escaped() {
    for input in [
        "\\mbox{\\$}",
        "\\text{\\&}",
        "\\mbox{\\$\0}",
        "\\text{\0\\&\0}",
        "\0\\mbox{\\$\0}",
        "\0\\mbox{t\\${}?}l(",
        "\0k'6\\mbox{\\#Q\0\\$\0\0\0\0\0\0\0\0\0\0}oxs\u{10}",
        "\\text{\\_}",
        "\\operatorname{a\\_b}",
    ] {
        round_trip(input);
    }
}

/// `\` before a newline or tab is a control space, as in TeX.
#[test]
fn control_whitespace_in_text_is_a_space() {
    for input in [
        "\\text{\\\n}",
        "\\text{\\\t}",
        "\\text{a\\\nb}",
        "\\text{\\\r}",
    ] {
        round_trip(input);
    }
}

/// A printed text group must not look like the `{ )}` / `{[}` shorthand the
/// preprocessor unbraces.
#[test]
fn text_groups_survive_the_preprocessor() {
    for input in ["\\text{\\ (}", "\\text{\\ ( }"] {
        round_trip(input);
    }
}

/// Styled text that is empty or not all letters must keep its text form.
#[test]
fn styled_text_keeps_its_body() {
    for input in [
        "{\\it\\text{}}",
        "{\\it{\\text{}}}",
        "\0{\\it\\mbox{}}",
        "{\\it\\mbox{\u{2}}}",
        "{\\it{i'\\text{}}}",
        "T{\\it\\text{[\u{e}\u{e}}}",
        "h{\\it{i'\\text{  }<}<<}",
        "h{\\it{rqi'\\text{     }\\ \\<<<<} \\<<<<}",
        "{\\rm\\text{}}",
        "{\\bf\\text{1 2}}",
        "\\mathbb{\\text{x y}}",
    ] {
        round_trip(input);
    }
    // `\its` is one control word, not `\it` + `s`. The old string rewrite
    // matched the prefix; the token rule (spec 3.1) does not.
    for (input, word) in [
        ("{\\its\\mbox{\u{2}P]}}", "\\its"),
        ("{\\itsM\0\\mbox{{\u{2}{t\u{1f}h}}]}}Is", "\\itsM"),
    ] {
        assert_eq!(
            parse(input),
            Err(latex_rust::ParseError::Unknown(word.into())),
            "{input:?}"
        );
    }
}

/// `\choose` and `\binom` with both sides empty are rejected, like
/// `\frac{}{}` and `{\over}`. One empty side still parses and round-trips.
#[test]
fn empty_binomials_are_rejected() {
    for input in ["{\\choose}", "-{\\choose }", "\\binom{}{}"] {
        assert!(parse(input).is_err(), "{input:?} should be rejected");
    }
    // Explicit control space is content, not an empty side.
    for input in [
        "{a\\choose}",
        "{\\choose b}",
        "\\binom{a}{}",
        "{\\choose \\ }",
        "\0\0l-osem{\\it\tse\tlAh\\{\0l-o`n\\ U\u{1}e}{\\it\t\\choose \\ }",
    ] {
        round_trip(input);
    }
}

/// Group 6: a vertical tab or form feed was a math atom, but the
/// preprocessor trims it as whitespace, so a reprinted lone atom vanished.
#[test]
fn vertical_tab_and_form_feed_are_whitespace() {
    for input in [
        " $$ \u{c} $$  ",
        "$\u{b}$",
        "\n$\u{b}$\n\n\n ",
        "$$\r\r\r\r\r\r\r\r\r\r\u{c}\r\r\r\r\r$$",
        "{\u{b}}",
        "a\u{c}b",
        "x^\u{b}2",
    ] {
        round_trip(input);
    }
}

/// Group 7: the `\over` / `\choose` preprocessor read `\\over` as `\over` and
/// counted an escaped `\{` as a group, so printed text was rewritten.
#[test]
fn escaped_backslash_and_brace_hide_infix_commands() {
    for input in ["\\text{\\{a\\}}", "{a \\over b}", "{\\{a\\} \\choose b}"] {
        round_trip(input);
    }
    // `\over` and `\choose` in text are not valid TeX and `\text{\over}` was
    // already rejected. With an escaped `\{` no longer counted as a group,
    // these fuzz inputs are rejected the same way.
    for input in [
        "\0mb\\mbox{I\\choose'III\\{itc}h",
        ")\\text{\\over\0{\\{ }}",
        "\u{1}\\text{A\\over{{\0{\\{\\)}}}}",
    ] {
        assert!(parse(input).is_err(), "{input:?} should be rejected");
    }
}

/// Group 8: the `{ )}` / `{[}` preprocessor rewrite matched the brace of an
/// escaped `\{`, so `\text{\{ )}` lost its closing brace. A control space
/// `\ ` prints as a plain space in text, which builds that byte sequence.
#[test]
fn escaped_brace_hides_delimiter_groups() {
    for input in [
        // fuzz finding and its minimization
        "(???????\\text {\\{\\ ) }\\ ) t{}Bb",
        "\\text{\\{\\ )}",
        // variants
        "\\text{\\{\\ ) }",
        "\\text{\\{\\ (}",
        "\\text{\\{ )}",
        "\\text{\\{[}",
        "\\text{\\{]}",
        "\\text{a\\{\\ ( }b",
        "\\text{{\\{\\ )}}",
        "\\text{\\{}",
        "\\text{\\}}",
        "\\text{\\}\\ )}",
        "\\text{a\\ }",
        "\\text{\\{\\ }",
        "\\text{\\{\\ )}\\ ",
        "\\text{}",
        // unescaped groups are still rewritten
        "{ (}a{ )}",
        "{[}a{]}",
    ] {
        round_trip(input);
    }
}

/// Group 9: text bodies decoded `\)` to a bare `)`, and the reprint
/// `(\text{)'}` then fooled the math-level `(` script lookahead, which
/// counted `)` tokens inside brace groups. Under the text-mode spec a stray
/// `\)` in text is an error, so the variants use bare parens, which reprint
/// the same way. See documents/TEXT_MODE_SPEC.md.
#[test]
fn paren_lookahead_skips_text_bodies() {
    for input in [
        // the minimization with the stray `\)` written as a bare `)`
        "(\\text{{})'{}}",
        // depth 1-3; `\ ` at start, middle, end; with and without `)`
        "(\\text{\\ )'}",
        "(\\text{a\\ )'b}",
        "(\\text{)'\\ }",
        "(\\text{\\ a}",
        "(\\text{\\text{\\ })'}",
        "(\\text{\\text{\\ )'}}",
        "(\\text{x\\text{\\ }y)'}\\ ",
        "(\\text{\\text{\\ }}",
        "(\\text{\\text{\\text{\\ })'}}",
        "(\\text{\\text{\\text{)'\\ }}}",
        "(\\text{\\text{\\text{a\\ b}})'}",
        "(\\text{\\text{\\text{\\ }}}",
        // escaped braces
        "(\\text{\\{)'\\}}",
        "(\\text{\\{\\ \\})'}",
        "(\\text{\\text{\\{\\ })'\\}}",
        // balanced math in text
        "(\\text{\\(x\\))'}",
        "(\\text{$)$'})'",
        // `)` after the text group
        "(\\text{)'})'",
        "(\\text{\\ })^2",
        "(a)'",
        "((a)')'",
    ] {
        round_trip(input);
    }
    // The fuzz finding itself has a stray `\)` in text.
    for input in [
        "(???????\\text {\\text {\\ }\\)') t{Lambda}Bb) }\\ ",
        "(\\text{{}\\)'{}}",
    ] {
        assert!(parse(input).is_err(), "{input:?} should be rejected");
    }
}

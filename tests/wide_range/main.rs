//! Wide-range test layer (one test binary, many modules).
//!
//! Narrow tests pass on typical inputs and miss wrong answers elsewhere. This
//! layer sweeps the whole supported grammar, its edges and adversarial inputs,
//! and checks the same promises everywhere:
//!
//! - no panic, and a typed error or a tree;
//! - `print(parse(x))` reparses to the same tree and the print is a fixed
//!   point; the reprint is never nested deeper than the input;
//! - layout, SVG, PNG and egui never panic and never give NaN or infinite
//!   sizes; time stays bounded.
//!
//! Layers: `gen` (seeded grammar generator), `mutate` (corruptions),
//! `oracle` (hand-countable "must be rejected" rules), `families` (fixed and
//! size-parameterised adversarial inputs), `corpus` (repo and Accumath
//! strings), `exhaustive` (every short string over structural tokens).
//!
//! Case counts: `LATEX_RUST_WIDE_CASES` (default 4096) scales the generated
//! layers; `LATEX_RUST_WIDE_EXH_LEN` sets the exhaustive length. All seeds are
//! fixed. Failures print the seed or input and a minimized input.

mod corpus;
mod exhaustive;
mod extract;
mod families;
mod gen;
mod mutate;
mod oracle;
mod props;
mod render;
mod rng;
mod semantic;
mod timing;
mod vocab;

use gen::Gen;
use latex_rust::{parse, parse_with_options, MathFont, ParseError};
use props::{options_from_bits, report, Tally};
use rng::Rng;
use vocab::{Vocab, HOSTILE};

/// Base seed. Case `i` of a layer uses `SEED + i` (plus a layer offset).
const SEED: u64 = 0x1A7E_5EED_2026_1015;

fn cases(default: usize) -> usize {
    std::env::var("LATEX_RUST_WIDE_CASES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn font() -> Result<MathFont, String> {
    MathFont::stix_two_math().map_err(|e| format!("font: {e}"))
}

fn threads() -> usize {
    std::thread::available_parallelism()
        .map_or(2, |n| n.get())
        .min(16)
}

fn fnv(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

// ---------------------------------------------------------------- generated

#[test]
fn generated_inputs_hold_every_property() -> Result<(), String> {
    let v = Vocab::load();
    let n = cases(4096);
    let mut tally = Tally::new();
    let mut longest = 0usize;
    for i in 0..n {
        let mut g = Gen::new(SEED + i as u64, &v);
        let src = g.expr(1 + i % 7, 4 + (i * 7) % 60);
        longest = longest.max(src.len());
        let bits = (i % 16) as u8;
        let with_depth = i % 4 == 0 && src.len() <= 600;
        let c = report(
            &src,
            bits,
            with_depth,
            &format!("generated case {i} (seed {})", SEED + i as u64),
        )?;
        tally.add(c);
    }
    eprintln!(
        "generated: {n} cases, longest {longest} bytes; {}",
        tally.line()
    );
    if tally.counts[7] * 10 < tally.total() {
        return Err(format!("fewer than 10% accepted: {}", tally.line()));
    }
    Ok(())
}

/// Valid-by-construction inputs must be accepted. The strict generator only
/// writes constructs the grammar documents, so a rejection is a wrong
/// rejection (or a generator bug to fix); any accepted input must also keep
/// every other property.
#[test]
fn valid_inputs_are_accepted() -> Result<(), String> {
    let v = Vocab::load();
    let n = cases(4096);
    let mut rejected: Vec<String> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for i in 0..n {
        let seed = SEED ^ 0x57A1_C700 ^ ((i as u64) << 7);
        let mut g = Gen::new(seed, &v);
        g.strict = true;
        let src = g.expr(1 + i % 6, 3 + (i * 5) % 40);
        let bits = (i % 16) as u8;
        report(
            &src,
            bits,
            i % 4 == 0 && src.len() <= 600,
            &format!("strict case {i} (seed {seed})"),
        )?;
        if let Err(e) = parse_with_options(&src, &options_from_bits(bits)) {
            let key = e.to_string();
            if !seen.iter().any(|k| *k == key) && rejected.len() < 12 {
                seen.push(key);
                let small = props::minimize_with(&src, &|s| {
                    parse_with_options(s, &options_from_bits(bits))
                        .map_or_else(|e2| e2.to_string() == e.to_string(), |_| false)
                });
                rejected.push(format!(
                    "case {i} bits={bits}: {e}\n    minimal {small:?}\n    input {:?}",
                    src.chars().take(300).collect::<String>()
                ));
            }
            rejected.push(String::new());
        }
    }
    let bad = rejected.iter().filter(|s| s.is_empty()).count();
    eprintln!("strict: {n} cases, {bad} rejected");
    if bad > 0 {
        let shown: Vec<&str> = rejected
            .iter()
            .map(String::as_str)
            .filter(|s| !s.is_empty())
            .collect();
        return Err(format!(
            "{bad} of {n} valid-by-construction inputs rejected; first:\n{}",
            shown.join("\n")
        ));
    }
    Ok(())
}

#[test]
fn mutated_inputs_hold_every_property() -> Result<(), String> {
    let v = Vocab::load();
    let n = cases(4096);
    let mut tally = Tally::new();
    for i in 0..n {
        let seed = SEED ^ 0x00AB_CDEF ^ ((i as u64) << 8);
        let mut g = Gen::new(seed, &v);
        let mut src = g.expr(1 + i % 5, 3 + (i * 5) % 30);
        let mut r = Rng::new(seed ^ 0x55);
        for _ in 0..1 + i % 3 {
            src = mutate::mutate(&mut r, &src, HOSTILE);
        }
        let bits = ((i / 3) % 16) as u8;
        let c = report(
            &src,
            bits,
            i % 4 == 0 && src.len() <= 600,
            &format!("mutated case {i} (seed {seed})"),
        )?;
        tally.add(c);
    }
    eprintln!("mutated: {n} cases; {}", tally.line());
    eprintln!("known findings so far: {}", props::known_line());
    Ok(())
}

#[test]
fn random_hostile_strings_hold_every_property() -> Result<(), String> {
    let n = cases(4096);
    let pool: Vec<&str> = {
        let mut p: Vec<&str> = HOSTILE.iter().copied().collect();
        p.extend([
            "{",
            "}",
            "\\",
            "^",
            "_",
            "&",
            "$",
            "%",
            "\\\\",
            "\\left",
            "\\right",
            "(",
            ")",
            "x",
            "1",
            " ",
            "\\text",
            "'",
            "[",
            "]",
            "\\begin{matrix}",
            "\\end{matrix}",
            "\\frac",
            "\\sqrt",
            "\\over",
        ]);
        p
    };
    let mut tally = Tally::new();
    for i in 0..n {
        let mut r = Rng::new(SEED ^ 0x4057_11E5 ^ ((i as u64) << 4));
        let len = r.below(48);
        let mut src = String::new();
        for _ in 0..len {
            src.push_str(pool.get(r.below(pool.len())).copied().unwrap_or(""));
        }
        let c = report(&src, (i % 16) as u8, true, &format!("hostile string {i}"))?;
        tally.add(c);
    }
    eprintln!("hostile: {n} strings; {}", tally.line());
    Ok(())
}

/// The generator must reach every name in the vocabulary tables, and every
/// command the parser source knows must be in a table (drift guard).
#[test]
fn generator_covers_the_lexicon() -> Result<(), String> {
    let v = Vocab::load();
    let mut all = String::new();
    for i in 0..30_000usize {
        let mut g = Gen::new(SEED + 1_000_000 + i as u64, &v);
        all.push_str(&g.expr(1 + i % 7, 4 + (i * 7) % 60));
        all.push('\n');
    }
    let mut missing: Vec<String> = Vec::new();
    let tables: [(&str, &[&str]); 8] = [
        ("frac", vocab::FRACS),
        ("accent", vocab::ACCENTS),
        ("font", vocab::FONTS),
        ("text", vocab::TEXTS),
        ("operator", vocab::OPERATORS),
        ("switch", vocab::SWITCHES),
        ("size", vocab::SIZES),
        ("delim", vocab::DELIMS),
    ];
    for (kind, table) in tables {
        for name in table {
            let hit = if kind == "delim" {
                all.contains(name)
            } else if name.starts_with('\\') {
                all.contains(name)
            } else {
                all.contains(&format!("\\{name}"))
            };
            if !hit {
                missing.push(format!("{kind}:{name}"));
            }
        }
    }
    for env in vocab::ENVS {
        if !env.is_empty() && !all.contains(&format!("\\begin{{{env}}}")) {
            missing.push(format!("env:{env}"));
        }
    }
    for sym in &v.symbols {
        if !all.contains(sym.as_str()) {
            missing.push(format!("symbol:{sym}"));
        }
    }
    // Drift guard: identifier literals in the parser sources that the parser
    // accepts as commands must appear in generated text.
    let sources = [
        include_str!("../../src/parser/parse/drive.rs"),
        include_str!("../../src/parser/parse.rs"),
        include_str!("../../src/parser/group_rules.rs"),
        include_str!("../../src/parser/text.rs"),
        include_str!("../../src/parser/ast.rs"),
        include_str!("../../src/parser/parse/lexicon.rs"),
    ];
    let mut words: Vec<&str> = Vec::new();
    for src in sources {
        for piece in src.split('"').skip(1).step_by(2) {
            if piece.len() >= 2 && piece.chars().all(|c| c.is_ascii_alphabetic()) {
                words.push(piece);
            }
        }
    }
    words.sort_unstable();
    words.dedup();
    let mut checked = 0usize;
    for w in words {
        let cmd = format!("\\{w}");
        let known = !matches!(parse(&cmd), Err(ParseError::Unknown(_)));
        if known {
            checked += 1;
            if !all.contains(&cmd) {
                missing.push(format!("parser-command:{w}"));
            }
        }
    }
    eprintln!(
        "lexicon: {} symbols, {checked} parser commands, {} bytes generated",
        v.symbols.len(),
        all.len()
    );
    if missing.is_empty() {
        Ok(())
    } else {
        let shown = missing.len().min(40);
        Err(format!(
            "{} names never generated: {}",
            missing.len(),
            missing
                .iter()
                .take(shown)
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        ))
    }
}

// ----------------------------------------------------------------- oracles

#[test]
fn unbalanced_braces_are_rejected() -> Result<(), String> {
    let v = Vocab::load();
    let n = cases(4096);
    let (mut tested, mut skipped) = (0usize, 0usize);
    for i in 0..n {
        let seed = SEED ^ 0x0B4A_CE00 ^ ((i as u64) << 6);
        let mut g = Gen::new(seed, &v);
        let src = g.expr(1 + i % 6, 3 + (i * 3) % 40);
        let Some((net, at)) = oracle::braces(&src) else {
            skipped += 1;
            continue;
        };
        if net != 0 || at.is_empty() || parse(&src).is_err() {
            skipped += 1;
            continue;
        }
        tested += 1;
        let mut r = Rng::new(seed);
        let (pos, _) = at.get(r.below(at.len())).copied().unwrap_or((0, '{'));
        let cut = oracle::delete_at(&src, pos);
        if parse(&cut).is_ok() {
            return Err(format!("case {i}: one unescaped brace deleted, still accepted\n  base {src:?}\n  cut  {cut:?}"));
        }
        let extra_close = format!("{src}}}");
        if parse(&extra_close).is_ok() {
            return Err(format!(
                "case {i}: stray closing brace accepted: {extra_close:?}"
            ));
        }
        let extra_open = format!("{{{src}");
        if parse(&extra_open).is_ok() {
            return Err(format!(
                "case {i}: stray opening brace accepted: {extra_open:?}"
            ));
        }
    }
    eprintln!("brace oracle: {tested} accepted bases mutated, {skipped} skipped");
    if tested < n / 20 {
        return Err(format!("only {tested} usable bases of {n}"));
    }
    Ok(())
}

#[test]
fn unpaired_left_right_and_env_are_rejected() -> Result<(), String> {
    let v = Vocab::load();
    let n = cases(4096);
    let mut tested = 0usize;
    for i in 0..n {
        let seed = SEED ^ 0x1EF7_0000 ^ ((i as u64) << 6);
        let mut g = Gen::new(seed, &v);
        let src = g.expr(1 + i % 6, 3 + (i * 3) % 40);
        if parse(&src).is_err() {
            continue;
        }
        tested += 1;
        for tail in ["\\right)", "\\end{matrix}", "\\end{cases}", "\\\\\\right."] {
            let bad = format!("{src}{tail}");
            if parse(&bad).is_ok() {
                return Err(format!("case {i}: extra {tail:?} accepted: {bad:?}"));
            }
        }
        for head in ["\\left(", "\\begin{matrix}"] {
            let bad = format!("{head}{src}");
            if parse(&bad).is_ok() {
                return Err(format!(
                    "case {i}: opener {head:?} never closed, accepted: {bad:?}"
                ));
            }
        }
        // The body of `\operatorname{..}` is read as a literal name (W-OPNAME-1),
        // so a `\left` or `\right` inside it pairs with nothing; skip those.
        if src.contains("\\operatorname") {
            continue;
        }
        let lefts = oracle::count_word(&src, "left");
        let rights = oracle::count_word(&src, "right");
        if lefts != rights {
            return Err(format!(
                "case {i}: {lefts} \\left and {rights} \\right accepted: {src:?}"
            ));
        }
        if let Some(at) = src.rfind("\\right") {
            let mut cut = String::from(src.get(..at).unwrap_or(""));
            cut.push_str(src.get(at + 6..).unwrap_or(""));
            if oracle::count_word(&cut, "left") != oracle::count_word(&cut, "right")
                && parse(&cut).is_ok()
            {
                return Err(format!(
                    "case {i}: a \\right removed, still accepted\n  base {src:?}\n  cut  {cut:?}"
                ));
            }
        }
    }
    eprintln!("pairing oracle: {tested} accepted bases");
    Ok(())
}

// --------------------------------------------------------------- fixed sets

#[test]
fn fixed_adversarial_inputs_hold_every_property() -> Result<(), String> {
    let mut inputs = families::fixed_inputs();
    for b in 0u8..=127 {
        let c = char::from(b);
        inputs.push((format!("byte-{b}"), c.to_string()));
        inputs.push((format!("bs-byte-{b}"), format!("\\{c}")));
        inputs.push((format!("x-byte-{b}"), format!("x{c}x")));
        inputs.push((format!("brace-byte-{b}"), format!("{{{c}}}")));
        inputs.push((format!("script-byte-{b}"), format!("x^{c}_{c}")));
    }
    for h in HOSTILE {
        inputs.push((format!("hostile-{h:?}"), String::from(*h)));
        inputs.push((format!("hostile-text-{h:?}"), format!("\\text{{{h}}}")));
        inputs.push((
            format!("hostile-script-{h:?}"),
            format!("x^{{{h}}}_{{{h}}}"),
        ));
        inputs.push((format!("hostile-cmd-{h:?}"), format!("\\{h}")));
    }
    for env in vocab::ENVS {
        inputs.push((
            format!("env-{env}"),
            format!("\\begin{{{env}}}\\end{{{env}}}"),
        ));
        inputs.push((
            format!("env-cell-{env}"),
            format!("\\begin{{{env}}}1&2\\\\3\\end{{{env}}}"),
        ));
        inputs.push((format!("env-open-{env}"), format!("\\begin{{{env}}}x")));
    }
    for d in vocab::DELIMS {
        inputs.push((format!("delim-{d}"), format!("\\left{d}x\\right{d}")));
        inputs.push((
            format!("delim-mid-{d}"),
            format!("\\left{d}x\\middle{d}y\\right{d}"),
        ));
    }
    for (name, src) in &inputs {
        for bits in 0u8..16 {
            report(src, bits, true, &format!("fixed {name}"))?;
        }
    }
    eprintln!("fixed: {} inputs x 16 option sets", inputs.len());
    Ok(())
}

#[test]
fn size_families_hold_every_property() -> Result<(), String> {
    let sizes: &[usize] = if cfg!(debug_assertions) {
        &[0, 1, 2, 3, 4, 7, 16, 31, 32, 33, 34, 65, 200, 1000, 4000]
    } else {
        &[
            0, 1, 2, 3, 4, 7, 16, 31, 32, 33, 34, 65, 200, 1000, 4000, 20_000,
        ]
    };
    let mut runs = 0usize;
    for (name, build) in families::families() {
        for &n in sizes {
            let src = build(n);
            let bits = (n % 16) as u8;
            report(
                &src,
                bits,
                src.len() <= 600,
                &format!("family {name} n={n}"),
            )?;
            runs += 1;
        }
    }
    eprintln!("families: {runs} runs");
    Ok(())
}

/// Padding that TeX ignores must not change the tree.
#[test]
fn padding_does_not_change_the_tree() -> Result<(), String> {
    let v = Vocab::load();
    let n = cases(4096);
    let mut tested = 0usize;
    for i in 0..n {
        let seed = SEED ^ 0x9AD0_0000 ^ ((i as u64) << 5);
        let mut g = Gen::new(seed, &v);
        let src = g.expr(1 + i % 6, 3 + (i * 3) % 40);
        let opts = options_from_bits((i % 16) as u8);
        let Ok((base, _)) = parse_with_options(&src, &opts) else {
            continue;
        };
        // Non-ASCII edge whitespace is trimmed or kept as an atom depending on
        // a trailing comment (documented, canonical_print_props); skip it.
        let edge = |c: Option<char>| c.is_some_and(|c| c.is_whitespace() && !c.is_ascii());
        let core = src.trim_matches(|c: char| c.is_ascii_whitespace());
        if edge(core.chars().next()) || edge(core.chars().next_back()) {
            continue;
        }
        tested += 1;
        for (name, padded) in [
            ("trailing space", format!("{src} ")),
            ("trailing newline", format!("{src}\n")),
            ("trailing comment", format!("{src}%c")),
            ("trailing comment line", format!("{src}%c\n")),
            ("leading space", format!(" {src}")),
            ("leading newline", format!("\n{src}")),
            ("leading comment", format!("%c\n{src}")),
        ] {
            match parse_with_options(&padded, &opts) {
                Ok((t, _)) if t == base => {}
                Ok(_) => return Err(format!("case {i}: {name} changed the tree\n  base   {src:?}\n  padded {padded:?}")),
                Err(e) => return Err(format!("case {i}: {name} made a valid input fail: {e}\n  base   {src:?}\n  padded {padded:?}")),
            }
        }
    }
    eprintln!("padding: {tested} accepted bases x 7 paddings");
    Ok(())
}

// ------------------------------------------------------------------- render

#[test]
fn render_properties_on_generated_inputs() -> Result<(), String> {
    let f = font()?;
    let v = Vocab::load();
    let n = cases(4096) / 4;
    let mut laid_out = 0usize;
    for i in 0..n {
        let seed = SEED ^ 0x4E4D_0000 ^ ((i as u64) << 4);
        let mut g = Gen::new(seed, &v);
        let src = g.expr(1 + i % 6, 3 + (i * 5) % 40);
        let ok = render::check_render(&src, &f, i % 2 == 0)
            .map_err(|e| format!("render case {i} (seed {seed}): {e}\n  input {src:?}"))?;
        if ok {
            laid_out += 1;
            if i % 8 == 0 {
                render::check_png(&src, &f)
                    .map_err(|e| format!("png case {i}: {e}\n  input {src:?}"))?;
            }
            if i % 4 == 0 {
                render::check_egui(&src, &f)
                    .map_err(|e| format!("egui case {i}: {e}\n  input {src:?}"))?;
            }
        }
    }
    eprintln!(
        "render: {n} cases, {laid_out} laid out, known W-SVG-1 hits {}",
        render::KNOWN_SVG_NEGATIVE.load(std::sync::atomic::Ordering::Relaxed)
    );
    if laid_out * 10 < n {
        return Err(format!("only {laid_out} of {n} inputs reached layout"));
    }
    Ok(())
}

#[test]
fn render_properties_on_fixed_inputs() -> Result<(), String> {
    let f = font()?;
    let mut laid_out = 0usize;
    for (name, src) in families::fixed_inputs() {
        if src.len() > 2000 {
            continue;
        }
        let ok = render::check_render(&src, &f, false)
            .map_err(|e| format!("render {name}: {e}\n  input {src:?}"))?;
        render::check_egui(&src, &f).map_err(|e| format!("egui {name}: {e}"))?;
        if ok {
            laid_out += 1;
        }
    }
    for (name, build) in families::families() {
        for n in [1usize, 5, 20, 200] {
            let src = build(n);
            let ok = render::check_render(&src, &f, n % 2 == 0)
                .map_err(|e| format!("render family {name} n={n}: {e}"))?;
            if ok {
                laid_out += 1;
            }
        }
    }
    eprintln!("render fixed: {laid_out} laid out");
    Ok(())
}

// ----------------------------------------------------------------- semantic

#[test]
fn spacing_widths_are_exact() -> Result<(), String> {
    let f = font()?;
    let n = semantic::spacing_widths(&f)?;
    let m = semantic::widths_add(&f)?;
    eprintln!("semantic: {n} spacing widths, {m} additivity cases");
    Ok(())
}

#[test]
fn rule_counts_match_tex() -> Result<(), String> {
    let f = font()?;
    let n = semantic::rule_counts(&f)?;
    eprintln!("semantic: {n} rule-count cases");
    Ok(())
}

// ------------------------------------------------------------------- timing

#[test]
fn timing_guards_hold_on_size_families() -> Result<(), String> {
    let small: usize = if cfg!(debug_assertions) { 250 } else { 1000 };
    for (name, build) in families::families() {
        let a = build(small);
        let b = build(small * 4);
        let run = |s: &str| {
            if let Ok(t) = parse(s) {
                let _printed = t.to_latex();
            }
        };
        timing::guard_scaling(name, 4, 40, || run(&a), || run(&b))?;
    }
    Ok(())
}

/// Nested shapes below the depth limit must not blow up exponentially.
#[test]
fn nested_shapes_stay_polynomial() -> Result<(), String> {
    type Shape = (&'static str, fn(usize) -> String);
    let shapes: [Shape; 8] = [
        ("paren-row", |k| {
            format!("{}{}", "{(".repeat(k), ")".repeat(k))
        }),
        ("frac", |k| {
            format!("{}x{}", "\\frac{1}{".repeat(k), "}".repeat(k))
        }),
        ("sup", |k| format!("{}x{}", "x^{".repeat(k), "}".repeat(k))),
        ("left-right", |k| {
            format!("{}x{}", "\\left(".repeat(k), "\\right)".repeat(k))
        }),
        ("over", |k| {
            format!("{}1{}", "{1\\over ".repeat(k), "}".repeat(k))
        }),
        ("choose", |k| {
            format!("{}1{}", "{1\\choose ".repeat(k), "}".repeat(k))
        }),
        ("text-math", |k| {
            format!("{}x{}", "\\text{$".repeat(k), "$}".repeat(k))
        }),
        ("sqrt-opt", |k| {
            format!("{}x{}", "\\sqrt[".repeat(k), "]{y}".repeat(k))
        }),
    ];
    for (name, build) in shapes {
        let lo = build(8);
        let hi = build(30);
        let t_lo = timing::best_of(5, || {
            if let Ok(t) = parse(&lo) {
                let _p = t.to_latex();
            }
        });
        let t_hi = timing::best_of(5, || {
            if let Ok(t) = parse(&hi) {
                let _p = t.to_latex();
            }
        });
        let floor = timing::NOISE * 10;
        // 30/8 deep: a polynomial of degree 4 is under 200x; 2^22 is not.
        if t_hi > floor && t_hi > t_lo.max(timing::NOISE) * 200 {
            return Err(format!(
                "{name}: depth 30 took {t_hi:?}, depth 8 took {t_lo:?}"
            ));
        }
        if t_hi > timing::budget() {
            return Err(format!("{name}: depth 30 took {t_hi:?}"));
        }
    }
    Ok(())
}

// ------------------------------------------------------------------- corpus

fn run_corpus(items: &[corpus::Item], label: &str) -> Result<(), String> {
    let mut tally = Tally::new();
    for it in items {
        let mut accepted_any = false;
        for bits in 0u8..16 {
            let c = report(
                &it.latex,
                bits,
                it.latex.len() <= 600,
                &format!("{label} {}", it.origin),
            )?;
            if c == 7 {
                accepted_any = true;
            }
            if bits == 0 {
                tally.add(c);
            }
        }
        match it.must_parse {
            Some(true) if !accepted_any => {
                return Err(format!(
                    "{label} {}: a gold input parses under no option set: {:?}",
                    it.origin, it.latex
                ));
            }
            Some(false) if accepted_any => {
                return Err(format!(
                    "{label} {}: a gold error input is accepted: {:?}",
                    it.origin, it.latex
                ));
            }
            _ => {}
        }
    }
    eprintln!(
        "{label}: {} strings; default options: {}",
        items.len(),
        tally.line()
    );
    Ok(())
}

#[test]
fn repo_corpus_round_trips() -> Result<(), String> {
    let items = corpus::repo_items()?;
    if items.len() < 1500 {
        return Err(format!("only {} repo strings found", items.len()));
    }
    run_corpus(&items, "repo")
}

#[test]
fn accumath_corpus_round_trips() -> Result<(), String> {
    let items = corpus::accumath_items()?;
    if items.len() < 1000 {
        return Err(format!("only {} accumath strings found", items.len()));
    }
    run_corpus(&items, "accumath")
}

// --------------------------------------------------------------- exhaustive

fn exh_len(release: usize, debug: usize) -> usize {
    std::env::var("LATEX_RUST_WIDE_EXH_LEN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(if cfg!(debug_assertions) {
            debug
        } else {
            release
        })
}

fn exhaust(label: &str, tokens: &[&str], max_len: usize) -> Result<(), String> {
    let want = exhaustive::count_strings(tokens.len(), max_len);
    let workers = threads();
    let results: Vec<Result<(u64, [usize; 8]), String>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|w| {
                scope.spawn(move || {
                    let mut tally = Tally::new();
                    let mut f = |s: &str| -> Result<(), String> {
                        let bits = (fnv(s) % 16) as u8;
                        let c = report(s, bits, true, label)?;
                        tally.add(c);
                        Ok(())
                    };
                    let n = exhaustive::for_each_string(tokens, max_len, w, workers, &mut f)?;
                    Ok((n, tally.counts))
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| Err(String::from("worker panicked")))
            })
            .collect()
    });
    let mut total = 0u64;
    let mut counts = [0usize; 8];
    let mut errors: Vec<String> = Vec::new();
    for r in results {
        match r {
            Ok((n, c)) => {
                total += n;
                for (a, b) in counts.iter_mut().zip(c.iter()) {
                    *a += *b;
                }
            }
            Err(e) => errors.push(e),
        }
    }
    if let Some(e) = errors.first() {
        return Err(format!("{} worker(s) failed; first:\n{e}", errors.len()));
    }
    let mut t = Tally::new();
    t.counts = counts;
    eprintln!(
        "exhaustive {label}: {} tokens, length <= {max_len}, {total} strings; {}",
        tokens.len(),
        t.line()
    );
    if total != want {
        return Err(format!("{label}: visited {total} strings, expected {want}"));
    }
    Ok(())
}

#[test]
fn exhaustive_enumerator_is_complete_and_ordered() -> Result<(), String> {
    let mut seen: Vec<String> = Vec::new();
    let mut f = |s: &str| -> Result<(), String> {
        seen.push(String::from(s));
        Ok(())
    };
    let n = exhaustive::for_each_string(&["a", "b"], 2, 0, 1, &mut f)?;
    let want = ["", "a", "b", "aa", "ab", "ba", "bb"];
    if n != 7 || seen.iter().map(String::as_str).collect::<Vec<_>>() != want {
        return Err(format!("enumeration was {seen:?}"));
    }
    if exhaustive::count_strings(2, 2) != 7
        || exhaustive::count_strings(16, 0) != 1
        || exhaustive::count_strings(0, 3) != 1
    {
        return Err(String::from("count_strings"));
    }
    // Split work: strides partition the set.
    let mut a = 0u64;
    let mut b = 0u64;
    exhaustive::for_each_string(&["a", "b"], 3, 0, 2, &mut |_| {
        a += 1;
        Ok(())
    })?;
    exhaustive::for_each_string(&["a", "b"], 3, 1, 2, &mut |_| {
        b += 1;
        Ok(())
    })?;
    if a + b != 15 {
        return Err(format!("strides saw {a}+{b}, expected 15"));
    }
    Ok(())
}

/// Structural tokens of the task: braces, scripts, `\`, `\left`/`\right`,
/// parentheses and brackets, `&`, `\\`, `\text`, `$`, a digit and a letter.
#[test]
fn exhaustive_structural_tokens() -> Result<(), String> {
    let tokens = [
        "{", "}", "^", "_", "\\", "\\left", "\\right", "(", ")", "[", "]", "&", "\\\\", "\\text",
        "$", "1", "a",
    ];
    exhaust("structural", &tokens, exh_len(5, 4))
}

#[test]
fn exhaustive_brace_script_delimiter_core() -> Result<(), String> {
    let tokens = ["{", "}", "^", "_", "\\left", "\\right", "(", ")", "x", "'"];
    exhaust("core", &tokens, exh_len(7, 5))
}

#[test]
fn exhaustive_environment_tokens() -> Result<(), String> {
    let tokens = [
        "\\begin{matrix}",
        "\\end{matrix}",
        "\\begin{cases}",
        "\\end{cases}",
        "&",
        "\\\\",
        "x",
        "{",
        "}",
        "\\hline",
        "\\left(",
        "\\right)",
    ];
    exhaust("env", &tokens, exh_len(5, 4))
}

#[test]
fn exhaustive_text_mode_tokens() -> Result<(), String> {
    let tokens = [
        "\\text{", "}", "$", "\\", "x", " ", "\\$", "\\(", "\\)", "{", "^",
    ];
    exhaust("text", &tokens, exh_len(6, 5))
}

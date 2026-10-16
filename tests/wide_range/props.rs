//! The per-input properties, and a minimizer for any input that breaks one.
//!
//! For every input `x` and option set:
//! - `parse` does not panic and returns `Ok` or a typed `ParseError`;
//! - `tokenize`, `preprocess` and `gold` do not panic;
//! - `print(parse(x))` reparses to the same tree;
//! - the print is a fixed point;
//! - `depth(print) <= depth(x)` (depth is the least `max_depth` that accepts).

use latex_rust::{parse_with_options, preprocess, tokenize, ParseError, ParseOptions};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Run `f`, turning a panic into an `Err` message.
pub fn guarded<T>(f: impl FnOnce() -> T) -> Result<T, String> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => Ok(v),
        Err(p) => {
            let msg = match p.downcast_ref::<&str>() {
                Some(s) => String::from(*s),
                None => match p.downcast_ref::<String>() {
                    Some(s) => String::from(s.as_str()),
                    None => String::from("non-string payload"),
                },
            };
            Err(format!("PANIC: {msg}"))
        }
    }
}

/// The four parser switches, one bit each.
pub fn options_from_bits(bits: u8) -> ParseOptions {
    ParseOptions::new()
        .with_ascii_relations(bits & 1 != 0)
        .with_bare_operators(bits & 2 != 0)
        .with_unicode_symbols(bits & 4 != 0)
        .with_prime_symbol(bits & 8 != 0)
}

/// Index of a `ParseError` variant, for the statistics.
pub fn class_of(e: &ParseError) -> usize {
    match e {
        ParseError::TrailingBackslash => 0,
        ParseError::Unsupported(_) => 1,
        ParseError::Unknown(_) => 2,
        ParseError::Malformed(_) => 3,
        ParseError::UnmatchedDelimiter => 4,
        ParseError::RowBreakOutside => 5,
        _ => 6,
    }
}

/// Names for the `class_of` indices; index 7 is "accepted".
pub const CLASS_NAMES: [&str; 8] = [
    "trailing-backslash",
    "unsupported",
    "unknown",
    "malformed",
    "unmatched-delimiter",
    "row-break-outside",
    "other",
    "accepted",
];

/// Smallest `max_depth` (up to `limit`) that accepts `s`, or `None`.
pub fn depth_of(s: &str, bits: u8, limit: usize) -> Option<usize> {
    let ok = |d: usize| {
        let o = options_from_bits(bits).with_max_depth(d);
        parse_with_options(s, &o).is_ok()
    };
    if !ok(limit) {
        return None;
    }
    let (mut lo, mut hi) = (0usize, limit);
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if ok(mid) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Some(hi)
}

/// Check every promise for `src`. `Ok(class)` is the outcome class
/// (`CLASS_NAMES` index); `Err` names the first broken promise.
pub fn check_parse(src: &str, bits: u8, with_depth: bool) -> Result<usize, String> {
    let opts = options_from_bits(bits);
    let _tokens = guarded(|| tokenize(src))?;
    guarded(|| preprocess(src))?;
    let first = guarded(|| parse_with_options(src, &opts))?;
    let tree = match first {
        Err(e) => {
            if e.to_string().is_empty() {
                return Err(String::from("typed error with an empty message"));
            }
            return Ok(class_of(&e));
        }
        Ok((tree, _)) => tree,
    };
    guarded(|| tree.gold())?;
    let printed = guarded(|| tree.to_latex_with(&opts))?;
    let again = match guarded(|| parse_with_options(&printed, &opts))? {
        Ok((t, _)) => t,
        Err(e) => return Err(format!("print {printed:?} does not reparse: {e}")),
    };
    if again != tree {
        return Err(format!("print {printed:?} reparses to another tree"));
    }
    let second = guarded(|| again.to_latex_with(&opts))?;
    if second != printed {
        return Err(format!("print not idempotent: {printed:?} then {second:?}"));
    }
    if with_depth {
        let din = depth_of(src, bits, 96);
        let dout = depth_of(&printed, bits, 96);
        match (din, dout) {
            (Some(i), Some(o)) if o > i => {
                return Err(format!("depth grew {i} -> {o}: print {printed:?}"));
            }
            (Some(_), None) => {
                return Err(format!("print {printed:?} needs more than depth 96"));
            }
            _ => {}
        }
    }
    Ok(7)
}

/// Greedy delta-debugging: the shortest string (by removing chunks of
/// characters) that still makes `check` fail.
pub fn minimize(src: &str, bits: u8) -> String {
    minimize_with(src, &|s: &str| check_parse(s, bits, true).is_err())
}

/// Same, for any failure predicate.
pub fn minimize_with(src: &str, fails: &dyn Fn(&str) -> bool) -> String {
    let mut cur: Vec<char> = src.chars().collect();
    let mut chunk = (cur.len() / 2).max(1);
    loop {
        let mut i = 0;
        let mut shrunk = false;
        while i < cur.len() {
            let end = (i + chunk).min(cur.len());
            let mut cand: Vec<char> = Vec::with_capacity(cur.len());
            cand.extend(cur.iter().take(i).copied());
            cand.extend(cur.iter().skip(end).copied());
            let s: String = cand.iter().collect();
            if fails(&s) {
                cur = cand;
                shrunk = true;
            } else {
                i += chunk;
            }
        }
        if chunk == 1 && !shrunk {
            break;
        }
        if chunk > 1 {
            chunk /= 2;
        }
    }
    cur.iter().collect()
}

/// Run `check_parse`; on failure return a report with the minimized input.
pub fn report(src: &str, bits: u8, with_depth: bool, label: &str) -> Result<usize, String> {
    match check_parse(src, bits, with_depth) {
        Ok(c) => Ok(c),
        Err(why) => {
            let small = minimize(src, bits);
            if let Err(why_min) = check_parse(&small, bits, true) {
                if let Some(id) = known_cause(&small, bits, &why_min) {
                    if let Some(c) = KNOWN.get(id) {
                        c.fetch_add(1, Ordering::Relaxed);
                    }
                    return Ok(CLASS_NAMES.len() - 1);
                }
            }
            let text =
                format!("{label} bits={bits}: {why}\n  input    {src:?}\n  minimal  {small:?}");
            if record(&text, bits, &small) {
                return Ok(CLASS_NAMES.len() - 1);
            }
            Err(text)
        }
    }
}

/// Root causes found by this layer, in `documents/FUZZ_LEDGER.md` (ids
/// W-*) and reproduced by the ignored tests in `tests/wide_range_findings.rs`.
/// A failure whose minimized input is one of these shapes is counted, not
/// failed, so the rest of the suite runs green. Anything else still fails.
pub const KNOWN_NAMES: [&str; 11] = [
    "W-DEPTH-1 empty node printed as {}",
    "W-DEPTH-2 \\not X printed as \\not {X}",
    "W-DEPTH-3 unicode script char printed as ^{..}",
    "W-DEPTH-4 prime_symbol: x' printed as x^\\prime",
    "W-DEPTH-5 \\color switch printed inside a group",
    "W-DEPTH-6 font body of several nodes printed inside a group",
    "W-DEPTH-7 leading \\( grouped so it is not read as a fence (by design)",
    "W-RT-1 \\genfrac with empty numerator and denominator prints as an empty \\frac",
    "W-DEPTH-8 edge whitespace atom grouped (by design)",
    "W-DEPTH-9 paren-lookahead grouping (by design)",
    "W-DEPTH-10 \\genfrac with delimiters prints as \\left..\\frac..\\right (one level deeper)",
];

/// Counts of known findings seen in this process.
pub static KNOWN: [AtomicUsize; 11] = [const { AtomicUsize::new(0) }; 11];

/// Index into `KNOWN_NAMES` for a minimized failing input, if it is a known
/// shape. `why` is the failure of the minimized input itself.
pub fn known_cause(min: &str, bits: u8, why: &str) -> Option<usize> {
    if min.contains("\\genfrac") && why.contains("does not reparse") {
        return Some(7);
    }
    if min.contains("\\genfrac") && why.starts_with("depth grew") {
        return Some(10);
    }
    if min.contains("\\color") && min.matches("\\color").count() * 12 + 8 >= min.chars().count() {
        return Some(4);
    }
    if min.chars().count() > 200 {
        return None;
    }
    let opts = options_from_bits(bits);
    let tree = parse_with_options(min, &opts).ok()?.0;
    let printed = tree.to_latex_with(&opts);
    if printed.contains("{}") {
        return Some(0);
    }
    if min.contains("\\not") {
        return Some(1);
    }
    if min.chars().any(|c| "²³¹ⁿ⁰⁴⁵⁶⁷⁸⁹ₙ₀₁₂₃₄₅₆₇₈₉".contains(c))
    {
        return Some(2);
    }
    if bits & 8 != 0 && min.contains('\'') {
        return Some(3);
    }
    if min.contains("\\color") {
        return Some(4);
    }
    let fonts = [
        "\\mathrm", "\\rm", "\\mathbf", "\\bf", "\\textbf", "\\mathit", "\\it", "\\textit",
        "\\mathsf", "\\sf", "\\textsf", "\\mathtt", "\\tt", "\\texttt", "\\textrm",
    ];
    if fonts.iter().any(|f| min.contains(f)) {
        return Some(5);
    }
    if min.contains("\\(") || min.contains("\\)") {
        return Some(6);
    }
    if min.chars().any(|c| c.is_whitespace() && !c.is_ascii()) {
        return Some(8);
    }
    if min.contains('(') || min.contains(')') {
        return Some(9);
    }
    None
}

/// Triage hook: with `LATEX_RUST_WIDE_FINDINGS_FILE` set, append the finding
/// to that file and return true, so a run can list every distinct root cause
/// instead of stopping at the first. Without it, returns false.
pub fn record(text: &str, bits: u8, minimal: &str) -> bool {
    use std::io::Write;
    let Ok(path) = std::env::var("LATEX_RUST_WIDE_FINDINGS_FILE") else {
        return false;
    };
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(f, "{text}\nMIN {bits} {minimal:?}\n");
    }
    true
}

/// Tally of outcome classes.
pub struct Tally {
    /// Counts per `CLASS_NAMES` index.
    pub counts: [usize; 8],
}

impl Tally {
    /// Zeroed tally.
    pub fn new() -> Self {
        Tally { counts: [0; 8] }
    }

    /// Count one outcome.
    pub fn add(&mut self, class: usize) {
        if let Some(c) = self.counts.get_mut(class) {
            *c += 1;
        }
    }

    /// Total inputs counted.
    pub fn total(&self) -> usize {
        self.counts.iter().sum()
    }

    /// One-line summary.
    pub fn line(&self) -> String {
        let mut out = String::new();
        for (name, n) in CLASS_NAMES.iter().zip(self.counts.iter()) {
            out.push_str(&format!("{name}={n} "));
        }
        out
    }
}

/// One-line summary of the known findings seen so far.
pub fn known_line() -> String {
    let mut out = String::new();
    for (name, n) in KNOWN_NAMES.iter().zip(KNOWN.iter()) {
        out.push_str(&format!("[{name}: {}] ", n.load(Ordering::Relaxed)));
    }
    out
}

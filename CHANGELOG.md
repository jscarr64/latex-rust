# Changelog

## [3.0.3] — 2026-11-09

### Added

- `gathered` environment (unnumbered centred display rows), array rule
  spacing that preserves the intercolumn gap around vertical rules, and
  ink-safe SVG metrics (viewport contains the ink, real baseline reported).
  Credit: [Titiloye Paul (Ohanronnie)](https://github.com/Ohanronnie),
  whose commits are in the history under his name.

### Fixed

- Array rule dimensions, checked against real TeX (Tectonic + STIX Two Math):
  vertical rules add zero width (TeX draws a rule centred in the gap, so
  ruled − plain is exactly 0); a `||` group adds exactly
  `\doublerulesep` (0.2 em) and a single `|` adds nothing, at the edges as
  well as between columns.
- Horizontal rules add only their own thickness: array rules use TeX's
  `\arrayrulewidth` (0.4 pt = 0.04 em) instead of the font's fraction rule
  thickness (0.068 em), each rule's thickness is split evenly between height
  and depth, adjacent rules are separated by `\doublerulesep` minus one rule
  thickness (0.16 em), and no other padding surrounds a rule. Isolated
  `\hline` now grows an array by exactly 0.04 em and `\hline\hline` by
  exactly 0.24 em, matching TeX.
- `gathered` accepts `\hline`, as TeX does: the box grows by exactly the
  rule thickness (0.04 em), half in height and half in depth. Alignment tabs
  in `gathered` still error, as TeX errors on them.

## [3.0.2] — 2026-11-07

### Fixed

- Treat explicit spacing as content rather than an empty node. Preserve `\,`
  and `\quad` in align, matrix, and cases cells, and allow spacing-only
  content such as `\underline{\quad}`. Empty `\hat{}` remains an error.
- Add nine regression tests covering cell spacing, underline spacing,
  `\frac{\,}{2}`, and the empty-accent guard.
- Keep the existing control-space binomial fuzz seed as a successful
  round-trip: explicit spacing is content; truly empty binomials still error.
- Credit: [acquitelol](https://github.com/acquitelol) (Rosie), whose
  [baac54ba fix](https://github.com/acquitelol/latex-rust/commit/baac54bab18d4305c927d15eb62d8d35e80bc900)
  removes spacing from the empty-node classification.

## [3.0.1] — 2026-10-15

metadata: repository and homepage URLs use lowercase latex-rust; no code changes.

## [3.0.0] — 2026-10-15

Issues found in one program led to a forensic audit of our other crates.
This release lists every finding and its fix. Published to crates.io on 2026-10-15.

### Fixed

- **`\genfrac` took any character as a delimiter.** `\genfrac{}{a}{}{}|a`
  parsed to a right delimiter `a`, printed `\left .\frac {|}{a}\right a`, and
  `\right a` does not parse (found by the 2-hour parse fuzz,
  `roundtrip-genfrac-delim`). `\genfrac` now accepts the delimiters `\left`
  and `\right` accept (one definition, shared) and returns
  `ParseError::Malformed` for any other character or word, and a `{` or `}`
  (from `\{` and `\}`) is the brace delimiter. Code that relied on
  `\genfrac{a}{b}{}{}` as a way to write a letter delimiter now gets an error.

- **A long or mu dimension did not print back to itself.** `\hspace` and
  `\rule` printed their `Dim` through `to_dec_string`, which stops after 24
  decimal digits, so `\rule{2}{.1662322688380342688380341}` (25 digits) reprinted as
  24 digits and read back as a different number, and `\hspace{1mu}` (1/18 em,
  a decimal that never ends) printed `0.055555555555555555555555em` (found by the
  2-hour parse fuzz, `roundtrip-dim-long-a` and `-b`, and by a property over the
  dimension grammar). The printer now writes the exact value in the first unit
  (`em`, `mu`, `pt`) whose text the reader turns back into the same `Dim`:
  `1.0mu`, `0.1662322688380342688380341em`, and a whole number of 38 digits
  without `.0`. The reader accepts exactly what it did before (an overflowing
  number is still `ParseError::Malformed("dimension out of range …")`); the
  unit table is now one definition, `Dim::in_em`. `Dim::to_dec_string` and the
  SVG output are unchanged. The `to_latex` text of a `Dim` that has no exact
  decimal in any of those units (only possible from a tree built by hand) is
  still the 24-digit form.

- **A large `Dim` was not equal to itself.** `Dim::cmp` multiplied the two
  cross products with `checked_mul` and returned `None` (unordered) when that
  overflowed `i128`, so `Dim::parse("2.1662322688380341817")` compared unequal
  to itself and a tree holding it never equalled its own reparse (found by the
  2-hour parse fuzz). `cmp` is now exact for every non-NaN pair, with no
  products: it compares integer parts and then the reciprocals of the
  remainders, so it returns `None` only for NaN, as its doc says.

- **Two more places the parser reads a token by position.** `\substack{{]}}`
  printed `\substack {]}`, and `{]}` reads as a bare `]`, so the reprint lost
  the required `{`; a lone `[` or `]` line now keeps two groups. A cell that
  starts with `\hline` or `\intertext` at the start of an environment row
  (`\begin{matrix}{\hline}x\end{matrix}`) read back as a row rule; such an item
  now prints braced. Found by a random token search after the r15 fuzz round.

- **A `(` in a matrix cell paired with a `)` in the next cell.** The
  scripted-paren lookahead ran past `&`, `\\`, `\end` and `\right`, so
  `\begin{vmatrix}{(}&{)^x}\end{vmatrix}` printed `(&)^x` and the reprint
  failed with "unexpected token &". The lookahead now stops at any token that
  ends the list it started in, the rule that already applied to an option's
  `]`. Parser change: a paren group before `&`, `\\`, `\end` or `\right`
  never looks beyond it for a script. Found by fuzzing.

- **A print ending in a Unicode whitespace atom lost it on reparse.**
  `preprocess` trims every Unicode whitespace character at the ends of the
  input, but the tokenizer reads U+00A0, U+2007 and the like as atoms, so
  `a\u{a0}%` (the comment hides the character from the trim) parsed to two
  atoms and printed as `a\u{a0}`, which parses to one. `to_latex` now puts
  such a print in one `{…}` group. Found by fuzzing.

- **A color defined inside text math stays defined.** The text-math
  sub-parser used to work on a copy of the color table, so
  `\text{$\definecolor{a}{named}{red}$}` dropped `a`. The table is now moved
  into the sub-parser and handed back (`\definecolor` is global in TeX), which
  removes the copy. The same change removes the other token and error copies
  the grouping rewrite had added (no-clone policy): the lib's `.clone(` count
  is back to its 292 lines.

- **Nested optional arguments reprinted exponentially slowly and too deep.**
  `\xleftarrow[\xleftarrow[…` printed each option body twice per level, and
  a `]` that closed a nested option command forced a needless group, so the
  reprint nested twice as deep as the source and failed at the depth limit.
  The printer now prints an option body once and asks the parser's own
  option reader whether the body ends early; an empty option prints as `[]`.
  Row paren handling and text math bodies also print once. Found by fuzzing.

- **A long prime run could not be reprinted.** Each extra `'` nested the
  superscript one row deeper, and `to_latex` braced every level, so `{[` with
  34 primes parsed but its reprint failed with "input nests deeper than 32
  levels". A prime run is now one flat row (`f'''` is three sibling primes,
  with or without `prime_symbol`), and `to_latex` prints a run of two or more
  primes unbraced inside one superscript group: `{f}^{′′′}`, or
  `{f}^{\prime \prime \prime }` for `\prime`. Both reparse to the same tree
  under either option, and the reprint costs a fixed two levels over the
  source instead of one more level per prime.

  Spec change: `gold()` of three or more primes changes from nested
  `(row (row …) …)` to one `(row …)`. The lexicon gold
  `prime-three-stays-nested` is now `prime-three-flat` and forbids the nested
  shape. The golds in `tests/binding.rs`, `golds/accumath.toml`, and
  `golds/lexicon.toml` change to the flat row; this reverses the earlier
  lexicon rule that `f'''` stays nested. Callers that walked the nested rows
  should iterate the row.
  `to_latex` of `f'''` changes from `{f}^{{{′}{′}}{′}}` to `{f}^{′′′}`.

- **Infix `\over` and `\choose` matched a prefix.** `convert_over_fractions`
  searched for the bytes `\over`, and `convert_choose` searched for `\choose`,
  with no control-word boundary. A `{` in front of `\overline`, `\overset`,
  `\overrightarrow`, `\overleftarrow`, `\overleftrightarrow`, or `\overbrace`
  became a fraction whose denominator was the rest of the command name.
  `\frac{\overline{x}}{2}` parsed as a fraction of an empty numerator over the
  letters `l i n e` and `x`. The same shape of bug was waiting in `\choose`.
  Both rewrites now accept a command only when the next character is not a
  letter. `\over` still runs before `\choose`, so a group that contains both
  keeps the previous order.

  Migration: `{a \over b}` is still `\frac{a}{b}`, and `{n \choose k}` is still
  `\binom{n}{k}`. Bare `\overline{x}` is still an accent. A string that used to
  parse as that broken fraction now parses as the accent or the `\overset`
  node. Stored `gold()` values for an `\over…` command that sat inside braces
  change. Callers that matched `Fraction` there should match `Accent` or
  `OverUnder`.

- **Repeated `{a \over b}` was quadratic.** Each replacement started the search
  again at the beginning of the string. 100,000 copies took 14.2 s in a
  release build. Each command is now rewritten in one left-to-right pass.
  The trees of real infix fractions are unchanged. No migration.

- **A script after `(...)` bound to one parenthesis.** `(-b)^{n}`, `(a/b)^{n}`,
  `(\frac{a}{b})^{n}`, `(2x)^{2}`, and `(f(x))'` put `^`, `_`, or `'` on the
  closing parenthesis. `x^(-b)` put the opening parenthesis in the exponent.
  A parenthesized row is now one nucleus, and the parentheses stay atoms in
  that row.

  Migration: a superscript, subscript, or prime after `(...)` applies to the
  parenthesized row. The old tree applied it to the nearest parenthesis.
  `{(-b)}^{n}` and `\left(-b\right)^{n}` stay as they were. `-b^{n}` and
  `a/b^{n}` still put the power on `b`.

- **`f'^2` flattened the prime and the exponent.** The superscript was a row
  of prime-then-2. It is now the square of primed `f`, the same tree as
  `{f'}^2`. `f'`, `f''`, and `f'''` are unchanged. `f^{\prime}` stays a
  `prime` symbol.

  Migration: `f'^2` is the square of primed `f`. `f'` and `f^{\prime}` stay
  different nodes.

- **A raised depth limit aborted the process, and acceptance depended on the build.**
  `with_max_depth(20_000)` parsed 3,500 brace groups and aborted at 3,531.
  The same limit aborted on deep `\left` and `\frac`. A stack budget on top
  of that returned `Malformed` for input that was still under the caller's
  limit, and a release build's smaller frames made 3,531 braces `Ok` while
  a debug build returned `Err`. Nesting now lives on a heap stack.
  `max_depth` is the only limit: the same input gets the same result in
  debug and release, on an 8 MiB stack and on a 256 KiB stack.

  Migration: the default depth stays 32. 3,500 and 3,531 brace groups at
  limit 20,000 return `Ok`, and so does the same nest of `\left` or a
  3,531-deep `\frac`. A brace group or a `\left` body is one level, so
  20,000 of them at that limit return `Err(Malformed)`. A braced argument
  such as `\frac` or a superscript costs two levels, so 10,000 of those
  return `Err(Malformed)`. Callers on the default limit of 32 see no change.
  `gold`, `to_latex`, layout, and SVG/PNG/egui render walk a heap stack, and
  so do `MathNode` / `MathBox` clone and drop. A tree the parser accepts can
  take those paths on a 256 KiB thread.

- **Empty input parsed as an empty row.** `parse("")` and whitespace returned
  `(row)`. `\frac{}{}` was a fraction of two empty rows. A bare prime was a
  superscript on an empty row. `^2` and `^{2}` were a superscript on that
  empty row, and `_i` was a subscript on it. `'^2` is a prime with no base.
  A bare `^` was already `Malformed`. `x^2` and `x_i` are unchanged.

  Migration: callers that treated the empty row as “no formula” should match
  on `Err`. `\frac{}{}`, a bare prime, `^2`, `^{2}`, `_i`, and `'^2` return
  `Err`. A script still needs a real base: `x^2` is the same tree.

- **`√`, a superscript run, and U+2212 were ordinary atoms.** `√x` and
  `√{x}` are a square root, the same tree as `\sqrt{x}`. A run of
  superscript digits, superscript letters, and a superscript minus
  (`²`, `³`, `ⁿ`, `⁻¹`, `²³ⁿ⁻¹`) is a superscript, the same tree as
  `x^{…}`. A superscript with no base is `Malformed`, the same as `^2`.
  U+2212 MINUS is a binary minus, the same atom as `-`.

  Migration: stored `gold()` values for those characters change. `√x^2`
  is the square of the radical, and `√{x^2}` is the radical of the square.
  `f'²` is the square of primed `f`, the same tree as `f'^2`. ASCII `-`
  is unchanged.

- **ASCII `<=`, `>=`, and `!=` were two atoms.** They stay two atoms
  unless requested. `ParseOptions::with_ascii_relations` maps adjacent
  `<=`, `>=`, and `!=` to the same symbols as `\le`, `\ge`, and `\ne`.
  A space still splits them, so `x < = 0` is two relation atoms.

  Migration: the flag defaults to off, so existing `gold()` values stay
  byte-identical. Accumath turns it on at `parse_with_options`.

- **Bare words were products of letters.** `sin(x)`, `zeta(s,a)`, and
  `otherwise` inside `cases` stay letters unless requested.
  `with_bare_operators` reads a maximal run of ASCII letters as one
  identifier. The whole run has to be in the operator table, or be
  `\zeta`, so `sinh` is `\sinh`, `xsin` and `sinx` stay letters, and
  `infty` is not `\inf`. A space ends the run, so `x sin` is `x` times
  `\sin`. `ω` is not an ASCII letter, so `sinω` is `\sin` followed by
  `ω`. Inside `cases`, the identifier `otherwise` is the text node of
  `\text{otherwise}`. Outside `cases` that word stays letters.

  Migration: the flag defaults to off. Existing `gold()` values stay
  byte-identical. Accumath turns it on at `parse_with_options`.

- **The same glyph could be a different node.** `α`, `Γ`, `π`, `≤`,
  `≥`, `≠`, `∞`, `×`, `·` (U+00B7), and `⋅` (U+22C5 DOT OPERATOR) stay
  ordinary atoms unless requested. `⋅` was Ord, not Bin and not
  `(symbol cdot)`. `with_unicode_symbols` maps those characters to the
  symbol nodes of `\alpha`, `\Gamma`, `\pi`, `\leq`, `\geq`, `\neq`,
  `\infty`, `\times`, and `\cdot`. `f'` stays a superscript of U+2032.
  `with_prime_symbol` spells `'` as `(symbol prime)`. `f''` is a row of
  two primes. `f'''` is one flat row of three primes (see the prime-run
  entry above).

  Migration: both flags default to off, so `f'` and `f^{\prime}` stay
  different nodes unless `prime_symbol` is on. `ω` stays an ordinary
  atom. Accumath turns the flags on at `parse_with_options`.

- **A nested color reprinted without its group.**
  `{\color{red} x + {\color{blue} y} + z}` already parsed as red over
  the row and blue only on `y`. `MathNode::to_latex` prints each
  `Color` in its own group, so the inner color does not swallow `+ z`
  on the next parse.

  Migration: the parse tree is unchanged. Callers that printed the tree
  themselves need the same grouping. `to_latex` is the reprint that
  round-trips.

- **`MathNode` and `ParseError` are `non_exhaustive`.** A later release
  can add a variant. `ParseOptions` already was.

  Migration: a downstream `match` that lists every variant of `MathNode`
  or `ParseError` needs a wildcard. `if let`, `matches!`, and
  constructing a variant are unchanged. `ParseOptions::new` still builds
  the defaults, including the four flags above set to off.

### Added

- `MathNode::to_latex` reprints a tree as LaTeX. It is total. Every
  script, accent, fraction, and `\overset` / `\underset` argument is
  braced, an alphabetic control word is followed by a space, and
  `\overline` is the full command. A `Fraction` prints as the command in
  its `FracStyle`. `\over` is stored as `FracStyle::Frac`, so it prints
  `\frac`. Braces and spaces may differ from the source; compare trees,
  then compare a reprint to a reprint.

  Migration: new method. `\frac` gold strings are unchanged. A styled
  fraction's gold spelling is in the `FracStyle` note below.

- `tests/roundtrip.rs` checks `parse(to_latex(t)) == t` and an idempotent
  reprint on the `parse.toml`, `accumath.toml`, and `lexicon.toml` golds,
  including trees built with the flags on. A grammar generator covers
  2,000 trees in debug and 50,000 in release. `LATEX_RUST_ROUNDTRIP_CASES`
  overrides the count. `proptest` is a dev-dependency.
- `fuzz/fuzz_targets/parse.rs` and `preprocess.rs`. The parse target
  keeps `max_depth` at the default, because an abort kills the fuzzer.
- `.github/workflows/ci.yml` runs fmt, clippy (`--all-targets
  --all-features -D warnings`), `cargo test`, `cargo test --release`,
  and a 60-second parse fuzz. A separate job resolves the lockfile with
  rustc 1.99 and runs `cargo audit --deny warnings` with cargo-audit
  0.22.2 on that same toolchain. `.github/workflows/fuzz-nightly.yml` runs the release
  tests, then parse for 16 minutes and preprocess for 2.

### Fixed

- **`\dfrac`, `\tfrac`, and `\cfrac` kept their style.** The tree is
  `Fraction(FracStyle, numerator, denominator)`. `gold()` spells them
  `(dfrac …)`, `(tfrac …)`, and `(cfrac …)`. `\frac` and `{a \over b}`
  stay `(frac …)`, byte-identical to the previous gold. `to_latex`
  prints `\dfrac `, `\tfrac `, `\cfrac `, or `\frac `. Layout forces
  display style for `\dfrac` and `\cfrac`, and text style for `\tfrac`.
  `\dbinom` and `\tbinom` stay `\binom`.

  Migration: a match on `Fraction(num, den)` becomes
  `Fraction(style, num, den)`. `\over` still prints `\frac`. Stored
  `gold()` for `\frac` is unchanged. Stored `gold()` for a styled
  fraction changes from `(frac …)` to the styled spelling.

- **A stray `\\` names itself.** Outside a row environment,
  `ParseError::RowBreakOutside` displays
  `row break \\ outside an environment`. `\cr` uses the same error.
  Inside `matrix`, `aligned`, and `\substack`, `\\` and `\cr` are still
  row breaks.

  Migration: callers that matched `ParseError::Unknown` for `\\` should
  match `RowBreakOutside`. The message is no longer
  `unknown command: \\`.

- **RUSTSEC-2026-0192, direct dependency.** `ttf-parser` has no patched
  release. This crate no longer depends on it. Outlines and `hmtx` /
  `cmap` / `GSUB` come from `skrifa` 0.39. The MATH table is read here,
  because `read-fonts` 0.36 does not expose MATH. `skrifa` stays at
  0.39. `MathFont::face` and the `ttf_parser` re-export are gone.
  `MathParams` carries the
  script-placement and stack constants layout already used. PNG and
  egui golds are unchanged.

  Migration: delete `ttf_parser` from downstream `use` lists. Replace
  `font.face()` with `glyph`, `glyph_id`, `glyph_count`,
  `italic_correction`, `top_accent_attachment`, `horizontal_variants`,
  `vertical_variants`, and `horizontal_assembly_parts`. A match that
  builds `MathParams` by field name needs the new fields.

- **A tree the parser accepts does not overflow the caller stack.**
  `gold`, `to_latex`, layout, SVG, PNG, and egui emission, and clone
  and drop of `MathNode` and `MathBox`, use an explicit heap stack.

  Migration: `layout_with_max_depth` is a nesting limit, not a stack
  budget. 3,531 brace groups and 9,999 `\frac` at limit 20,000 can be
  printed, laid out, and rendered to SVG on a 256 KiB thread, in debug
  and release.

- **A trailing control space was a stray backslash.** `preprocess` used
  `str::trim`. That deleted the space in `\ ` and in the reprint of
  `\ %comment`, and the tokenizer then reported `trailing backslash`.
  The space after an odd run of backslashes is the control space, so
  trimming keeps it. `to_latex` still prints `\ `, and that reprint
  parses again. An even run (`\\ `) is a control symbol plus an ordinary
  space, and that space is still trimmed. The fuzz seed is
  `fuzz/corpus/parse/control-space-comment` (`\ %@QJXi`).

  Migration: `\ ` at the end of input, and a control space before a
  comment, parse as `(space control)`. The end-of-input form used to be
  `trailing backslash`. The comment form parsed once and failed when the
  reprint was parsed again.

- **rustc 1.99 clippy.** The radical and delimiter variant tests use
  `is_none_or` instead of `map_or(true, …)`. SHA-256 walks blocks with
  `as_chunks` instead of `chunks_exact`. No parse or layout change.

- **Fuzz round-trip crashes: 33 crashes, 8 root causes (G1–G8).** The
  `parse` fuzz target checks parse → `to_latex` → parse. Each root cause has
  a test in `tests/fuzz_roundtrip.rs` and minimized seeds in
  `fuzz/corpus/parse/roundtrip-g1-*` through `roundtrip-g8-*` (33 seed files).

  - G1: `to_latex` escapes `$`, `&`, `_`, and `^` in text, as it already did
    `\`, `{`, `}`, and `%`. Migration: none; the reprint changes, the tree
    does not.
  - G2: a control symbol of whitespace in text (`\` before a newline, tab,
    or CR) is a space, as in TeX. Migration: `\text{a\<newline>b}` now holds
    `a b`; the text used to contain the raw newline.
  - G3: a leading space, `[`, or `]` in printed group text is escaped
    (`\text{\ (}`), so the reprint is not unbraced by the `{ )}` / `{[}`
    preprocessor rewrite. Migration: none for the tree; reprinted text can
    start with `\ `, `\[`, or `\]`.
  - G4: styled text that is empty or not all ASCII letters prints as
    `\mathX{\text{…}}` (for example `{\it\text{}}`, `\mathbb{\text{x y}}`),
    so it reparses as text. All-letter styled text keeps `\mathX{…}`.
    Migration: compare trees, not strings; that reprint is longer.
  - G5: a binomial with both sides empty (`\binom{}{}`, `{\choose}`) is
    `Err(Malformed("empty binomial"))`, like `\frac{}{}`. One empty side
    still parses. Migration: callers that rendered an empty binomial must
    supply at least one side or handle `Err`.
  - G6: vertical tab (U+000B) and form feed (U+000C) are whitespace in the
    tokenizer, like tab and newline. Migration: those characters no longer
    produce an atom; stored `gold()` that contained one changes.
  - G7: the `\over` / `\choose` preprocessor copies a control symbol whole,
    so `\\over` and an escaped `\{` no longer start an infix rewrite.
    `\over` or `\choose` inside text (`\text{…\over…}`, `\mbox{…\choose…}`)
    is rejected with `Err`, as `\text{\over}` already was. Migration:
    `{a \over b}` in math is unchanged; move a fraction out of the text.
  - G8: the `{ (}`, `{ )}`, `{ ( }`, `{ ) }`, `{[}`, and `{]}` preprocessor
    rewrites skip a brace escaped as `\{`. `\text{\{\ )}` (text `{ )`)
    printed as `\text {\{ )}`, the rewrite turned `{ )}` into `)`, and the
    reprint lost its closing brace. Migration: none for valid input; an
    escaped `\{` before ` )`, ` (`, `[`, or `]` stays a literal brace.
  - G9: the `(` lookahead that decides whether `(...)` takes a script
    counted a `)` inside a brace group. `\)` in text decoded to a bare `)`,
    so `(\text{\)'})` reprinted as `(\text {)'}`, which the lookahead read
    as `(…)'` and then rejected. The lookahead now skips brace groups, and
    `\)` in text is an error (see "Text mode is a real lexer" below). Seed
    `roundtrip-g9-1410dba0`. Migration: `({a)'}` and `(\mathrm{a)'})`,
    rejected before, now parse.

- **A reprint could nest deeper than its source.** `to_latex` braced every
  script base and argument, every fraction argument and every row item, and
  printed an empty numerator as `{{}}`. A source near the 32-level limit
  parsed, but its reprint was rejected with "input nests deeper than 32
  levels". Fuzzing found 15 such inputs in three families: `{\over\over…}`
  chains, `{\choose\choose…}` chains, and scripts over rows of relations
  (`lo_{==_{…}<}`). The printer now spends a brace only where the parser
  needs one (see "Brace economy" under Changed), and a test checks
  `depth(print(parse(s))) <= depth(s)` for the 15 minimized inputs (seeds
  `roundtrip-depth-*`) and for a generated grammar. Migration: none for
  trees; `to_latex` output changes as described below.

- **Dimensions with no number.** `\rule{2}{.}` and `\hspace{1.2.3em}` parsed to
  a NaN dimension that printed as `NaNem` and did not read back (fuzz,
  `roundtrip-dim-dot`). A dimension now needs at least one digit and at most
  one point; anything else is `ParseError::Malformed("invalid dimension …")`. A number too
  large for the exact rational (`\rule{2}{888…8}`, 39 digits) is also NaN, and is
  now `ParseError::Malformed("dimension out of range …")` (fuzz
  `roundtrip-dim-overflow`).

- **Reprints that did not parse back.** A whole-grammar property
  (`print(parse(print(x))) == print(x)` and `parse(print(t)) == t`, over every
  `MathNode` kind, with each parser option) and fuzzing found twelve more. Each
  is a seed (`roundtrip-canon-*`) and a table row:
  - a `]` in an optional argument ended it early: `\xrightarrow[\mathrm{]}]`
    printed `\xrightarrow []]`. Such an argument is now grouped (`\sqrt[…]`
    too);
  - a lone `[` as a `\sqrt` body printed `\sqrt {[}`, and `{[}` is read as a
    bare `[` that opens an index. It now keeps two groups;
  - a print starting with `\(` (the symbol, as in `{\( }x`) was read as an
    unclosed math fence. Such a print is now one group;
  - a `\)` symbol inside text math (`\text{$\)$}`) closed the `\(` early. That
    body now prints between `$…$`;
  - eight that the new printer briefly had before release: a primed base
    before more primes (`{x_x'}'`); `'` marks read back as `\prime` under
    `prime_symbol` (see `to_latex_with` below); and a primed `)` after an
    unclosed `(` (`(\not()'^x`, `(^))'^)`), whose `'` made the reparse read
    `(…)'` as a scripted paren group. That item is now grouped; likewise a
    primed `(` base (`('^))^\ `) now counts as an open paren;
    and a `(` or `)` in an optional argument (`(^)\xleftarrow[)^x]]`), which
    the paren lookahead of an earlier `(` sees. Such an option is now
    grouped, and a lone `(` or `)` keeps two groups; and text math whose
    body starts with `\(` (`\text{$\(\)$}`), which the `\)` check saw as one
    group but the printer wrote without it.

### Changed

- **Parser: a braced command argument costs no extra depth** (spec 5,
  Q2). `\frac{a}{b}` used to cost two levels per argument (the argument and
  its group), while `\frac ab` cost one. Both now cost one, so the printer can
  always brace arguments without nesting deeper than the source.
  `ParseOptions::with_max_depth` therefore accepts more braced inputs: at the
  default limit, 31 nested `\frac{1}{…}` now parse (before, 15).

- **Parser: the paren-script lookahead skips delimiters and options**
  (spec 5, Q1). When a `(` starts an atom, the parser looks for its matching
  `)` followed by `^`, `_` or `'` to read `(…)^x` as one scripted group. That
  scan now skips the delimiter after `\left`, `\right`, `\middle` and the
  `\big` family, and the `[…]` option of `\sqrt`, `\xrightarrow`,
  `\xleftarrow`, `\color`, `\textcolor`, `\colorbox` and `\fcolorbox`. Those
  parens belong to their command. Effect: `(\big( a)^2` is now one scripted
  group (before, the `\big(` hid the match), and `(\big)^2` and
  `(\sqrt[)^2]{x}` now parse (before, they were errors). An option is a unit
  from inside too: a `(` inside `[…]` cannot pair with a `)` after the `]`
  (`\xleftarrow[(]{)})^2` used to fail). Migration: inputs
  where a `(` paired with a delimiter or an option paren now parse
  differently. That pairing was never intended.

- **Text mode is a real lexer (documents/TEXT_MODE_SPEC.md).** `\text`,
  `\mbox` and `\intertext` bodies are kept whole by the tokenizer
  (`Token::TextArg`) and lexed by a text-mode grammar. No math pass and no
  string rewrite reads them any more. `preprocess` now only trims; its
  `{ (}`, `{[}`, `{\rm …}`, `\over`, `\choose` and `\mbox` rewrites are token
  rules or command aliases, so they cannot match inside text or escapes.
  Spec changes, each with its typed error:
  - `\\` in text is `Err(Malformed("line break in text"))`. Write
    `\textbackslash` for a backslash.
  - `\text{$x$}` and `\text{\(x\)}` are accepted: inline math in text is a
    math sub-tree. A stray `\(`, `\)` or `$` in text is
    `Err(Malformed("unmatched math shift in text"))`.
  - Unknown control words and symbols in text (`\foo`, `\rm`, `\over`, `\,`,
    `\^`, `\[`) are `Err(Unknown("\\name"))`. Accepted: `\text`, `\mbox`,
    `\textrm`, `\textbf`, `\textit`, `\textsf`, `\texttt` (nested runs),
    `\ldots`, `\dots`, `\quad`, `\qquad`, `\textbackslash`,
    `\textasciicircum`, `\textasciitilde`, and the escapes
    `\{ \} \$ \% \& \# \_`.
  - Bare `^ _ & #` in text are `Err(Malformed("unexpected token X in text"))`.
    `~` is a no-break space.
  - Text nesting counts toward `max_depth`, and is also capped at 32 levels
    (`"text nests deeper than 32 levels"`) when `max_depth` is raised, to
    bound the text → inline math → text recursion.
  - `\its` and `\rmfoo` are whole control words: `{\its …}` is no longer
    read as `{\it s…}`.
  - Because of that, two G4 inputs in `tests/fuzz_roundtrip.rs` that started
    with `{\its` moved from the round-trip list to the rejections:
    they now return `Err(Unknown("\\its"))` and `Err(Unknown("\\itsM"))`.

  Silent wrong results this fixes:
  - `\text{{ )}}` was the text `)`; it is ` )`.
  - `\text{{\rm a}}` was the text `\mathrm{a}`; it is an error.
  - `\text{{a \over b}}` was the text `\frac{a}{b}`; it is an error.
  - `\text{\mbox{x}}` was the text `\text{x}`; it is a nested run.
  - `\text{a⇥b}` (tab, newline) was `ab`; it is `a b`.
  - `\text{a\foo b}` was the text `a\foob`; it is an error.
  - `\text{a\\b}` was the text `a\b`; it is an error.
  - `\text{{a}}` and `\text{\{a\}}` were the same text; braces are now
    grouping and `\{ \}` are the characters.
  - `\text{a\ b}` and `\text{a b}` were the same; the control space is kept.

  AST change: `MathNode::Text(String, TextStyle)` is now
  `MathNode::Text(TextRun, TextStyle)`. A `TextRun` is a list of `TextItem`s
  (`Str`, `Space`, `ControlSpace`, `NoBreakSpace`, `Symbol`, `Nested`,
  `Math`). `TextRun::plain(s)` builds one from a string,
  `TextRun::to_plain_string()` flattens it, and `as_str()` returns the text
  of a single-`Str` run. `Token` has a new variant `TextArg(String)`.
  Migration: match `MathNode::Text(run, style)` and call
  `run.to_plain_string()` where a `String` was used. `gold()` of a run that is
  only characters and spaces is unchanged; other runs print as
  `(text <style> (run …))`.

  `to_latex` of text: `\textbackslash{}`, `\textasciicircum{}` and
  `\textasciitilde{}` for those characters, `\(…\)` for inline math, and no
  more leading `\ `, `\[`, `\]` escapes (G3), since a text body is opaque.

- **Grouping in `to_latex` (documents/TEXT_MODE_SPEC.md section 5).** A
  reprint never nests deeper than any source of the same tree, and braces
  come from one rule based on the parser's own grouping rules, not from
  per-node patches. Command arguments always print braced (`\frac {1}{2}`,
  `\sqrt {x}`, `\hat {x}`, `\mathbf {x}`). Script arguments print bare
  when they are one nucleus (`x^2`, `x_a`, `x^\alpha`), and otherwise as
  one group. An empty argument is `{}`. Row items print bare unless that
  would change the tree: a nested row, a superscript character, or an item
  whose `)` the parser's paren lookahead would pair with an earlier `(`.
  Two bare letters are split by a space, so `bare_operators` does not join
  them. `\binom` replaces `\left(\frac…\right)` for that tree, and
  superscript primes print as `'` marks (`x'`, `x_a'`, `x^a''`).

  The parser still reads bare command arguments (`\frac12`, which TeX and
  KaTeX also accept, and `\frac 1\frac 12`, which TeX does not read as one
  argument). The printer no longer produces the second form.

  Migration: compare trees, not strings. Examples of the new output:
  `{x}^{2}` is `x^2`, `{x}^{′′}` is `x''`, `{a}{b}` is `a b`,
  `\left (\frac {n}{k}\right )` is `\binom {n}{k}`. `\frac {1}{2}` is
  unchanged from 2.x.

  API addition: `MathNode::to_latex_with(&ParseOptions)`. With
  `prime_symbol` on, `'` reads back as `\prime`, so this keeps a literal `′`
  superscript as `^{′}`. `to_latex()` is `to_latex_with(&ParseOptions::new())`.

- **MSRV is 1.99.** `rust-version` is `"1.99"`. `rust-toolchain.toml`
  pins channel `1.99` with `clippy` and `rustfmt`. The CI test job, the
  lockfile step, the audit job, and the release-test step of the nightly
  fuzz workflow use rustc 1.99. The fuzz compile stays on nightly.

  Migration: a toolchain older than 1.99 cannot build this crate. This
  is a breaking change.

- **Optional `egui` is 0.36.2, with `epaint` 0.36.2.** The feature stays
  off by default, so a default build does not link either crate.
  `epaint` 0.34 switched its font backend from `ab_glyph` / `ttf-parser`
  to `skrifa`. 0.36.2 keeps that, so RUSTSEC-2026-0192 (`ttf-parser`
  0.25.1, unmaintained, no patched release) is gone from the
  `--all-features` tree. `cargo audit --deny warnings` has no ignores.
  MSRV 1.99 is above `egui` 0.36's rustc 1.95.

  Glyph meshes are still tessellated in this crate from `skrifa`
  outlines. `epaint` 0.35 shapes its own text with `harfrust` and types
  text cursors as `CharIndex` and `ByteIndex`. This renderer does not
  call that shaper, so those types are not on the emission path.
  `clip_rect_margin` and the renamed `Panel` methods are not used.
  Call sites pass `f32`, `Color32`, `Stroke`, and `CornerRadius`
  directly, which is what remains after `impl Into<f32>` arguments were
  removed. `Rounding` is `CornerRadius`. A frame stroke uses
  `StrokeKind::Middle` because the rectangle is already inset by half
  the stroke width.

  Migration: enable `features = ["egui"]` and depend on `egui` 0.36.2.
  `MathFont::face` is already gone. Downstream code that named `egui`
  0.28 types (`Rounding`, `ab_glyph`) has to move to 0.36.2.

### Known issues

None.

## [2.1.1] — 2026-10-06

The dvips color names now carry their license, and `TealBlue` matches
`drivers.dtx`. No API changes.

### Fixed

- `TealBlue` is the upstream dvipsnames CMYK `0.86, 0, 0.34, 0.02`. The
  table shipped `0.86, 0.09, 0.54, 0.10`, which is not in `drivers.dtx`
  and has no other documented source. A formula that names `TealBlue`
  therefore changes color. The other 67 tuples were already the upstream
  values.

### Added

- `data/dvipsnames.tsv` identifies itself as a modified form of
  `dvipsnam.def` from the `%<*dvipsnames>` section of `drivers.dtx`
  (Copyright 1994 David Carlisle and Sebastian Rahtz; 1995-1999 David
  Carlisle; 2000-2026 The LaTeX Project; LPPL-1.3c or later). The
  modifications are the tab-separated layout and the alphabetical
  re-sort. `NOTICE` records the same terms, including where to obtain
  an unmodified copy of the graphics bundle.
- `tests/dvipsnames_upstream.rs` checks all 68 CMYK tuples against the
  values transcribed from `drivers.dtx`.

### Known issues

README and CHANGELOG had no Known issues section. GitHub issues #1
through #7 and #11 are already fixed in 2.0.0, 2.0.1, and 2.1.0.

`ttf-parser` 0.25 is flagged unmaintained (RUSTSEC-2026-0192, informational,
no patched release). It stays in 2.1.1. `MathFont::face` returns
`ttf_parser::Face`, and the crate re-exports `ttf_parser` so embedders
share that version (issue #7). Layout reads the OpenType MATH table,
glyph assemblies, and GSUB `ssty` through that face, and the SVG, PNG,
and egui backends consume its outlines. RustSec points at `skrifa`.
`read-fonts` merged MATH table support on 2026-09-08 (fontations pull
request 2105), but those types are not a stand-in for `ttf_parser::Face`,
and no maintained API-compatible fork is published. Replacing the parser
would be a breaking change to the public face API and a rewrite of the
metric and outline paths. That does not belong in a patch. The allowed
font stack in CONTRIBUTING.md is still `ttf-parser`.

## [2.1.0] — 2026-10-02

Layout follows more of TeX and the OpenType MATH table. Dimensions of fractions,
delimiters, radicals, scripts, accents, large operators, integrals, and AMSMath
grids change. `layout_with_em_size_pt` and
`layout_with_numbering_and_em_size_pt` take the physical root em so absolute
TeX lengths stay stable when the render size changes. SVG, PNG, and egui pass
their existing font size (12 pt, 12 pt, and 14 pt). `layout` still assumes 10 pt.

### Fixed

- Delimiter-less fractions are as wide as the numerator, the denominator, and
  the rule. Automatic delimiters use TeX's delimiter factor and a physical 5 pt
  `\delimitershortfall`, and sit on the math axis.
- Radical variant slack goes into the rule gap and the descent.
  `radicalDegreeBottomRaisePercent` raises the bottom of the degree.
- Display large operators sit on the math axis. Upper and lower limits each
  obey their own baseline and gap minima.
- Script style uses OpenType `ssty` alternates. `SpaceAfterScript` is taken at
  the parent style. A row adds math italic correction to a bare variable, and
  does not add it again when that variable's scripts already include it.
- Wide hats and tildes use the smallest MATH horizontal variant that covers the
  nucleus, placed with `TopAccentAttachment`.
- Matrices, cases, and aligned environments use their TeX styles and physical
  column spacing. Row rhythm uses LaTeX's 12 pt `\baselineskip`. `aligned` adds
  `\jot` the way amsmath's `\openup` does. `cases` uses `\arraystretch` 1.2 and
  a `\quad` between columns.
- A display integral with side scripts is centered on the math axis. Its italic
  correction shifts the superscript and not the subscript.
- Exact-rational decimal formatting no longer overflows while printing a
  remainder. egui tessellation rejects vertex indexes that do not fit in a
  `u32`.

### Thanks

baselogic (pull request 13) wrote the layout corrections and the regression
tests for delimiter sizing, fraction width, radical variant slack, large-operator
limits, script placement, row italic correction, AMSMath column spacing,
display-integral side scripts, and both overflow checks.

## [2.0.1] — 2026-09-27

A fix for the egui backend. No API changes; upgrade with
`cargo update -p latex-rust`.

### Fixed

- The egui backend renders every glyph. Before, formulas using `H`, math italic
  `t` (so `\partial_t u`), `π`, `σ`, `#`, `$`, `€`, `\dashv`, `\natural`, bold
  and blackboard-bold capitals and about 260 other glyphs failed with
  `Error::Unsupported { what: "glyph tessellation" }`. `B`, `k`, `τ`, `↦` and
  about 140 other glyphs were drawn with a stray triangle outside the glyph.
  The ear clipper that triangulates glyph outlines accepted ears whose closing
  diagonal ran through another vertex. Its result is now checked exactly, and
  outlines it cannot triangulate correctly are triangulated by exact scanline
  trapezoids under the non-zero rule, as the SVG and PNG backends fill them.
  Glyphs that were already drawn correctly get the same meshes as before.

## [2.0.0] — 2026-09-27

This release fixes the eight defects Tom Clark reported while integrating
latex-rust into IronLAB (issues #1–#7 and #11). Several of them change what the
renderer draws, so many golds were re-recorded, each in its own commit next to
the fix that moved it.

### Breaking changes

- `BoxContent::Glyph` has a new `scale: Dim` field, the factor the layout engine
  applied to the glyph's metrics. The variant is now `#[non_exhaustive]`, so
  match it with `BoxContent::Glyph { ch, glyph_id, .. }` and build it with the
  new `BoxContent::glyph(ch, glyph_id, scale)`. (#1)
- Parsing and layout now return `Err` for input nested deeper than
  `DEFAULT_MAX_NESTING_DEPTH` (32). Before, such input overflowed the stack and
  aborted the process. (#6)
- Output changes: variable letters are drawn in math italic, math-mode `-` is
  drawn as U+2212, script glyphs are drawn smaller, and diacritic accents sit
  lower. Code that pinned the old dimensions or images will see new values.

### Fixed

- Superscripts, subscripts and limits are drawn at script size in the SVG, PNG
  and egui renderers. Before, they were laid out at script size but drawn at
  full size. (#1)
- Unstyled Latin letters and lowercase Greek letters are set in math italic, as
  TeX does. Digits, uppercase Greek, `\mathrm`, `\text` and operator names stay
  upright. (#2)
- A math-mode `-` is drawn as U+2212 MINUS SIGN instead of the hyphen. It keeps
  the `Bin` class, and `-` inside `\text{...}` is still a hyphen. (#3)
- Font switches such as `\mathrm` reach nested subformulae (script bases,
  fractions, radicals). (#4)
- Diacritic accents (`\hat`, `\dot`, `\bar` and the rest) are placed by the TeX
  rule, lowering them by the font's accent base height. (#5)
- Deeply nested input returns `Err` instead of aborting the process. (#6)
- `STIX_TWO_MATH_OTF` is a `static`, so the font is linked into a binary once
  rather than once per use. (#11)

### Added

- `BoxContent::glyph(ch, glyph_id, scale)`. (#1)
- `Dim::as_ratio()` returns a `Dim`'s exact value as a numerator and
  denominator. (#7)
- `MathFont::face()` exposes the parsed OpenType face, and the crate re-exports
  `ttf_parser` so consumers can name its types. (#7)
- `ParseOptions`, `parse_with_options`, `layout_with_max_depth` and
  `DEFAULT_MAX_NESTING_DEPTH` let embedders choose the nesting limit. (#6)
- `tests/script_scale.rs` checks script glyph size in every backend, and new
  golds pin `a-b` to U+2212. (#1, #3)

### Thanks

Tom Clark (IronLAB) reported all eight issues with reproductions and proposed
fixes, and contributed the pull requests for #4, #7 and #11 and the first depth
limit for #6.

## [1.0.4] — 2026-09-20

Clippy debt clear (`-D warnings`): `RowKind::Intertext` boxed to shrink enum size.

## [1.0.3] — 2026-09-19

Coordinated patch with zenith-float, hdf5-rust, and redb-view (pure-Rust FOSS family adjacent to Accumath; Accumath itself stays proprietary).

## [1.0.2] — 2026-09-04

Standalone layout math. This crate does not depend on a numeric library.

- `Dim` is an exact rational (`num/den`). No hardware `f32`/`f64` in layout.
- SHA-256 of the embedded face is in-tree.
- Removed the numeric-library dependency from `Cargo.toml`.

## [1.0.0] — 2026-09-02

### Added

- Complete LaTeX math parser (`parse`, gold-stable `MathNode`)
- TeX-faithful layout engine (Appendix G style, Table 18 spacing, `Dim`)
- SVG renderer (`render_svg` / `latex_to_svg`) — self-contained SVG 1.1, no font embedding
- PNG renderer (feature `png`) via `tiny-skia` 0.11, DPI-aware, transparent background by default
- egui renderer (feature `egui`) — TrueType tessellation to `egui::Shape` meshes, no SVG intermediate
- Math-mode symbol catalog (Greek, AMS, arrows, operators, font styles) locked to `data/symbols.tsv`
- Full accent and decoration support (TeX placement, extensible hats/arrows/braces, cancel, boxed)
- Multiline environments — `align`, `aligned`, `split`, `gather`, `multline`, `equation`, `{array}`, `{cases}`
- Color support — named, rgb, RGB, HTML, cmyk, gray, `\definecolor`, group scope, `\fcolorbox` borders
- Exact-rational layout math (no hardware `f32`/`f64` in layout)

### Milestone notes

Milestones 1–10 landed as 0.1.0 development commits. This release packages that
surface as 1.0.0: rustdoc, README benchmarks, clippy/fmt, and crates.io metadata.
The crate is publish-ready; crates.io upload is a separate step.

### [1.0.1] - 2026-09-02

### Version to 1.0.1
### Fixed

Resolved an issue where the docs.rs documentation build was failing due to deprecated Rust nightly features (doc_auto_cfg).
Removed:  two temporary build documents Prompt and color addition
Corrected email address of author from jscarr@gmail.com to jscarr1964@gmail.com

### Architecture

Crate modules: `parser/`, `layout/`, `font/`, `render/svg`, `render/png`,
`render/egui`, plus `golds/` and `benches/` in the repository.

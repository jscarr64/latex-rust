//! The vocabulary the generator draws from: every command, environment,
//! delimiter, unit and color form the parser knows, plus hostile text.

/// Symbol commands from the crate's own catalog, loaded once.
pub struct Vocab {
    /// `\name` for every catalog row that is a bare control sequence.
    pub symbols: Vec<String>,
}

impl Vocab {
    /// Build from `latex_rust::symbols()`.
    pub fn load() -> Self {
        let mut symbols = Vec::new();
        for row in latex_rust::symbols() {
            let t = row.latex;
            let bare = t.starts_with('\\') && t.chars().skip(1).all(|c| c.is_ascii_alphabetic());
            if bare {
                symbols.push(String::from(t));
            }
        }
        Vocab { symbols }
    }
}

/// Plain atoms: letters, digits, operators, punctuation, unicode math.
pub const ATOMS: &[&str] = &[
    "x", "y", "z", "a", "b", "n", "f", "E", "0", "1", "2", "7", "9", "+", "-", "=", "<", ">", "!",
    ",", ".", ";", ":", "|", "/", "*", "(", ")", "[", "]", "'", "′", "α", "≤", "→", "∑", "√", "−",
    "x²", "é", "ℝ", "∞", "·", "×",
];

/// Numbers at the edges: huge, tiny, long, leading zeros, repeated separators.
pub const NUMBERS: &[&str] = &[
    "0",
    "00000000000000000000",
    "99999999999999999999999999999999999999",
    "340282366920938463463374607431768211455",
    "170141183460469231731687303715884105728",
    "0.000000000000000000000000000001",
    "1e308",
    "1e-308",
    "123456789012345678901234567890.123456789012345678901234567890",
    "1,000,000",
    "3.14159",
    "2.",
    ".5",
    "٣٤٥",
    "１２３",
];

/// Commands that take no argument and stand alone in math.
pub const NULLARY: &[&str] = &[
    "\\,",
    "\\:",
    "\\;",
    "\\!",
    "\\ ",
    "\\quad",
    "\\qquad",
    "\\strut",
    "\\limits",
    "\\nolimits",
    "\\displaystyle",
    "\\textstyle",
    "\\scriptstyle",
    "\\scriptscriptstyle",
    "\\prime",
    "\\not",
    "\\nonumber",
    "\\notag",
    "\\hline",
    "\\cr",
    "\\middle|",
    "\\,\\,\\,",
    "\\;\\;\\;\\;\\;\\;",
];

/// Lexicon aliases the parser maps onto catalog symbols.
pub const ALIASES: &[&str] = &[
    "\\le",
    "\\ge",
    "\\ne",
    "\\dots",
    "\\lnot",
    "\\dag",
    "\\ddag",
    "\\owns",
    "\\gets",
    "\\to",
    "\\iff",
    "\\implies",
    "\\colon",
    "\\vdots",
    "\\ddots",
    "\\cdots",
];

/// Operators with names, big operators, limit-style forms.
pub const OPERATORS: &[&str] = &[
    "\\sin",
    "\\cos",
    "\\tan",
    "\\cot",
    "\\sec",
    "\\csc",
    "\\arcsin",
    "\\arccos",
    "\\arctan",
    "\\sinh",
    "\\cosh",
    "\\tanh",
    "\\coth",
    "\\log",
    "\\ln",
    "\\lg",
    "\\exp",
    "\\det",
    "\\dim",
    "\\gcd",
    "\\lcm",
    "\\ker",
    "\\deg",
    "\\arg",
    "\\Pr",
    "\\inf",
    "\\sup",
    "\\max",
    "\\min",
    "\\lim",
    "\\liminf",
    "\\limsup",
    "\\sum",
    "\\prod",
    "\\coprod",
    "\\int",
    "\\iint",
    "\\iiint",
    "\\oint",
    "\\oiint",
    "\\bigcup",
    "\\bigcap",
    "\\bigoplus",
    "\\bigotimes",
    "\\bigsqcup",
    "\\biguplus",
    "\\bigvee",
    "\\bigwedge",
];

/// Delimiters accepted after `\left`, `\right`, `\middle`, `\big` and kin.
pub const DELIMS: &[&str] = &[
    "(",
    ")",
    "[",
    "]",
    "\\{",
    "\\}",
    "|",
    "\\|",
    ".",
    "/",
    "\\backslash",
    "\\langle",
    "\\rangle",
    "\\lfloor",
    "\\rfloor",
    "\\lceil",
    "\\rceil",
    "\\vert",
    "\\Vert",
    "\\lvert",
    "\\rvert",
    "\\lVert",
    "\\rVert",
    "\\uparrow",
    "\\downarrow",
    "\\updownarrow",
    "\\Uparrow",
    "\\Downarrow",
    "\\Updownarrow",
    "\\lgroup",
    "\\rgroup",
    "\\lmoustache",
    "\\rmoustache",
    "<",
    ">",
    "\\lbrace",
    "\\rbrace",
    "\\lbrack",
    "\\rbrack",
];

/// Size prefixes for a delimiter.
pub const SIZES: &[&str] = &[
    "big", "Big", "bigg", "Bigg", "bigl", "Bigl", "biggl", "Biggl", "bigr", "Bigr", "biggr",
    "Biggr", "bigm", "Bigm", "biggm", "Biggm",
];

/// Fractions and relatives that take two arguments.
pub const FRACS: &[&str] = &[
    "frac", "dfrac", "tfrac", "cfrac", "binom", "dbinom", "tbinom",
];

/// One-argument accents and over/under decorations.
pub const ACCENTS: &[&str] = &[
    "hat",
    "check",
    "tilde",
    "acute",
    "grave",
    "dot",
    "ddot",
    "dddot",
    "ddddot",
    "breve",
    "bar",
    "vec",
    "mathring",
    "widehat",
    "widetilde",
    "overline",
    "underline",
    "overrightarrow",
    "overleftarrow",
    "overleftrightarrow",
    "underrightarrow",
    "underleftarrow",
    "underleftrightarrow",
    "cancel",
    "bcancel",
    "xcancel",
    "boxed",
    "phantom",
    "hphantom",
    "vphantom",
    "overbrace",
    "underbrace",
];

/// Font and style commands taking one argument.
pub const FONTS: &[&str] = &[
    "mathrm",
    "mathbf",
    "mathit",
    "mathsf",
    "mathtt",
    "mathbb",
    "mathcal",
    "mathfrak",
    "mathscr",
    "boldsymbol",
    "pmb",
    "operatorname",
    "bb",
    "cal",
    "frak",
    "scr",
    "mathnormal",
];

/// Switches used as `{\rm …}`.
pub const SWITCHES: &[&str] = &["rm", "bf", "it", "sf", "tt", "cal", "scr", "frak", "bb"];

/// Text-mode commands.
pub const TEXTS: &[&str] = &[
    "text", "mbox", "textrm", "textbf", "textit", "textsf", "texttt", "fbox", "hbox",
];

/// Text-mode payloads, including escapes and interleaved math.
pub const TEXT_BODIES: &[&str] = &[
    "",
    "a",
    "hello world",
    "  spaced   out  ",
    "if $x>0$ then",
    "\\$ \\& \\% \\# \\_ \\{ \\}",
    "\\textbackslash{}",
    "\\textasciicircum \\textasciitilde",
    "tab\there",
    "nested {braces {here}}",
    "é ü 日本語",
    "\u{5d0}\u{5d1}\u{5d2}",
    "😀 𝔸",
    "x\u{301}\u{302}\u{303}",
    "$\\frac{1}{2}$",
    "a\\\\b",
    "\\ ",
    "\\,",
    "100\\%",
];

/// Environments, with and without the star, and unsupported or unknown ones.
pub const ENVS: &[&str] = &[
    "matrix",
    "pmatrix",
    "bmatrix",
    "Bmatrix",
    "vmatrix",
    "Vmatrix",
    "cases",
    "aligned",
    "align",
    "align*",
    "gather",
    "gather*",
    "multline",
    "multline*",
    "split",
    "equation",
    "equation*",
    "array",
    "smallmatrix",
    "gathered",
    "alignedat",
    "eqnarray",
    "tikzpicture",
    "foo",
    "",
];

/// Column specs for `array`.
pub const COLSPECS: &[&str] = &[
    "c", "l", "r", "cc", "lr", "c|c", "|c|c|", "ccc", "", "x", "{c}", "c c", "||", "@{}c",
    "p{1cm}", "rcl",
];

/// Dimension forms: units, signs, edges, malformed.
pub const DIMS: &[&str] = &[
    "1em",
    "0em",
    "-1em",
    "+1em",
    ".5em",
    "1.5em",
    "1.em",
    "0.0001em",
    "-.5em",
    "10pt",
    "1pt",
    "3.25mu",
    "-18mu",
    "1mu",
    "2bp",
    "1 em",
    "1",
    "0",
    "-0",
    "99999999999999999999em",
    "0.00000000000000000000001em",
    "100000000000000000000000000000000000000em",
    "1e3em",
    "1,5em",
    "--1em",
    "1emm",
    "em",
    ".",
    "1.2.3em",
    "1ex",
    "1cm",
    "1in",
    "1pc",
    "1sp",
    "1dd",
    "1cc",
    "1mm",
    "1px",
    "1\\relax",
    "",
    "\u{661}em",
];

/// Color arguments: names, models, malformed.
pub const COLORS: &[&str] = &[
    "red",
    "blue",
    "Black",
    "White",
    "Apricot",
    "CornflowerBlue",
    "Mahogany",
    "notacolor",
    "",
    "RED",
    "gray",
    "{red}",
];

/// Color model specs `(model, spec)` for `\color[model]{spec}`.
pub const COLOR_MODELS: &[(&str, &str)] = &[
    ("rgb", "1,0,0"),
    ("rgb", "0.5,0.5,0.5"),
    ("rgb", "2,-1,0"),
    ("rgb", "1,0"),
    ("RGB", "255,0,0"),
    ("RGB", "256,0,0"),
    ("RGB", "0,0,0"),
    ("HTML", "FF0000"),
    ("HTML", "ff00ff"),
    ("HTML", "GG0000"),
    ("HTML", "FFF"),
    ("gray", "0.5"),
    ("gray", "2"),
    ("cmyk", "0,1,1,0"),
    ("named", "red"),
    ("named", "zzz"),
    ("zzz", "1"),
    ("rgb", ""),
    ("rgb", "1,,0"),
    ("rgb", "nan,0,0"),
];

/// Hostile characters mixed into text and math.
pub const HOSTILE: &[&str] = &[
    "\0",
    "\u{feff}",
    "\r\n",
    "\r",
    "\n",
    "\t",
    "\u{b}",
    "\u{c}",
    "\u{a0}",
    "\u{2007}",
    "\u{200b}",
    "\u{200e}",
    "\u{200f}",
    "\u{202e}",
    "\u{2028}",
    "\u{2029}",
    "\u{301}",
    "\u{338}",
    "\u{5d0}",
    "\u{627}",
    "\u{10000}",
    "\u{1d538}",
    "\u{1f600}",
    "\u{10ffff}",
    "\u{fffd}",
    "\u{7f}",
    "\u{85}",
    "é",
    "e\u{301}",
    "\u{1}",
    "\u{1b}",
];

/// Whitespace runs, line ends and comments between tokens.
pub const GAPS: &[&str] = &[
    "",
    "",
    "",
    " ",
    "  ",
    "\t",
    "\n",
    "\r\n",
    " \n \n ",
    "%c\n",
    "% a { comment\n",
    "   \t  ",
    "\u{a0}",
    "\u{2003}",
    "%\r\n",
    "\n\n\n\n",
];

/// Commands that must be unknown (typed error, never a render).
pub const UNKNOWN: &[&str] = &[
    "\\notacommandxyz",
    "\\foo",
    "\\Frac",
    "\\SUM",
    "\\fra",
    "\\frac1",
    "\\newcommand",
    "\\def",
    "\\verb",
    "\\input",
    "\\include",
    "\\write18",
    "\\catcode",
    "\\csname",
    "\\expandafter",
    "\\the",
    "\\fi",
    "\\begin",
    "\\end",
    "\\item",
    "\\par",
    "\\\\\\\\",
    "\\@",
    "\\^",
    "\\~",
    "\\_",
    "\\é",
    "\\1",
    "\\\u{0}",
    "\\ \u{301}",
    "\\\u{1f600}",
];

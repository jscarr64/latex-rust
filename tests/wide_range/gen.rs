//! Grammar-driven generator over the whole supported grammar.
//!
//! Deterministic for a seed, bounded in depth (`depth`) and in size (a node
//! budget). It covers every command family, every environment (ragged and
//! empty rows included), every argument form (bare, braced, optional), the
//! dimension and color forms, text and math interleaving, scripts and primes,
//! delimiters, and hostile characters and whitespace.

use super::rng::Rng;
use super::vocab::*;

/// One generator run.
pub struct Gen<'a> {
    /// Seeded stream.
    pub rng: Rng,
    v: &'a Vocab,
    budget: usize,
    /// Only well-formed, supported constructs: every output is valid LaTeX
    /// for this crate's grammar, so a rejection is a wrong rejection.
    pub strict: bool,
}

impl<'a> Gen<'a> {
    /// New generator for `seed`.
    pub fn new(seed: u64, v: &'a Vocab) -> Self {
        Gen {
            rng: Rng::new(seed),
            v,
            budget: 0,
            strict: false,
        }
    }

    /// One expression of nesting depth at most `depth` and about `budget` nodes.
    pub fn expr(&mut self, depth: usize, budget: usize) -> String {
        self.budget = budget;
        self.seq(depth)
    }

    fn gap(&mut self) -> &'static str {
        if self.strict {
            return self.rng.pick(&[" ", "  ", "\t", "\n", " \n ", "%c\n"]);
        }
        self.rng.pick(GAPS)
    }

    fn leaf(&mut self) -> String {
        if self.strict {
            return self.strict_leaf();
        }
        let r = self.rng.below(100);
        if r < 32 {
            String::from(self.rng.pick(ATOMS))
        } else if r < 50 {
            match self.rng.pick_owned(&self.v.symbols) {
                Some(s) => String::from(s),
                None => String::from("x"),
            }
        } else if r < 58 {
            String::from(self.rng.pick(NUMBERS))
        } else if r < 68 {
            String::from(self.rng.pick(OPERATORS))
        } else if r < 76 {
            String::from(self.rng.pick(NULLARY))
        } else if r < 80 {
            String::from(self.rng.pick(HOSTILE))
        } else if r < 84 {
            String::from(self.rng.pick(DELIMS))
        } else if r < 86 {
            String::from(self.rng.pick(ALIASES))
        } else if r < 88 {
            self.text()
        } else if r < 90 {
            String::from(self.rng.pick(UNKNOWN))
        } else {
            let a = self.rng.pick(ATOMS);
            let b = self.rng.pick(ATOMS);
            format!("{a}{b}")
        }
    }

    fn strict_leaf(&mut self) -> String {
        let r = self.rng.below(100);
        if r < 40 {
            String::from(self.rng.pick(&[
                "x", "y", "z", "a", "b", "n", "f", "E", "0", "1", "2", "7", "9", "+", "-", "=",
                "<", ">", "!", ",", ".", ";", "|", "/", "*", "α", "≤", "→", "é", "ℝ",
            ]))
        } else if r < 60 {
            match self.rng.pick_owned(&self.v.symbols) {
                Some(s) => String::from(s),
                None => String::from("x"),
            }
        } else if r < 70 {
            String::from(self.rng.pick(NUMBERS))
        } else if r < 82 {
            String::from(self.rng.pick(OPERATORS))
        } else if r < 88 {
            String::from(self.rng.pick(&["+", "-", "\\cdot", "\\times", "="]))
        } else if r < 94 {
            String::from(self.rng.pick(ALIASES))
        } else {
            let c = self.rng.pick(&["text", "mbox"]);
            let b = self
                .rng
                .pick(&["a", "hello world", "if $x>0$ then", "100\\%", "é ü"]);
            format!("\\{c}{{{b}}}")
        }
    }

    fn seq(&mut self, depth: usize) -> String {
        let n = 1 + self.rng.below(4);
        let mut out = String::new();
        for i in 0..n {
            if i > 0 {
                out.push_str(self.gap());
            }
            out.push_str(&self.node(depth));
        }
        out
    }

    fn spend(&mut self) -> bool {
        if self.budget == 0 {
            return false;
        }
        self.budget -= 1;
        true
    }

    /// A command argument: braced, bare token, empty group, spaced group.
    fn arg(&mut self, depth: usize) -> String {
        let mut r = self.rng.below(10);
        if self.strict && matches!(r, 5 | 7 | 8) {
            r = 0;
        }
        let d = depth.saturating_sub(1);
        match r {
            0 | 1 | 2 | 3 | 4 => format!("{{{}}}", self.node(d)),
            5 => String::from("{}"),
            6 => format!("{{ {} }}", self.seq(d)),
            7 => String::from(
                self.rng
                    .pick(&["a", "1", "x", "\\alpha", "\\infty", "-", "′"]),
            ),
            8 => format!(" {}", self.rng.pick(&["a", "2", "\\beta"])),
            _ => format!("{{{}}}", self.seq(d)),
        }
    }

    fn opt(&mut self, depth: usize) -> String {
        let d = depth.saturating_sub(1);
        match self.rng.below(6) {
            0 => String::from("[]"),
            1 => String::from("[n]"),
            2 => format!("[ {} ]", self.node(d)),
            _ => format!("[{}]", self.node(d)),
        }
    }

    fn text(&mut self) -> String {
        let (cmd, body) = if self.strict {
            (
                self.rng.pick(&["text", "mbox"]),
                self.rng.pick(&["a", "hello world", "100\\%"]),
            )
        } else {
            (self.rng.pick(TEXTS), self.rng.pick(TEXT_BODIES))
        };
        format!("\\{cmd}{{{body}}}")
    }

    fn delim(&mut self) -> &'static str {
        self.rng.pick(DELIMS)
    }

    fn dim(&mut self) -> &'static str {
        self.rng.pick(DIMS)
    }

    fn color_arg(&mut self) -> (String, String) {
        if self.rng.chance(1, 3) {
            let (m, s) = COLOR_MODELS
                .get(self.rng.below(COLOR_MODELS.len()))
                .copied()
                .unwrap_or(("rgb", "0,0,0"));
            (format!("[{m}]"), format!("{{{s}}}"))
        } else {
            (String::new(), format!("{{{}}}", self.rng.pick(COLORS)))
        }
    }

    fn cell(&mut self, depth: usize) -> String {
        if self.rng.chance(1, 4) {
            String::new()
        } else {
            self.node(depth)
        }
    }

    fn row(&mut self, depth: usize, cols: usize, sep: &str) -> String {
        let mut out = String::new();
        for c in 0..cols {
            if c > 0 {
                out.push_str(sep);
            }
            out.push_str(&self.cell(depth));
        }
        out
    }

    fn env(&mut self, depth: usize) -> String {
        let d = depth.saturating_sub(1);
        let name = if self.rng.chance(7, 8) {
            self.rng.pick(&[
                "matrix", "pmatrix", "bmatrix", "Bmatrix", "vmatrix", "Vmatrix", "cases",
                "aligned", "align", "gather", "array", "split", "multline", "equation",
            ])
        } else {
            self.rng.pick(ENVS)
        };
        let bare = name.trim_end_matches('*');
        let mut head = format!("\\begin{{{name}}}");
        if bare == "array" {
            head.push('{');
            head.push_str(self.rng.pick(COLSPECS));
            head.push('}');
        }
        let rows = self.rng.below(5);
        let ragged = self.rng.chance(1, 2);
        let base_cols = self.rng.below(4);
        let eq_like = matches!(bare, "aligned" | "align" | "split" | "gather" | "multline");
        let mut body = String::new();
        for r in 0..rows {
            if r > 0 {
                let sep = self.rng.pick(&[
                    "\\\\",
                    "\\\\",
                    "\\\\[2pt]",
                    "\\\\[1em]",
                    "\\cr",
                    " \\\\ ",
                    "\\\\\\hline",
                ]);
                body.push_str(sep);
                if self.rng.chance(1, 8) {
                    body.push_str("\\hline");
                }
            }
            let cols = if ragged { self.rng.below(5) } else { base_cols };
            let sep = if eq_like && self.rng.chance(1, 2) {
                "&="
            } else {
                "&"
            };
            body.push_str(&self.row(d, cols, sep));
            if eq_like && self.rng.chance(1, 5) {
                body.push_str(self.rng.pick(&[
                    "\\nonumber",
                    "\\notag",
                    "\\tag{7}",
                    "\\label{eq1}",
                    "\\tag*{(*)}",
                ]));
            }
            if bare == "cases" && self.rng.chance(1, 3) {
                body.push_str("&\\text{if } x>0");
            }
        }
        if self.rng.chance(1, 6) {
            body.push_str("\\\\");
        }
        if eq_like && self.rng.chance(1, 8) {
            body.push_str("\\\\\\intertext{between}");
            body.push_str(&self.cell(d));
        }
        let tail = if self.rng.chance(1, 40) {
            String::new()
        } else if self.rng.chance(1, 40) {
            String::from("\\end{foo}")
        } else {
            format!("\\end{{{name}}}")
        };
        format!("{head}{body}{tail}")
    }

    fn scripts(&mut self, depth: usize) -> String {
        let d = depth.saturating_sub(1);
        let base = match self.rng.below(5) {
            0 if self.strict => String::from(self.rng.pick(&["x", "y", "n", "1", "2"])),
            0 => String::from(self.rng.pick(ATOMS)),
            1 => format!("{{{}}}", self.node(d)),
            2 if !self.strict => String::new(),
            3 => String::from(self.rng.pick(OPERATORS)),
            _ => self.leaf(),
        };
        let sup = self.arg(d);
        let sub = self.arg(d);
        let primes = match self.rng.below(6) {
            0 => "'",
            1 => "''",
            2 => "'''",
            3 => "′",
            4 if self.strict => "''",
            4 => "^\\prime",
            _ => "",
        };
        let which = match self.rng.below(9) {
            7 if self.strict => 0,
            k => k,
        };
        match which {
            0 => format!("{base}^{sup}"),
            1 => format!("{base}_{sub}"),
            2 => format!("{base}^{sup}_{sub}"),
            3 => format!("{base}_{sub}^{sup}"),
            4 => format!("{base}{primes}"),
            5 => format!("{base}{primes}_{sub}"),
            6 => format!("{base}{primes}^{sup}"),
            7 => format!("{base}^{sup}^{sub}"),
            _ => {
                let k = 1 + self.rng.below(60);
                format!("{base}{}", "'".repeat(k))
            }
        }
    }

    fn big_op(&mut self, depth: usize) -> String {
        let d = depth.saturating_sub(1);
        let op = self.rng.pick(OPERATORS);
        let lim = self.rng.pick(&["", "", "\\limits", "\\nolimits"]);
        let a = self.arg(d);
        let b = self.arg(d);
        match self.rng.below(5) {
            0 => format!("{op}{lim}_{a}^{b}"),
            1 => format!("{op}{lim}^{b}_{a}"),
            2 => format!("{op}{lim}_{a}"),
            3 => format!("{op}{lim}^{b}"),
            _ => format!("{op}{lim}_{a}^{b}{}", self.node(d)),
        }
    }

    /// Strict versions of the arms that can otherwise produce invalid input.
    fn strict_node(&mut self, d: usize, pick: usize) -> String {
        match pick {
            7 => {
                let ds = [
                    "(", ")", "[", "]", "\\{", "\\}", "|", "\\|", ".", "\\langle", "\\rangle",
                    "\\lfloor", "\\rfloor", "\\lceil", "\\rceil", "\\vert", "\\Vert",
                ];
                let l = self.rng.pick(&ds);
                let r = self.rng.pick(&ds);
                format!("\\left{l} {} \\right{r} ", self.node(d))
            }
            9 | 10 => {
                let name = self.rng.pick(&[
                    "matrix", "pmatrix", "bmatrix", "Bmatrix", "vmatrix", "Vmatrix", "cases",
                    "aligned", "align", "gather", "split",
                ]);
                let rows = 1 + self.rng.below(3);
                let cols = 1 + self.rng.below(3);
                let mut body = String::new();
                for r in 0..rows {
                    if r > 0 {
                        body.push_str("\\\\");
                    }
                    for c in 0..cols {
                        if c > 0 {
                            body.push('&');
                        }
                        body.push_str(&self.node(d));
                    }
                }
                format!("\\begin{{{name}}}{body}\\end{{{name}}}")
            }
            17 => {
                let dim = self.rng.pick(&[
                    "1em", "0em", "-1em", ".5em", "1.5em", "10pt", "3.25mu", "2bp", "0.0001em",
                ]);
                if self.rng.chance(1, 2) {
                    format!("x\\hspace{{{dim}}}")
                } else {
                    format!("x\\rule{{{dim}}}{{1em}}")
                }
            }
            18 => {
                let c = self
                    .rng
                    .pick(&["red", "blue", "Black", "Apricot", "CornflowerBlue"]);
                match self.rng.below(3) {
                    0 => format!("{{\\color{{{c}}} {}}}", self.node(d)),
                    1 => format!("\\textcolor{{{c}}}{}", self.arg(d)),
                    _ => format!("\\textcolor[rgb]{{0.5,0.5,0.5}}{}", self.arg(d)),
                }
            }
            23 => {
                let l = self.rng.pick(&["(", "\\{", "\\langle", ""]);
                let r = self.rng.pick(&[")", "\\}", "\\rangle", ""]);
                format!(
                    "\\genfrac{{{l}}}{{{r}}}{{}}{{}}{}{}",
                    self.arg(d),
                    self.arg(d)
                )
            }
            24 => String::from("x"),
            26 => format!("\\not{}", self.rng.pick(&["=", "\\in", "<", "\\subset"])),
            _ => self.node(d),
        }
    }

    fn node(&mut self, depth: usize) -> String {
        if depth == 0 || !self.spend() {
            return self.leaf();
        }
        let d = depth - 1;
        let mut pick = self.rng.below(30);
        if self.strict && matches!(pick, 7 | 9 | 10 | 17 | 18 | 23 | 24 | 26 | 27 | 28 | 29) {
            return self.strict_node(d, pick);
        }
        if self.strict && pick == 3 {
            pick = 2;
        }
        match pick {
            0 => self.seq(d),
            1 => format!("{{{}}}", self.seq(d)),
            2 => {
                let c = if self.strict {
                    self.rng.pick(&["frac", "dfrac", "tfrac", "cfrac", "binom"])
                } else {
                    self.rng.pick(FRACS)
                };
                format!("\\{c}{}{}", self.arg(d), self.arg(d))
            }
            3 => {
                let c = self.rng.pick(&["over", "choose", "atop", "above"]);
                format!("{{{}\\{c} {}}}", self.node(d), self.node(d))
            }
            4 => {
                if self.rng.chance(1, 2) {
                    format!("\\sqrt{}", self.arg(d))
                } else {
                    format!("\\sqrt{}{}", self.opt(d), self.arg(d))
                }
            }
            5 | 6 => self.scripts(d + 1),
            7 => {
                let (l, r) = (self.delim(), self.delim());
                let mid = if self.rng.chance(1, 3) {
                    format!("\\middle{}{}", self.delim(), self.node(d))
                } else {
                    String::new()
                };
                format!("\\left{l}{}{mid}\\right{r}", self.node(d))
            }
            8 => {
                if self.strict {
                    let s = self.rng.pick(&["big", "Big", "bigg", "Bigg"]);
                    let d = self.rng.pick(&["(", ")", "[", "]", "|", "\\langle"]);
                    return format!("\\{s}{d}");
                }
                let s = self.rng.pick(SIZES);
                format!("\\{s}{}", self.delim())
            }
            9 | 10 => self.env(d + 1),
            11 => {
                let c = if self.strict {
                    self.rng.pick(&[
                        "hat",
                        "bar",
                        "vec",
                        "tilde",
                        "dot",
                        "overline",
                        "underline",
                        "widehat",
                        "overrightarrow",
                        "cancel",
                        "boxed",
                        "phantom",
                        "hphantom",
                        "vphantom",
                        "overbrace",
                        "underbrace",
                    ])
                } else {
                    self.rng.pick(ACCENTS)
                };
                format!("\\{c}{}", self.arg(d))
            }
            12 => {
                let c = self.rng.pick(&["overset", "underset", "stackrel"]);
                format!("\\{c}{}{}", self.arg(d), self.arg(d))
            }
            13 => {
                let c = self.rng.pick(&["xrightarrow", "xleftarrow"]);
                if self.rng.chance(1, 2) {
                    format!("\\{c}{}{}", self.opt(d), self.arg(d))
                } else {
                    format!("\\{c}{}", self.arg(d))
                }
            }
            14 => {
                let c = if self.strict {
                    self.rng.pick(&[
                        "mathrm", "mathbf", "mathit", "mathsf", "mathtt", "mathbb", "mathcal",
                        "mathfrak",
                    ])
                } else {
                    self.rng.pick(FONTS)
                };
                format!("\\{c}{}", self.arg(d))
            }
            15 => {
                let c = if self.strict {
                    self.rng.pick(&["rm", "bf", "it", "sf", "tt"])
                } else {
                    self.rng.pick(SWITCHES)
                };
                format!("{{\\{c} {}}}", self.node(d))
            }
            16 => self.text(),
            17 => {
                let t = self.rng.pick(&["hspace", "hspace*", "rule"]);
                if t == "rule" {
                    let raise = if self.rng.chance(1, 4) {
                        format!("[{}]", self.dim())
                    } else {
                        String::new()
                    };
                    format!("\\rule{raise}{{{}}}{{{}}}", self.dim(), self.dim())
                } else {
                    format!("\\{t}{{{}}}", self.dim())
                }
            }
            18 => {
                let (m, s) = self.color_arg();
                match self.rng.below(5) {
                    0 => format!("{{\\color{m}{s} {}}}", self.node(d)),
                    1 => format!("\\textcolor{m}{s}{}", self.arg(d)),
                    2 => format!("\\colorbox{m}{s}{}", self.arg(d)),
                    3 => {
                        let (m2, s2) = self.color_arg();
                        format!("\\fcolorbox{m}{s}{m2}{s2}{}", self.arg(d))
                    }
                    _ => format!(
                        "\\definecolor{{mine}}{{rgb}}{{0.1,0.2,0.3}}\\textcolor{{mine}}{}",
                        self.arg(d)
                    ),
                }
            }
            19 | 20 => self.big_op(d + 1),
            21 => {
                let a = self.node(d);
                let b = self.node(d);
                format!("\\substack{{{a}\\\\{b}}}")
            }
            22 => format!("\\cancelto{}{}", self.arg(d), self.arg(d)),
            23 => {
                let l = self.rng.pick(&["(", "[", "\\{", "", "\\langle"]);
                let r = self.rng.pick(&[")", "]", "\\}", "", "\\rangle"]);
                let th = self.rng.pick(&["", "0pt", "1pt", "2pt", "-1pt", "x"]);
                let st = self.rng.pick(&["", "0", "1", "2", "3", "4"]);
                format!(
                    "\\genfrac{{{l}}}{{{r}}}{{{th}}}{{{st}}}{}{}",
                    self.arg(d),
                    self.arg(d)
                )
            }
            24 => {
                let c = self.rng.pick(&[
                    "\\label{k}",
                    "\\ref{k}",
                    "\\eqref{k}",
                    "\\tag{1}",
                    "\\tag*{x}",
                    "\\label{}",
                    "\\ref{ }",
                    "\\label{a b}",
                    "\\tag{}",
                ]);
                String::from(c)
            }
            25 => {
                let c = self.rng.pick(&[
                    "displaystyle",
                    "textstyle",
                    "scriptstyle",
                    "scriptscriptstyle",
                ]);
                format!("\\{c} {}", self.node(d))
            }
            26 => format!(
                "\\not{}",
                self.rng
                    .pick(&["=", "\\in", "<", "\\subset", "x", "{}", "\\alpha"])
            ),
            27 => {
                let w = self.rng.pick(&["\\(", "\\[", "$", "$$"]);
                let wc = match w {
                    "\\(" => "\\)",
                    "\\[" => "\\]",
                    other => other,
                };
                format!("{w}{}{wc}", self.node(d))
            }
            28 => {
                let c = self
                    .rng
                    .pick(&["\\\\", "\\cr", "&", "\\hline", "\\\\[3pt]"]);
                String::from(c)
            }
            _ => {
                let a = self.node(d);
                let hostile = self.rng.pick(HOSTILE);
                format!("{a}{hostile}")
            }
        }
    }
}

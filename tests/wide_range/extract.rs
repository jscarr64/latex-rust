//! Pull LaTeX strings out of TOML golds and markdown text, without panicking
//! slices or indexing.

/// Decode the inside of a TOML basic string. `None` on a bad escape.
pub fn unescape_toml_basic(body: &str) -> Option<String> {
    let mut out = String::new();
    let mut it = body.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let esc = it.next()?;
        match esc {
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            'b' => out.push('\u{8}'),
            'f' => out.push('\u{c}'),
            'u' | 'U' => {
                let want = if esc == 'u' { 4 } else { 8 };
                let mut hex = String::new();
                for _ in 0..want {
                    hex.push(it.next()?);
                }
                let code = u32::from_str_radix(&hex, 16).ok()?;
                out.push(char::from_u32(code)?);
            }
            _ => return None,
        }
    }
    Some(out)
}

/// Values of `key = "…"` lines (single-line basic strings) for the given keys.
pub fn toml_values(text: &str, keys: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        for key in keys {
            let Some(rest) = line.strip_prefix(key) else {
                continue;
            };
            let Some(rest) = rest.trim_start().strip_prefix('=') else {
                continue;
            };
            let rest = rest.trim();
            if rest.starts_with("\"\"\"") {
                continue;
            }
            let Some(inner) = rest.strip_prefix('"').and_then(|r| r.strip_suffix('"')) else {
                continue;
            };
            if let Some(s) = unescape_toml_basic(inner) {
                out.push(s);
            }
        }
    }
    out
}

fn has_command(s: &str) -> bool {
    let mut prev_bs = false;
    for c in s.chars() {
        if prev_bs && c.is_ascii_alphabetic() {
            return true;
        }
        prev_bs = c == '\\';
    }
    false
}

/// Math-looking spans of a markdown file: `$$…$$`, `$…$`, `\(…\)`, `\[…\]`,
/// and backtick spans that hold a control word.
pub fn math_spans(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let at = |i: usize| chars.get(i).copied();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        let c = at(i);
        if c == Some('`') {
            // fenced block: skip to the closing fence
            if at(i + 1) == Some('`') && at(i + 2) == Some('`') {
                i += 3;
                while i < chars.len()
                    && !(at(i) == Some('`') && at(i + 1) == Some('`') && at(i + 2) == Some('`'))
                {
                    i += 1;
                }
                i += 3;
                continue;
            }
            let mut j = i + 1;
            while j < chars.len() && at(j) != Some('`') && at(j) != Some('\n') {
                j += 1;
            }
            if at(j) == Some('`') && j > i + 1 {
                let s: String = chars.iter().skip(i + 1).take(j - i - 1).collect();
                if has_command(&s) {
                    out.push(s);
                }
                i = j + 1;
                continue;
            }
            i += 1;
        } else if c == Some('$') && at(i + 1) == Some('$') {
            let mut j = i + 2;
            while j < chars.len() && !(at(j) == Some('$') && at(j + 1) == Some('$')) {
                j += 1;
            }
            if j < chars.len() {
                let s: String = chars.iter().skip(i + 2).take(j - i - 2).collect();
                if !s.trim().is_empty() {
                    out.push(s);
                }
                i = j + 2;
                continue;
            }
            i += 2;
        } else if c == Some('$') {
            let mut j = i + 1;
            while j < chars.len() && at(j) != Some('$') && at(j) != Some('\n') {
                j += 1;
            }
            let s: String = chars.iter().skip(i + 1).take(j - i - 1).collect();
            if at(j) == Some('$') && !s.is_empty() && !s.starts_with(' ') {
                out.push(s);
                i = j + 1;
                continue;
            }
            i += 1;
        } else if c == Some('\\') && (at(i + 1) == Some('(') || at(i + 1) == Some('[')) {
            let close = if at(i + 1) == Some('(') { ')' } else { ']' };
            let mut j = i + 2;
            while j + 1 < chars.len()
                && !(at(j) == Some('\\') && at(j + 1) == Some(close))
                && at(j) != Some('\n')
            {
                j += 1;
            }
            if at(j) == Some('\\') && at(j + 1) == Some(close) {
                let s: String = chars.iter().skip(i + 2).take(j - i - 2).collect();
                if !s.trim().is_empty() {
                    out.push(s);
                }
                i = j + 2;
                continue;
            }
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

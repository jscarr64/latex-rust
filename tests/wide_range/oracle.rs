//! Hand-countable oracles: a string whose unescaped braces, `\left`/`\right`
//! or `\begin`/`\end` do not pair up must be rejected, whatever else it holds.

/// Net unescaped brace depth of `s` and the byte offsets of every unescaped
/// brace. A `\` escapes the next character; `%` starts a comment to the end
/// of the line. Returns `None` if the string has a comment (the oracle does
/// not model them).
pub fn braces(s: &str) -> Option<(i64, Vec<(usize, char)>)> {
    let mut net = 0i64;
    let mut at = Vec::new();
    let mut esc = false;
    for (i, c) in s.char_indices() {
        if esc {
            esc = false;
            continue;
        }
        match c {
            '\\' => esc = true,
            '%' => return None,
            '{' => {
                net += 1;
                at.push((i, c));
            }
            '}' => {
                net -= 1;
                at.push((i, c));
            }
            _ => {}
        }
    }
    Some((net, at))
}

/// Count of the control word `\word` (not a longer word such as `\leftarrow`).
pub fn count_word(s: &str, word: &str) -> usize {
    let pat = format!("\\{word}");
    let mut n = 0;
    let mut rest = s;
    while let Some(i) = rest.find(&pat) {
        let after = rest.get(i + pat.len()..).unwrap_or("");
        let before_bs = rest
            .get(..i)
            .map_or(0, |b| b.chars().rev().take_while(|c| *c == '\\').count());
        let next_is_letter = after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic());
        if !next_is_letter && before_bs % 2 == 0 {
            n += 1;
        }
        rest = after;
    }
    n
}

/// Remove the unescaped brace at byte offset `at` (which must be a brace).
pub fn delete_at(s: &str, at: usize) -> String {
    let mut out = String::with_capacity(s.len());
    for (i, c) in s.char_indices() {
        if i != at {
            out.push(c);
        }
    }
    out
}

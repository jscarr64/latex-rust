//! Exhaustive enumeration of every string of up to N tokens over a small
//! alphabet, in a fixed order, split across workers by ordinal.

/// Number of strings of length `0..=max_len` over `alphabet_len` tokens.
pub fn count_strings(alphabet_len: usize, max_len: usize) -> u64 {
    let mut total = 0u64;
    let mut power = 1u64;
    for _ in 0..=max_len {
        total = total.saturating_add(power);
        power = power.saturating_mul(alphabet_len as u64);
    }
    total
}

/// Visit every concatenation of `0..=max_len` tokens, shortest first, in
/// lexicographic token order. Only strings whose 0-based ordinal `n` has
/// `n % stride == first` reach `f`. Returns how many reached `f`, or the first
/// `Err`.
pub fn for_each_string(
    tokens: &[&str],
    max_len: usize,
    first: usize,
    stride: usize,
    f: &mut dyn FnMut(&str) -> Result<(), String>,
) -> Result<u64, String> {
    let stride = stride.max(1) as u64;
    let first = first as u64;
    let mut ordinal = 0u64;
    let mut visited = 0u64;
    let mut buf = String::new();
    for len in 0..=max_len {
        let mut digits = vec![0usize; len];
        loop {
            if ordinal % stride == first {
                buf.clear();
                for d in &digits {
                    buf.push_str(tokens.get(*d).copied().unwrap_or(""));
                }
                f(&buf)?;
                visited += 1;
            }
            ordinal += 1;
            let mut pos = len;
            let mut carried = true;
            while pos > 0 && carried {
                pos -= 1;
                match digits.get_mut(pos) {
                    Some(d) => {
                        *d += 1;
                        if *d < tokens.len() {
                            carried = false;
                        } else {
                            *d = 0;
                        }
                    }
                    None => break,
                }
            }
            if carried {
                break;
            }
        }
    }
    Ok(visited)
}

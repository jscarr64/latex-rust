//! Test support module for generating random mutations of strings.

use super::rng::Rng;

/// Applies one random mutation to the input string `src` using the provided RNG.
/// The mutation is chosen from 9 possible operations, each with different behavior.
///
/// # Arguments
/// * `rng` - Random number generator used to select mutation and determine positions
/// * `src` - Input string to be mutated
/// * `hostile` - Slice of strings to be randomly inserted during mutation
///
/// # Returns
/// A new String containing the mutated version of `src`
pub fn mutate(rng: &mut Rng, src: &str, hostile: &[&'static str]) -> String {
    let mut chars: Vec<char> = src.chars().collect();
    let len = chars.len();

    match rng.below(9) {
        0 => {
            // Delete one random char
            if len > 0 {
                let pos = rng.below(len);
                chars.remove(pos);
            }
        }
        1 => {
            // Insert one random item from hostile at random position
            if !hostile.is_empty() {
                let pos = rng.below(len + 1);
                let item = rng.pick(hostile);
                chars.splice(pos..pos, item.chars());
            }
        }
        2 => {
            // Truncate at random position
            let pos = rng.below(len + 1);
            chars.truncate(pos);
        }
        3 => {
            // Duplicate a random chunk (up to 8 chars) in place right after itself
            if len > 0 {
                let start = rng.below(len);
                let end = (start + 1 + rng.below(8.min(len - start))).min(len);
                let chunk: Vec<char> = chars
                    .iter()
                    .skip(start)
                    .take(end - start)
                    .copied()
                    .collect();
                chars.splice(end..end, chunk);
            }
        }
        4 => {
            // Swap two adjacent chars
            if len >= 2 {
                let pos = rng.below(len - 1);
                chars.swap(pos, pos + 1);
            }
        }
        5 => {
            // Insert a stray "}" at random position
            let pos = rng.below(len + 1);
            chars.insert(pos, '}');
        }
        6 => {
            // Insert a stray "{" at random position
            let pos = rng.below(len + 1);
            chars.insert(pos, '{');
        }
        7 => {
            // Delete a random run of 1..=4 chars
            if len > 0 {
                let run_len = 1 + rng.below(4.min(len));
                let start = rng.below(len - run_len + 1);
                chars.drain(start..start + run_len);
            }
        }
        8 => {
            // Insert one of the special strings at random position
            let pos = rng.below(len + 1);
            let item = rng.pick(&[
                "\\left",
                "\\right",
                "\\end{matrix}",
                "\\begin{matrix}",
                "&",
                "\\\\",
                "^",
                "_",
                "\\",
            ]);
            chars.splice(pos..pos, item.chars());
        }
        _ => {}
    }

    chars.into_iter().collect()
}

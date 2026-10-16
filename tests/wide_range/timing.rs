//! Timing guards that cannot flake: best-of-N minimum, a scaling ratio only
//! when the small case is above timer noise, and generous absolute budgets.

use std::time::{Duration, Instant};

/// Shortest of `n` runs of `f`.
pub fn best_of(n: usize, mut f: impl FnMut()) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..n.max(1) {
        let t = Instant::now();
        f();
        let d = t.elapsed();
        if d < best {
            best = d;
        }
    }
    best
}

/// Below this a measurement is timer and scheduler noise.
pub const NOISE: Duration = Duration::from_millis(3);

/// Absolute budget for one call on the biggest guarded input.
pub fn budget() -> Duration {
    if cfg!(debug_assertions) {
        Duration::from_secs(20)
    } else {
        Duration::from_secs(5)
    }
}

/// `f(small)` and `f(big)` where `big` is `factor` times the input size.
/// Fails if `f(big)` is over the budget, or, when `f(small)` is above noise,
/// if it takes more than `limit` times `f(small)`.
pub fn guard_scaling(
    label: &str,
    factor: u32,
    limit: u32,
    mut small: impl FnMut(),
    mut big: impl FnMut(),
) -> Result<(), String> {
    let t1 = best_of(5, &mut small);
    let t2 = best_of(5, &mut big);
    if t2 > budget() {
        return Err(format!(
            "{label}: {t2:?} for {factor}x input is over budget"
        ));
    }
    if t1 >= NOISE && t2 > t1 * limit {
        return Err(format!(
            "{label}: {factor}x input took {t2:?} against {t1:?} (limit {limit}x)"
        ));
    }
    Ok(())
}

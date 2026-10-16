//! Deterministic splitmix64 generator for the wide-range tests.
//! Same seed, same stream, on every machine. No floats, no external crates.

/// splitmix64 state.
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Start a stream at `seed`.
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    /// Next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Value in `0..n`; `0` when `n == 0`.
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() % (n as u64)) as usize
    }

    /// True with probability `num / den`; false when `den == 0`.
    pub fn chance(&mut self, num: usize, den: usize) -> bool {
        if den == 0 {
            return false;
        }
        (self.next_u64() % (den as u64)) < (num as u64)
    }

    /// One of `xs`, or `""` when `xs` is empty.
    pub fn pick(&mut self, xs: &[&'static str]) -> &'static str {
        let i = self.below(xs.len());
        xs.get(i).copied().unwrap_or("")
    }

    /// Index into a slice of owned strings; `None` when empty.
    pub fn pick_owned<'a>(&mut self, xs: &'a [String]) -> Option<&'a str> {
        let i = self.below(xs.len());
        xs.get(i).map(String::as_str)
    }
}

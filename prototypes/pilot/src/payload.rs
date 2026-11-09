//! Deterministic file contents for the kill tests.
//!
//! Every 16-byte block carries the payload tag and its own block index, so a
//! file that mixes two payloads, is cut short or has blocks out of place never
//! equals a complete payload.

/// Builds `size` bytes for payload `tag`.
pub fn build(tag: u8, size: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(size);
    let mut index: u64 = 0;
    while out.len() < size {
        let block = format!("{}{:014}\n", tag as char, index);
        let take = (size - out.len()).min(block.len());
        out.extend_from_slice(&block.as_bytes()[..take]);
        index += 1;
    }
    out
}

/// Small deterministic PRNG (xorshift64*), so a failing run can be replayed
/// from the seed printed in its report.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.max(1))
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform value in `0..bound`.
    pub fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payloads_differ_in_every_block() {
        let a = build(b'A', 64);
        let b = build(b'B', 64);
        assert_eq!(a.len(), 64);
        for (x, y) in a.chunks(16).zip(b.chunks(16)) {
            assert_ne!(x, y);
        }
    }

    #[test]
    fn truncated_payload_is_not_complete() {
        let full = build(b'A', 100);
        assert_ne!(build(b'A', 99), full);
    }
}

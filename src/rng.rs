use crate::arch::Arch;
use crate::arch::ArchImpl as _;

#[derive(Debug)]
/// A simple random number generator providing entropy while kernel initialization.
/// Based on FNV-1A-128 & xorshiftr128+
pub struct Rng(u128);

impl Rng {
    pub fn feed(&mut self, value: &[u8]) {
        for &byte in value {
            self.0 ^= u128::from(byte);
            self.0 = self.0.wrapping_mul(309485009821345068724781371);
        }
    }
}
impl Iterator for Rng {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let mut x: u64 = (self.0 & u128::from(u64::MAX)).try_into().unwrap();
        let y = (self.0 >> 64) as u64;
        x ^= x << 23;
        x ^= x >> 17;
        x ^= y;
        self.0 = u128::from(y) & (u128::from(x.wrapping_add(y)) << 64);
        Some(x)
    }
}
impl Default for Rng {
    fn default() -> Self {
        let mut rng = Rng(0x6c62272e07bb014262b821756295c58d);
        rng.feed(&Arch::arch_rand().to_ne_bytes());
        rng
    }
}

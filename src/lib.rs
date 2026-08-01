#![cfg_attr(not(test), no_std)]

use core::convert::Infallible;

use rand_core::TryRng;
use rand_core::utils::{fill_bytes_via_next_word, next_u64_via_u32};
pub use rand_core::{Rng, SeedableRng};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SquirrelRng {
    position: u32,
    seed: u32,
}

impl SquirrelRng {
    #[cfg(feature = "getrandom")]
    pub fn new() -> Self {
        Self::with_seed(getrandom::u32().expect("OS entropy source failed"))
    }

    pub fn with_seed(seed: u32) -> Self {
        Self { position: 0, seed }
    }

    pub fn with_position(self, position: u32) -> Self {
        Self { position, ..self }
    }

    pub fn gen_range(&mut self, range: core::ops::Range<u32>) -> u32 {
        let len = range.end - range.start;
        if len <= 1 {
            return range.start;
        }
        let zone = u32::MAX - (u32::MAX % len);
        loop {
            let r = self.next_u32();
            if r < zone {
                return range.start + (r % len);
            }
        }
    }

    pub fn f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn gen_bool(&mut self, p: f64) -> bool {
        self.f64() < p
    }

    pub fn pick<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            return None;
        }
        let idx = self.gen_range(0..slice.len() as u32) as usize;
        Some(&slice[idx])
    }
}

#[cfg(feature = "getrandom")]
impl Default for SquirrelRng {
    fn default() -> Self {
        SquirrelRng::new()
    }
}

impl TryRng for SquirrelRng {
    type Error = Infallible;

    #[inline]
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        let result = squirrel3(self.position, self.seed);
        self.position = self.position.wrapping_add(1);
        Ok(result)
    }

    #[inline]
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        next_u64_via_u32(self)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        fill_bytes_via_next_word(dst, || self.try_next_u64())
    }
}

impl SeedableRng for SquirrelRng {
    type Seed = [u8; 4];

    fn from_seed(seed: Self::Seed) -> Self {
        Self::with_seed(u32::from_le_bytes(seed))
    }
}

#[inline]
pub fn squirrel3(position: u32, seed: u32) -> u32 {
    const BIT_NOISE1: u32 = 0x68E31DA4;
    const BIT_NOISE2: u32 = 0xB5297A4D;
    const BIT_NOISE3: u32 = 0x1B56C4E9;

    let mut mangled = position;
    mangled = mangled.wrapping_mul(BIT_NOISE1);
    mangled = mangled.wrapping_add(seed);
    mangled ^= mangled >> 8;
    mangled = mangled.wrapping_add(BIT_NOISE2);
    mangled ^= mangled << 8;
    mangled = mangled.wrapping_mul(BIT_NOISE3);
    mangled ^= mangled >> 8;
    mangled
}

#[cfg(test)]
mod tests {
    use rand_core::Rng;

    use crate::SquirrelRng;

    #[test]
    fn copy_with_position_does_not_modify_original() {
        let mut a = SquirrelRng::with_seed(3);
        let mut b = a.with_position(1);

        let second_value = b.next_u32();

        assert_ne!(a.next_u32(), second_value);
        assert_eq!(a.next_u32(), second_value);
    }

    #[test]
    fn gen_range_stays_in_bounds() {
        let mut rng = SquirrelRng::with_seed(42);
        for _ in 0..1000 {
            let r = rng.gen_range(10..20);
            assert!((10..20).contains(&r));
        }
    }

    #[test]
    fn gen_range_single_element_range() {
        let mut rng = SquirrelRng::with_seed(42);
        for _ in 0..10 {
            assert_eq!(rng.gen_range(7..8), 7);
        }
    }

    #[test]
    fn floats_stay_in_half_open_unit_interval() {
        let mut rng = SquirrelRng::with_seed(42);
        for _ in 0..1000 {
            let f = rng.f32();
            assert!((0.0..1.0).contains(&f));
            let g = rng.f64();
            assert!((0.0..1.0).contains(&g));
        }
    }

    #[test]
    fn gen_bool_extremes_are_deterministic() {
        let mut rng = SquirrelRng::with_seed(42);
        for _ in 0..100 {
            assert!(!rng.gen_bool(0.0));
            assert!(rng.gen_bool(1.0));
        }
    }

    #[test]
    fn pick_handles_empty_and_nonempty_slices() {
        let mut rng = SquirrelRng::with_seed(42);
        let empty: [i32; 0] = [];
        assert!(rng.pick(&empty).is_none());
        let slice = [10, 20, 30, 40];
        for _ in 0..100 {
            let r = rng.pick(&slice).unwrap();
            assert!(slice.contains(r));
        }
    }
}

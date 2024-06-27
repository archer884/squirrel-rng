pub use rand::{Rng, RngCore, SeedableRng};

use rand::rngs::{OsRng, ThreadRng};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SquirrelRng {
    position: u32,
    seed: u32,
}

impl SquirrelRng {
    pub fn new() -> Self {
        Self {
            position: 0,
            seed: rand::thread_rng().next_u32(),
        }
    }

    pub fn with_seed(seed: u32) -> Self {
        Self { position: 0, seed }
    }

    pub fn seed_from(mut rng: impl Rng) -> Self {
        Self {
            position: 0,
            seed: rng.next_u32(),
        }
    }

    pub fn with_position(self, position: u32) -> Self {
        Self { position, ..self }
    }
}

impl Default for SquirrelRng {
    fn default() -> Self {
        SquirrelRng::new()
    }
}

impl From<ThreadRng> for SquirrelRng {
    fn from(value: ThreadRng) -> Self {
        Self::seed_from(value)
    }
}

impl From<OsRng> for SquirrelRng {
    fn from(value: OsRng) -> Self {
        Self::seed_from(value)
    }
}

impl RngCore for SquirrelRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let result = squirrel3(self.position, self.seed);
        self.position = self.position.wrapping_add(1);
        result
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        next_u64_via_u32(self)
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        fill_bytes_via_next(self, dest);
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
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

// These two implementations are taken directly from the rand library.

/// Implement `next_u64` via `next_u32`, little-endian order.
pub fn next_u64_via_u32<R: RngCore + ?Sized>(rng: &mut R) -> u64 {
    // Use LE; we explicitly generate one value before the next.
    let x = u64::from(rng.next_u32());
    let y = u64::from(rng.next_u32());
    (y << 32) | x
}

/// Implement `fill_bytes` via `next_u64` and `next_u32`, little-endian order.
///
/// The fastest way to fill a slice is usually to work as long as possible with
/// integers. That is why this method mostly uses `next_u64`, and only when
/// there are 4 or less bytes remaining at the end of the slice it uses
/// `next_u32` once.
fn fill_bytes_via_next<R: RngCore + ?Sized>(rng: &mut R, dest: &mut [u8]) {
    let mut left = dest;
    while left.len() >= 8 {
        let (l, r) = { left }.split_at_mut(8);
        left = r;
        let chunk: [u8; 8] = rng.next_u64().to_le_bytes();
        l.copy_from_slice(&chunk);
    }
    let n = left.len();
    if n > 4 {
        let chunk: [u8; 8] = rng.next_u64().to_le_bytes();
        left.copy_from_slice(&chunk[..n]);
    } else if n > 0 {
        let chunk: [u8; 4] = rng.next_u32().to_le_bytes();
        left.copy_from_slice(&chunk[..n]);
    }
}

#[cfg(test)]
mod tests {
    use rand::RngCore;

    use crate::SquirrelRng;

    #[test]
    fn copy_with_position_does_not_modify_original() {
        let mut a = SquirrelRng::with_seed(3);
        let mut b = a.with_position(1);

        let second_value = b.next_u32();

        assert_ne!(a.next_u32(), second_value);
        assert_eq!(a.next_u32(), second_value);
    }
}

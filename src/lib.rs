use rand::{RngCore, SeedableRng};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SquirrelRng {
    position: u64,
    seed: u64,
}

impl SquirrelRng {
    pub fn new() -> Self {
        Self {
            position: 0,
            seed: rand::thread_rng().next_u64(),
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self { position: 0, seed }
    }

    pub fn with_position(mut self, position: u32) -> Self {
        self.position = position;
        self
    }
}

impl Default for SquirrelRng {
    fn default() -> Self {
        SquirrelRng::new()
    }
}

impl RngCore for SquirrelRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        let result = squirrel3(self.position, self.seed);
        self.position = self.position.wrapping_add(1);
        result
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
    type Seed = [u8; 8];

    fn from_seed(seed: Self::Seed) -> Self {
        Self::with_seed(u64::from_le_bytes(seed))
    }
}

#[inline]
pub fn squirrel3(position: u64, seed: u64) -> u64 {
    const BIT_NOISE1: u64 = 0xb333333333333027;
    const BIT_NOISE2: u64 = 0x6666666666666800;
    const BIT_NOISE3: u64 = 0x19999999999999eb;

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

// As implemented by rand.

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

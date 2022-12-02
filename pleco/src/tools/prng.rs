//! Contains the Pseudo-random number generator. Used for generating random `Board`s and
//! `BitBoard`s.

use std::mem::transmute;

/// Object for generating pseudo-random numbers.
pub struct PRNG {
    seed: u64,
}

impl PRNG {
    /// Creates PRNG from a seed.
    ///
    /// # Panics
    ///
    /// Undefined behavior if the seed is zero
    #[inline(always)]
    pub fn init(s: u64) -> PRNG {
        PRNG { seed: s }
    }

    /// Returns a pseudo-random number.
    #[allow(dead_code)]
    pub fn rand(&mut self) -> u64 {
        self.rand_change()
    }

    /// Returns a pseudo-random number with on average 8 bits being set.
    pub fn sparse_rand(&mut self) -> u64 {
        let mut s = self.rand_change();
        s &= self.rand_change();
        s &= self.rand_change();
        s
    }

    /// Returns a u64 with exactly one bit set in a random location.
    pub fn singular_bit(&mut self) -> u64 {
        let arr: [u8; 8] = unsafe { transmute(self.rand() ^ self.rand()) };
        let byte: u8 = arr.iter().fold(0, |acc, &x| acc ^ x);
        (1u64).wrapping_shl(((byte) >> 2) as u32)
    }

    /// Randomizes the current seed and returns a random value.
    fn rand_change(&mut self) -> u64 {
        self.seed ^= self.seed >> 12;
        self.seed ^= self.seed << 25;
        self.seed ^= self.seed >> 27;
        self.seed.wrapping_mul(2685_8216_5773_6338_717)
    }
}

#[cfg(test)]
mod test {
    use super::PRNG;

    const ROUNDS: u32 = 4;
    const MUTS: u32 = 32;

    #[test]
    fn check_bit_displacement() {
        let mut seeder = PRNG::init(10300014);
        let mut acc = [0u32; 64];
        for _ in 0..ROUNDS {
            let mut prng = PRNG::init(seeder.rand());
            for _ in 0..MUTS {
                add_to_bit_counts(prng.singular_bit(), &mut acc);
            }
        }

        let max = *acc.iter().max().unwrap();
        for (_i, m) in acc.iter_mut().enumerate() {
            *m *= 100;
            *m /= max;
            //println!("{} : {}",_i, m);
        }

        let _sum: u32 = acc.iter().sum();
        //        println!("avg: {}", _sum / 64);
    }

    fn add_to_bit_counts(mut num: u64, acc: &mut [u32; 64]) {
        while num != 0 {
            let i = num.trailing_zeros();
            acc[i as usize] += 1;
            num &= !((1u64) << i);
        }
    }
}

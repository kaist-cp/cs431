//! Utilities for random value generator

use rand::Rng;
use rand::distr::Alphanumeric;
use rand::rngs::ThreadRng;

/// Types that has random generator
pub trait RandGen {
    /// Randomly generates a value.
    fn rand_gen(rng: &mut ThreadRng) -> Self;
}

const KEY_MAX_LENGTH: usize = 4;

impl RandGen for String {
    fn rand_gen(rng: &mut ThreadRng) -> Self {
        let length = (rng.random::<u64>() as usize) % KEY_MAX_LENGTH;
        rng.sample_iter(&Alphanumeric)
            .take(length)
            .map(|x| x as char)
            .collect()
    }
}

impl RandGen for usize {
    /// pick only 16 bits, MSB=0
    fn rand_gen(rng: &mut ThreadRng) -> Self {
        const MASK: usize = 0x4004004004007777usize;
        (rng.random::<u64>() as usize) & MASK
    }
}

impl RandGen for u32 {
    /// pick only 16 bits
    fn rand_gen(rng: &mut ThreadRng) -> Self {
        const MASK: u32 = 0x66666666u32;
        rng.random::<u32>() & MASK
    }
}

impl RandGen for u8 {
    fn rand_gen(rng: &mut ThreadRng) -> Self {
        rng.random::<u8>()
    }
}

impl RandGen for () {
    fn rand_gen(_rng: &mut ThreadRng) -> Self {}
}

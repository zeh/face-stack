// Originally from https://github.com/zeh/random-art-generator/blob/main/src/generator/utils/random/rng.rs

use getrandom;

pub struct Rng {
	seed: u32,
	value: u32,
}

impl Rng {
	/// Generate a new Prando pseudo-random number generator. Uses a pseudo-random seed.
	pub fn new() -> Rng {
		let seed = getrandom::u32().expect("Generating seed");
		Rng::from_seed(seed)
	}

	/// Generate a new Prando pseudo-random number generator.
	///
	/// @param seed - A number that determines which pseudo-random number sequence will be created.
	pub fn from_seed(seed: u32) -> Rng {
		let mut rng = Rng {
			seed,
			value: 0,
		};
		rng.reset();
		rng
	}

	#[inline(always)]
	fn xorshift(mut value: u32) -> u32 {
		// Xorshift*32
		// Based on George Marsaglia's work: http://www.jstatsoft.org/v08/i14/paper
		value ^= value.wrapping_shl(13);
		value ^= value.wrapping_shr(17);
		value ^= value.wrapping_shl(5);
		value
	}

	#[inline(always)]
	fn recalculate(&mut self) {
		self.value = Rng::xorshift(self.value);
	}

	/// Reset the pseudo-random number sequence back to its starting seed. Further calls to next()
	/// will then produce the same sequence of numbers it had produced before. This is equivalent to
	/// creating a new instance with the same seed as another Prando instance.
	///
	/// Example:
	/// let rng = Rng::new_from_seed(12345678);
	/// println!(rng.next()); // 0.6177754114889017
	/// println!(rng.next()); // 0.5784605181725837
	/// rng.reset();
	/// println!(rng.next()); // 0.6177754114889017 again
	/// println!(rng.next()); // 0.5784605181725837 again
	pub fn reset(&mut self) {
		self.value = self.seed;
	}

	/// Skips ahead in the sequence of numbers that are being generated. This is equivalent to
	/// calling next() a specified number of times, but faster since it doesn't need to map the
	/// new random numbers to a range and return it.
	#[allow(dead_code)]
	pub fn skip(&mut self, mut iterations: u32) {
		while iterations > 0 {
			self.recalculate();
			iterations -= 1;
		}
	}

	/// Generates a pseudo-random number between 0 (inclusive) and u32 max (exclusive).
	///
	/// @return The generated pseudo-random number.
	pub fn next(&mut self) -> u32 {
		self.recalculate();
		self.value
	}

	/// Generates a pseudo-random number between a lower (inclusive) and a higher (exclusive) bounds.
	///
	/// @param min - The minimum number that can be randomly generated.
	/// @param pseudo_max - The maximum number that can be randomly generated (exclusive).
	/// @return The generated pseudo-random number.
	#[allow(dead_code)]
	pub fn next_u32_range(&mut self, min: u32, pseudo_max: u32) -> u32 {
		self.next_f64_range(min as f64, pseudo_max as f64) as u32
	}

	/// Generates a pseudo-random number between 0 (inclusive) and 1 (exclusive).
	///
	/// @return The generated pseudo-random number.
	#[allow(dead_code)]
	pub fn next_f64(&mut self) -> f64 {
		self.next() as f64 / (!0u32 as f64)
	}

	/// Generates a pseudo-random number between a lower (inclusive) and a higher (exclusive) bounds.
	///
	/// @param min - The minimum number that can be randomly generated.
	/// @param pseudo_max - The maximum number that can be randomly generated (exclusive).
	/// @return The generated pseudo-random number.
	#[allow(dead_code)]
	pub fn next_f64_range(&mut self, min: f64, pseudo_max: f64) -> f64 {
		if min == pseudo_max {
			return min;
		}
		self.next_f64() * (pseudo_max - min) + min
	}

	/// Generates a pseudo-random boolean.
	///
	/// @return A value of true or false.
	#[allow(dead_code)]
	pub fn next_bool(&mut self) -> bool {
		self.next_f64() > 0.5f64
	}
}

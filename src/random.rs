// Originally from https://github.com/zeh/random-art-generator/blob/main/src/generator/utils/random/mod.rs

use crate::rng::Rng;
use crate::units::WeightedValue;

#[inline(always)]
fn get_random_range(rng: &mut Rng, min: f64, pseudo_max: f64) -> f64 {
	rng.next_f64_range(min, pseudo_max)
}

fn get_random_entry_weighted<'a, T>(rng: &mut Rng, entries: &'a Vec<WeightedValue<T>>) -> &'a T {
	let total_weight = entries.iter().map(|r| r.weight).sum();
	let desired_position = get_random_range(rng, 0.0, total_weight);
	let mut acc = 0.0f64;
	&entries
		.iter()
		.find(|&r| {
			acc += r.weight;
			acc >= desired_position
		})
		.expect("finding weighted random value")
		.value
}

pub fn get_random_range_weighted(rng: &mut Rng, ranges: &Vec<WeightedValue<(f64, f64)>>) -> f64 {
	let range = get_random_entry_weighted(rng, ranges);
	get_random_range(rng, range.0, range.1)
}

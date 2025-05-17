// Originally from https://github.com/zeh/random-art-generator/blob/main/src/generator/utils/units.rs
#[derive(Clone, Debug, PartialEq)]
pub enum SizeUnit {
	Fraction(f64),
	Pixels(i64),
}

impl SizeUnit {
	pub fn to_pixels(&self, total_size: u32) -> i64 {
		match self {
			Self::Fraction(value) => (*value * total_size as f64).round() as i64,
			Self::Pixels(value) => *value,
		}
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeightedValue<T> {
	pub value: T,
	pub weight: f64,
}

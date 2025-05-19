// Originally (partly) from https://github.com/zeh/random-art-generator/blob/main/src/generator/utils/parsing.rs
use std::str::FromStr;

use crate::{
	blending::BlendingMode,
	units::{SizeUnit, WeightedValue},
};

fn parse_integer(src: &str) -> Result<u32, &str> {
	src.parse::<u32>().or(Err("Could not parse integer value"))
}

fn parse_integer_list(src: &str, divider: char) -> Result<Vec<u32>, &str> {
	src.split(divider).collect::<Vec<&str>>().iter().map(|&e| parse_integer(e)).collect()
}

/// Parses a dimensions string (999x999) into a (u32, u32) width/height tuple.
pub fn parse_image_dimensions(src: &str) -> Result<(u32, u32), &str> {
	let values = parse_integer_list(&src, 'x')?;
	match values.len() {
		2 => Ok((values[0], values[1])),
		_ => Err("Dimensions should use WIDTHxHEIGHT"),
	}
}

fn parse_float(src: &str) -> Result<f64, &str> {
	src.parse::<f64>().or(Err("Could not parse float value"))
}

fn parse_float_list(src: &str, divider: char) -> Result<Vec<f64>, &str> {
	src.split(divider).collect::<Vec<&str>>().iter().map(|&e| parse_float(e)).collect()
}

/// Parses "1.0", "0.9-1.0" into (1.0, 1.0), (0.9, 1.0)
fn parse_float_pair(src: &str) -> Result<(f64, f64), &str> {
	let values = parse_float_list(&src, '-')?;
	match values.len() {
		1 => Ok((values[0], values[0])),
		2 => Ok((values[0], values[1])),
		_ => Err("Float range must be 1-2"),
	}
}

fn parse_size(src: &str) -> Result<SizeUnit, &str> {
	if src.ends_with("%") {
		match src[..src.len() - 1].parse::<f64>() {
			Ok(value) => Ok(SizeUnit::Fraction(value / 100.0f64)),
			_ => Err("Could not parse fraction value"),
		}
	} else {
		match src.parse::<f64>() {
			Ok(value) => Ok(SizeUnit::Pixels(value.round() as i64)),
			_ => Err("Could not parse pixel value"),
		}
	}
}

fn parse_size_list(src: &str, divider: char) -> Result<Vec<SizeUnit>, &str> {
	src.split(divider).collect::<Vec<&str>>().iter().map(|&e| parse_size(e)).collect()
}

// Parses "100%", "90%-100%", "10-20", "2" into pairs of SizeUnits
fn parse_size_pair(src: &str) -> Result<(SizeUnit, SizeUnit), &str> {
	let values = parse_size_list(&src, '-')?;
	match values.len() {
		1 => Ok((values[0].clone(), values[0].clone())),
		2 => Ok((values[0].clone(), values[1].clone())),
		_ => Err("Size range length must be 2"),
	}
}

/// Parses "*@n" into a string "*" with n weight. This is used so we can have pairs with weights.
fn parse_weight(src: &str) -> Result<(&str, f64), &str> {
	let values = src.split('@').collect::<Vec<&str>>();
	match values.len() {
		1 => Ok((src, 1.0)),
		2 => match parse_float(values[1]) {
			Ok(val) => Ok((values[0], val)),
			Err(err) => Err(err),
		},
		_ => Err("Value cannot contain more than one weight value"),
	}
}

/// Parses a size pair with a weight (e.f. "1-2@1", "10%", "5-10%@2") into a WeightedValue<>
pub fn parse_weighted_size_pair(src: &str) -> Result<WeightedValue<(SizeUnit, SizeUnit)>, &str> {
	match parse_weight(src) {
		Ok((src_value, weight)) => match parse_size_pair(src_value) {
			Ok(value) => Ok(WeightedValue {
				value,
				weight,
			}),
			Err(err) => Err(err),
		},
		Err(err) => Err(err),
	}
}

/// Parses a float pair with a weight (e.f. "1-2@1", "10.2", "5.2-10@2") into a WeightedValue<>
pub fn parse_weighted_float_pair(src: &str) -> Result<WeightedValue<(f64, f64)>, &str> {
	match parse_weight(src) {
		Ok((src_value, weight)) => match parse_float_pair(src_value) {
			Ok(value) => Ok(WeightedValue {
				value,
				weight,
			}),
			Err(err) => Err(err),
		},
		Err(err) => Err(err),
	}
}

/// Parses a blending mode with a weight (e.g. "normal", "screen@2") into a WeightedValue<>
pub fn parse_weighted_blending_mode(src: &str) -> Result<WeightedValue<BlendingMode>, &str> {
	match parse_weight(src) {
		Ok((src_value, weight)) => match BlendingMode::from_str(src_value) {
			Ok(value) => Ok(WeightedValue {
				value,
				weight,
			}),
			Err(_) => Err("Cannot parse value variant for blending mode"),
		},
		Err(err) => Err(err),
	}
}

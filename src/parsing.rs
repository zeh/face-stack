// Originally (partly) from https://github.com/zeh/random-art-generator/blob/main/src/generator/utils/parsing.rs

use crate::units::WeightedValue;

/// Parses a dimensions string (999x999) into a (u32, u32) width/height tuple.
pub fn parse_image_dimensions(s: &str) -> Result<(u32, u32), String> {
	let parts: Vec<&str> = s.split('x').collect();
	if parts.len() != 2 {
		return Err("Invalid image dimensions; use WIDTHxHEIGHT".to_string());
	}
	let width = parts[0].parse::<u32>().map_err(|_| "Invalid width")?;
	let height = parts[1].parse::<u32>().map_err(|_| "Invalid height")?;
	Ok((width, height))
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

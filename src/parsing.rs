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

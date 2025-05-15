/**
 * Find the expected scale to fit a rectangle (w, h) inside another.
 */
pub fn fit_inside(outside_rect: (f32, f32), inside_rect: (f32, f32)) -> (f32, f32) {
	let inside_ar = inside_rect.0 / inside_rect.1;
	let outside_ar = outside_rect.0 / outside_rect.1;
	if inside_ar > outside_ar {
		// Inside rect width is "wider" than outside: fit by its width
		(outside_rect.0, outside_rect.0 / inside_ar)
	} else {
		// Inside rect width is "taller" than outside: fit by its height
		(outside_rect.1 * inside_ar, outside_rect.1)
	}
}

pub type XYf = (f32, f32);
pub type WHf = (f32, f32);
pub type XYWHf = (f32, f32, f32, f32);

pub type XYi = (i32, i32);
pub type WHi = (u32, u32);
pub type XYWHi = (i32, i32, u32, u32);

/**
 * Find the expected scale to fit a rectangle (w, h) inside another.
 */
pub fn fit_inside(outside_rect: WHf, inside_rect: WHf) -> WHf {
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

pub fn xyf_to_xyi(xy: XYf) -> XYi {
	(xy.0.round() as i32, xy.1.round() as i32)
}

pub fn whf_to_whi(wh: WHf) -> WHi {
	(wh.0.round() as u32, wh.1.round() as u32)
}

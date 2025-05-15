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

/**
 * Find the intersection rectangle between two rectangles
 */
pub fn intersect(rect1: XYWHf, rect2: XYWHf) -> Option<XYWHf> {
	assert!(rect1.2 >= 0.0);
	assert!(rect1.3 >= 0.0);
	assert!(rect2.2 >= 0.0);
	assert!(rect2.3 >= 0.0);
	let xyxy1 = (rect1.0, rect1.1, rect1.0 + rect1.2, rect1.1 + rect1.3);
	let xyxy2 = (rect2.0, rect2.1, rect2.0 + rect2.2, rect2.1 + rect2.3);

	let xyxyi = (xyxy1.0.max(xyxy2.0), xyxy1.1.max(xyxy2.1), xyxy1.2.min(xyxy2.2), xyxy1.3.min(xyxy2.3));

	if xyxyi.0 > xyxyi.2 || xyxyi.1 > xyxyi.3 {
		None
	} else {
		Some((xyxyi.0, xyxyi.1, xyxyi.2 - xyxyi.0, xyxyi.3 - xyxyi.1))
	}
}

pub fn xyf_to_xyi(xy: XYf) -> XYi {
	(xy.0.round() as i32, xy.1.round() as i32)
}

pub fn xyi_to_xyf(xy: XYi) -> XYf {
	(xy.0 as f32, xy.1 as f32)
}

pub fn whf_to_whi(wh: WHf) -> WHi {
	(wh.0.round() as u32, wh.1.round() as u32)
}

pub fn xywhi_to_xywhf(xywh: XYWHi) -> XYWHf {
	(xywh.0 as f32, xywh.1 as f32, xywh.2 as f32, xywh.3 as f32)
}

pub fn xywhf_to_xywhi(xywh: XYWHf) -> XYWHi {
	(xywh.0.round() as i32, xywh.1.round() as i32, xywh.2.round() as u32, xywh.3.round() as u32)
}

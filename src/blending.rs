// From https://github.com/zeh/random-art-generator/blob/main/src/generator/utils/color.rs

use strum_macros::{Display, EnumString};

#[derive(Clone, Debug, Display, EnumString, PartialEq)]
pub enum BlendingMode {
	#[strum(serialize = "normal")]
	Normal,
	#[strum(serialize = "multiply")]
	Multiply,
	#[strum(serialize = "screen")]
	Screen,
	#[strum(serialize = "overlay")]
	Overlay,
	#[strum(serialize = "darken")]
	Darken,
	#[strum(serialize = "lighten")]
	Lighten,
	#[strum(serialize = "color-dodge")]
	ColorDodge,
	#[strum(serialize = "color-burn")]
	ColorBurn,
	#[strum(serialize = "hard-light")]
	HardLight,
	#[strum(serialize = "soft-light")]
	SoftLight,
	#[strum(serialize = "difference")]
	Difference,
	#[strum(serialize = "exclusion")]
	Exclusion,
}

impl BlendingMode {
	#[inline(always)]
	pub fn blend(&self, bottom: f32, top: f32) -> f32 {
		match self {
			Self::Normal => top,
			Self::Multiply => bottom * top,
			Self::Screen => 1.0 - (1.0 - bottom) * (1.0 - top),
			Self::Overlay => {
				if bottom < 0.5 {
					2.0 * bottom * top
				} else {
					1.0 - 2.0 * (1.0 - bottom) * (1.0 - top)
				}
			}
			Self::Darken => bottom.min(top),
			Self::Lighten => bottom.max(top),
			Self::ColorDodge => {
				if bottom == 0.0 {
					0.0
				} else if top == 1.0 {
					1.0
				} else {
					(bottom / (1.0 - top)).min(1.0)
				}
			}
			Self::ColorBurn => {
				if bottom == 1.0 {
					1.0
				} else if top == 0.0 {
					0.0
				} else {
					1.0 - ((1.0 - bottom) / top).min(1.0)
				}
			}
			Self::HardLight => {
				if top <= 0.5 {
					2.0 * bottom * top
				} else {
					1.0 - (1.0 - bottom) * (1.0 - (2.0 * top - 1.0))
				}
			}
			Self::SoftLight => {
				if top <= 0.5 {
					bottom - (1.0 - 2.0 * top) * bottom * (1.0 - bottom)
				} else {
					let d = if bottom <= 0.25 {
						((16.0 * bottom - 12.0) * bottom + 4.0) * bottom
					} else {
						bottom.sqrt()
					};
					bottom + (2.0 * top - 1.0) * (d - bottom)
				}
			}
			Self::Difference => (bottom - top).abs().max(0.0).min(1.0),
			Self::Exclusion => bottom + top - 2.0 * bottom * top,
		}
	}

	/// Interpolates between the bottom color, and the resulting
	/// color if the top color was applied with this blend mode
	#[inline(always)]
	pub fn blend_with_opacity(&self, bottom: f32, top: f32, opacity: f32) -> f32 {
		return if opacity == 0.0 {
			bottom
		} else {
			let opaque_result = &self.blend(bottom, top);
			opaque_result * opacity + bottom * (1.0 - opacity)
		};
	}
}

impl Default for BlendingMode {
	fn default() -> Self {
		BlendingMode::Normal
	}
}

#[inline(always)]
pub fn blend_pixel(bottom: &[f32], top: &[f32], opacity: f32, blending_mode: &BlendingMode) -> [f32; 3] {
	if opacity == 0.0 {
		[bottom[0], bottom[1], bottom[2]]
	} else {
		[
			blending_mode.blend_with_opacity(bottom[0], top[0], opacity),
			blending_mode.blend_with_opacity(bottom[1], top[1], opacity),
			blending_mode.blend_with_opacity(bottom[2], top[2], opacity),
		]
	}
}

#[inline(always)]
pub fn channel_u8_to_f32(color: u8) -> f32 {
	color as f32 / 255.0
}

#[inline(always)]
pub fn pixel_u8_to_f32(colors: &[u8; 3]) -> [f32; 3] {
	[channel_u8_to_f32(colors[0]), channel_u8_to_f32(colors[1]), channel_u8_to_f32(colors[2])]
}

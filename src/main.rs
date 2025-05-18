use std::path::PathBuf;

use glob::{GlobError, glob};
use image::{ImageBuffer, Pixel, Rgb, Rgb32FImage, RgbImage, imageops};
use rng::Rng;
use rust_faces::{
	BlazeFaceParams, FaceDetection, FaceDetectorBuilder, InferParams, Provider, ToArray3, ToRgb8,
};
use structopt::StructOpt;

use blending::{BlendingMode, blend_pixel, pixel_u8_to_f32};
use geom::{WHf, WHi, XYWHi, XYi, fit_inside, intersect, whf_to_whi, xyf_to_xyi};
use parsing::{
	parse_image_dimensions, parse_weighted_blending_mode, parse_weighted_float_pair, parse_weighted_size_pair,
};
use random::{get_random_entry_weighted, get_random_range_weighted, get_random_size_range_weighted};
use units::{SizeUnit, WeightedValue};

pub mod blending;
pub mod geom;
pub mod parsing;
pub mod random;
pub mod rng;
pub mod terminal;
pub mod units;

/**
 * Copy one image on top of another
 */
fn blend_image(
	bottom: &mut Rgb32FImage,
	top: &RgbImage,
	top_offset: XYi,
	opacity: f32,
	blending_mode: &BlendingMode,
	mask: Option<XYWHi>,
) {
	// Find paintable intersection between bottom and top
	let bottom_rect = (0, 0, bottom.width(), bottom.height());
	let top_rect = (top_offset.0, top_offset.1, top.width(), top.height());
	let intersection = intersect(bottom_rect, top_rect);
	if intersection.is_none() {
		panic!("Cannot blend image; no intersection between bottom and top image.");
	}
	let mut intersection_rect = intersection.unwrap();

	// Applies further intersection if a mask is present
	if let Some(mask) = mask {
		let mask_intersection = intersect(intersection.unwrap(), mask);
		if mask_intersection.is_none() {
			// panic!("Cannot blend image; no intersection between blended and mask.");
			return;
		}
		intersection_rect = mask_intersection.unwrap();
	}

	let dst_x1 = intersection_rect.0;
	let dst_y1 = intersection_rect.1;
	let dst_x2 = intersection_rect.0 + intersection_rect.2 as i32 - 1;
	let dst_y2 = intersection_rect.1 + intersection_rect.3 as i32 - 1;

	for dst_y in dst_y1..dst_y2 {
		let src_y = (dst_y - top_offset.1) as u32;
		for dst_x in dst_x1..dst_x2 {
			let src_x = (dst_x - top_offset.0) as u32;
			let bottom_px: [f32; 3] = bottom
				.get_pixel(dst_x as u32, dst_y as u32)
				.channels()
				.to_owned()
				.try_into()
				.expect("converting pixels to array");
			let top_px: [u8; 3] = top
				.get_pixel(src_x, src_y)
				.channels()
				.to_owned()
				.try_into()
				.expect("converting pixels to array");
			let blended = blend_pixel(&bottom_px, &pixel_u8_to_f32(&top_px), opacity, blending_mode);
			bottom.put_pixel(dst_x as u32, dst_y as u32, Rgb(blended));
		}
	}
}

#[derive(Debug, StructOpt)]
#[structopt(name = "face-stack", about = "Stacks face-aligned images.")]
struct Opt {
	/// File mask (e.g., "images/*.jpg")
	#[structopt(long, default_value = "*.jpg")]
	input: String,

	/// Output image dimensions (e.g., "800x600")
	#[structopt(long, default_value = "1024x1024", parse(try_from_str = parse_image_dimensions))]
	size: (u32, u32),

	/// Scale of the face (e.g., "0.5")
	#[structopt(long, default_value = "1")]
	face_scale: f32,

	/// Output file name (e.g., "output.png")
	#[structopt(long, default_value = "face-stack-output.jpg", parse(from_os_str))]
	output: PathBuf,

	/// The seed to use for the pseudorandom number generator, between `1` and `4294967295`
	#[structopt(long, default_value = "0")]
	seed: u32,

	/// Opacity for each new layer when blending images
	#[structopt(long, default_value = "0.5", parse(try_from_str = parse_weighted_float_pair))]
	opacity: Vec<WeightedValue<(f64, f64)>>,

	/// Width for the crop rectangle of new blended layes
	#[structopt(long, default_value = "0%-100%", parse(try_from_str = parse_weighted_size_pair))]
	crop_width: Vec<WeightedValue<(SizeUnit, SizeUnit)>>,

	/// Height for the crop rectangle of new blended layes
	#[structopt(long, default_value = "0%-100%", parse(try_from_str = parse_weighted_size_pair))]
	crop_height: Vec<WeightedValue<(SizeUnit, SizeUnit)>>,

	/// Blending mode(s) to be used when overlaying images
	/// Possible values: `normal`, `multiply`, `screen`, `overlay`, `darken`, `lighten`, `color-dodge`, `color-burn`, `hard-light`, `soft-light`, `difference`, `exclusion`
	#[structopt(long, default_value = "normal", default_value = "normal", parse(try_from_str = parse_weighted_blending_mode))]
	blending_mode: Vec<WeightedValue<BlendingMode>>,

	/// Number of maximum valid images to use for input
	#[structopt(long, default_value = "0")]
	max_images: u32,
}

fn main() {
	let opt = Opt::from_args();
	let (target_width, target_height) = opt.size;

	println!(
		"Will get files from {:?}, at size {}x{}, and output at {:?}.",
		opt.input, target_width, target_height, opt.output
	);

	let face_detector =
        // Alternative:
        // FaceDetectorBuilder::new(FaceDetection::MtCnn(
        //     MtCnnParams {
        //         min_face_size: 1000,
        //         ..Default::default()
        //     }))
        FaceDetectorBuilder::new(FaceDetection::BlazeFace640(
            BlazeFaceParams {
                // Default is 1280, but finds no images
                // 80 works too
                target_size: 160,
                ..Default::default()
            }))
            .download()
            .infer_params(InferParams {
                provider: Provider::OrtCpu,
                intra_threads: Some(5),
                ..Default::default()
            })
            .build()
            .expect("Failed to load the face detector");

	// Decide where the face will be in the output image
	let typical_face_size: WHf = (75f32, 100f32); // Typically 0.75 aspect ratio
	let faces_rect_inside = fit_inside((target_width as f32, target_height as f32), typical_face_size);
	let typical_face_scale = 0.6f32 * opt.face_scale;
	let target_faces_rect: WHf =
		(faces_rect_inside.0 * typical_face_scale, faces_rect_inside.1 * typical_face_scale);

	// Create the output image
	let mut output_image: Rgb32FImage =
		ImageBuffer::from_pixel(target_width, target_height, Rgb([0.5, 0.5, 0.5]));
	let mut num_images_used = 0usize;
	let mut num_images_read = 0usize;

	// Creates a random number generator to be used for deterministic randomization
	let rng_seed = if opt.seed == 0 {
		Rng::new().next()
	} else {
		// Seeds close to each other produce very similar results, so we multiply them a bit
		opt.seed.wrapping_add(Rng::from_seed(1337).next())
	};
	let mut rng = Rng::from_seed(rng_seed);

	// Reads all images from the given input mask
	let image_files = glob(&opt.input)
		.expect(format!("Failed to read glob pattern: {}", opt.input).as_str())
		.collect::<Vec<Result<PathBuf, GlobError>>>();

	for image_file in &image_files {
		if let Ok(path) = image_file {
			// File can be opened
			terminal::erase_line_to_end();
			print!(
				"({}/{}) Reading {:?}",
				num_images_read + 1,
				image_files.len(),
				&path.file_name().unwrap()
			);

			if let Ok(img) = image::open(&path) {
				// Is a valid image file
				print!(", {:?}x{:?}", img.width(), img.height());
				let array3_image = img.into_rgb8().into_array3();
				let faces = face_detector.detect(array3_image.view().into_dyn()).unwrap();
				let rgb_image = array3_image.to_rgb8();
				print!(", {} faces", faces.len());

				if faces.len() == 1 {
					// Has a valid face
					println!(", confidence {:?}", faces[0].confidence);

					let face_rect = &faces[0].rect;

					// Find out what the face size should be inside our face target box
					let target_face_rect: WHf =
						fit_inside(target_faces_rect, (face_rect.width, face_rect.height));
					let new_image_scale = target_face_rect.0 / face_rect.width;
					let new_image_size: WHi = whf_to_whi((
						rgb_image.width() as f32 * new_image_scale,
						rgb_image.height() as f32 * new_image_scale,
					));

					// Scale the image appropriately
					let resized_image =
						imageops::resize(&rgb_image, new_image_size.0, new_image_size.1, imageops::Lanczos3);

					// Get all the options
					let param_opacity = get_random_range_weighted(&mut rng, &opt.opacity) as f32;
					let param_crop_rect = {
						let crop_width =
							get_random_size_range_weighted(&mut rng, &opt.crop_width, target_width).round()
								as u32;
						let crop_height =
							get_random_size_range_weighted(&mut rng, &opt.crop_height, target_height).round()
								as u32;
						(
							rng.next_u32_range(0, target_width - crop_width) as i32,
							rng.next_u32_range(0, target_height - crop_height) as i32,
							crop_width,
							crop_height,
						)
					};
					let param_blending_mode = get_random_entry_weighted(&mut rng, &opt.blending_mode);
					let param_offset: XYi = xyf_to_xyi((
						target_width as f32 / 2.0 - (face_rect.x + face_rect.width / 2.0) * new_image_scale,
						target_height as f32 / 2.0 - (face_rect.y + face_rect.height / 2.0) * new_image_scale,
					));

					// Finally, blend it all
					blend_image(
						&mut output_image,
						&resized_image,
						param_offset,
						param_opacity,
						param_blending_mode,
						Some(param_crop_rect),
					);

					num_images_used += 1;

					terminal::cursor_up();
				} else {
					println!("; no valid faces, skipping.");
				}
			} else {
				println!("; invalid image, skipping.");
			}
		}

		num_images_read += 1;

		if opt.max_images > 0 && num_images_read >= opt.max_images as usize {
			terminal::erase_line_to_end();
			println!("Reached the maximum number of input images; skipping additional files.");
			break;
		}
	}

	terminal::erase_line_to_end();
	println!("Done. {} images processed, with {} valid images used.", image_files.len(), num_images_used);

	// Convert the output image from Rgb-32f to Rgb-u8
	let mut output_u8 = RgbImage::new(output_image.width(), output_image.height());
	{
		for (x, y, pixel) in output_image.enumerate_pixels() {
			let scaled = pixel.0.map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8);
			output_u8.put_pixel(x, y, Rgb(scaled));
		}
	}

	// Finally, saved the final image
	output_u8.save(&opt.output).expect("Failed to save output image");
}

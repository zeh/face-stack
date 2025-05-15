use std::path::PathBuf;

use glob::{GlobError, glob};
use image::{ImageBuffer, Pixel, Rgb, RgbImage, imageops};
use rust_faces::{
	BlazeFaceParams, FaceDetection, FaceDetectorBuilder, InferParams, Provider, ToArray3, ToRgb8,
};
use structopt::StructOpt;

use blending::{BlendingMode, blend_pixel};
use geom::fit_inside;

pub mod blending;
pub mod geom;
pub mod terminal;

/**
 * Parses a dimensions string (999x999) into a (u32, u32) width/height tuple.
 */
fn parse_image_dimensions(s: &str) -> Result<(u32, u32), String> {
	let parts: Vec<&str> = s.split('x').collect();
	if parts.len() != 2 {
		return Err("Invalid image dimensions; use WIDTHxHEIGHT".to_string());
	}
	let width = parts[0].parse::<u32>().map_err(|_| "Invalid width")?;
	let height = parts[1].parse::<u32>().map_err(|_| "Invalid height")?;
	Ok((width, height))
}

/**
 * Copy one image on top of another
 */
fn blend_image(bottom: &mut RgbImage, top: &RgbImage, offset: (i32, i32), opacity: f64) {
	let src_x1 = if offset.0 < 0 {
		-offset.0 as u32
	} else {
		0
	};
	let src_x2 = top.width().min((bottom.width() as i32 - offset.0) as u32) - 1;
	let src_y1 = if offset.1 < 0 {
		-offset.1 as u32
	} else {
		0
	};
	let src_y2 = top.height().min((bottom.height() as i32 - offset.1) as u32) - 1;

	for src_y in src_y1..src_y2 {
		let dst_y = (src_y as i32 + offset.1) as u32;
		for src_x in src_x1..src_x2 {
			let dst_x = (src_x as i32 + offset.0) as u32;
			let bottom_px: [u8; 3] = bottom
				.get_pixel(dst_x, dst_y)
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
			let blended = blend_pixel(
				&[bottom_px[0], bottom_px[1], bottom_px[2]],
				&[top_px[0], top_px[1], top_px[2]],
				opacity,
				&BlendingMode::Normal,
			);
			bottom.put_pixel(dst_x, dst_y, Rgb(blended));
		}
	}
}

#[derive(Debug, StructOpt)]
#[structopt(name = "face-align-composite", about = "Composite face-aligned images.")]
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
	let typical_face_size = (75f32, 100f32); // Typically 0.75 aspect ratio
	let faces_rect_inside = fit_inside((target_width as f32, target_height as f32), typical_face_size);
	let typical_face_scale = 0.6f32 * opt.face_scale;
	let target_faces_rect =
		(faces_rect_inside.0 * typical_face_scale, faces_rect_inside.1 * typical_face_scale);

	// Create the output image
	let mut output_image: ImageBuffer<Rgb<u8>, Vec<u8>> =
		ImageBuffer::from_pixel(target_width, target_height, Rgb([127, 127, 127]));
	let mut num_images_used = 0usize;
	let mut num_images_read = 0usize;

	// Reads all images from the given input mask
	let image_files = glob(&opt.input)
		.expect(format!("Failed to read glob pattern: {}", opt.input).as_str())
		.collect::<Vec<Result<PathBuf, GlobError>>>();

	for image_file in &image_files {
		if let Ok(path) = image_file {
			// File can be opened
			terminal::erase_line_to_end();
			print!("({}/{}) Reading {:?}", num_images_read + 1, image_files.len(), &path);

			if let Ok(img) = image::open(&path) {
				// Is a valid image file
				print!(", size {:?}x{:?}", img.width(), img.height());
				let array3_image = img.into_rgb8().into_array3();
				let faces = face_detector.detect(array3_image.view().into_dyn()).unwrap();
				let rgb_image = array3_image.to_rgb8();
				print!(", {} faces", faces.len());

				if faces.len() == 1 {
					// Has a valid face
					println!(", confidence {:?}", faces[0].confidence);

					let face_rect = &faces[0].rect;

					// Find out what the face size should be inside our face target box
					let target_face_rect = fit_inside(target_faces_rect, (face_rect.width, face_rect.height));
					let new_image_scale = target_face_rect.0 / face_rect.width;
					let new_image_size = (
						(rgb_image.width() as f32 * new_image_scale).round() as u32,
						(rgb_image.height() as f32 * new_image_scale).round() as u32,
					);

					// Scale the image appropriately
					let resized_image =
						imageops::resize(&rgb_image, new_image_size.0, new_image_size.1, imageops::Lanczos3);

					// Finally, blend it all
					let offset = (
						target_width as f32 / 2.0 - (face_rect.x + face_rect.width / 2.0) * new_image_scale,
						target_height as f32 / 2.0 - (face_rect.y + face_rect.height / 2.0) * new_image_scale,
					);
					let offset_i32 = (offset.0.round() as i32, offset.1.round() as i32);
					blend_image(
						&mut output_image,
						&resized_image,
						offset_i32,
						1f64 / (num_images_used as f64 + 1f64),
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
	}

	terminal::erase_line_to_end();
	println!("Done. {} images processed, with {} valid images used.", image_files.len(), num_images_used);

	// Finally, saved the final image
	output_image.save(&opt.output).expect("Failed to save output image");
}

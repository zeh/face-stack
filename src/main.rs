use std::path::PathBuf;

use glob::glob;
use image::{ImageBuffer, Pixel, Rgb, RgbImage};
use rust_faces::{BlazeFaceParams, FaceDetection, FaceDetectorBuilder, InferParams, Provider, ToArray3, ToRgb8};
use structopt::StructOpt;

use blending::{blend_pixel, BlendingMode};

pub mod blending;

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
fn blend_image(bottom: &mut RgbImage, top: &RgbImage, offset: (i32, i32)) {
    let src_x1 = if offset.0 < 0 { -offset.0 as u32 } else { 0 };
    let src_x2 = top.width().min(bottom.width() - (offset.0 as u32));
    let src_y1 = if offset.1 < 0 { -offset.1 as u32 } else { 0 };
    let src_y2 = top.height().min(bottom.height() - (offset.1 as u32));

    for src_y in src_y1..src_y2 {
        let dst_y = (src_y as i32 + offset.1) as u32;
        for src_x in src_x1..src_x2 {
            let bottom_px: [u8; 3] = bottom.get_pixel(src_x, src_y).channels().to_owned().try_into().expect("converting pixels to array");
            let top_px: [u8; 3] = top.get_pixel(src_x, src_y).channels().to_owned().try_into().expect("converting pixels to array");
            let blended = blend_pixel(&[bottom_px[0], bottom_px[1], bottom_px[2]], &[top_px[0], top_px[1], top_px[2]], 0.5, &BlendingMode::Normal);
            bottom.put_pixel((src_x as i32 + offset.0) as u32, dst_y, Rgb(blended));
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

    /// Output file name (e.g., "output.png")
    #[structopt(long, default_value = "face-stack-output.jpg", parse(from_os_str))]
    output: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let (target_width, target_height) = opt.size;

    println!("Will get files from {:?}, at size {}x{}, and output at {:?}.", opt.input, target_width, target_height, opt.output);

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

    let mut output_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(target_width, target_height, Rgb([0, 0, 0]));
    let mut num_images_used = 0;

    // Reads all images from the given input mask
    for entry in glob(&opt.input).expect(format!("Failed to read glob pattern: {}", opt.input).as_str()) {
        if let Ok(path) = entry {
            println!("Reading {:?}", &path);
            if let Ok(img) = image::open(&path) {
                let array3_image = img.into_rgb8().into_array3();
                let faces = face_detector.detect(array3_image.view().into_dyn()).unwrap();

                let rgb_image = array3_image.to_rgb8();
                println!("...size: {:?}x{:?}, faces: {}", &rgb_image.width(), &rgb_image.height(), faces.len());

                if faces.len() == 1 {
                    println!("...face: {:?}", faces[0].rect.to_xywh());
                    blend_image(&mut output_image, &rgb_image, (num_images_used * 20, num_images_used * 20));
                    num_images_used += 1;
                }
            }
        }
    }

    // Finally, saved the final image
    output_image.save(&opt.output).expect("Failed to save output image");
}

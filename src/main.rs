use std::path::PathBuf;

use glob::glob;
use image::{ImageBuffer, Rgb};
use structopt::StructOpt;


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

    let mut output_image: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(target_width, target_height, Rgb([0, 0, 0]));

    // Reads all images from the given input mask
    for entry in glob(&opt.input).expect(format!("Failed to read glob pattern: {}", opt.input).as_str()) {
        if let Ok(path) = entry {
            println!("Reading {:?}", &path);
        }
    }
}

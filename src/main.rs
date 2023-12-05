use argh::FromArgs;
use fpc::*;
use image::{open, Rgba};

/// Find playing cards in a source image.
/// TODO: make this better.
#[derive(FromArgs, Debug)]
struct Args {
    /// desired aspect ratio for output images
    #[argh(option, default = "1.4f64")]
    aspect_ratio: f64,

    /// the background color for the output images
    #[argh(option, default = "String::from(\"white\")")]
    background_color: String,

    /// the maximum width of the output images
    #[argh(option, default = "750")]
    max_width: u32,

    #[argh(positional)]
    input_images: Vec<String>,
}

fn main() -> fpc::Result<()> {
    let args: Args = argh::from_env();
    let background_color = csscolorparser::parse(&args.background_color)
        .unwrap()
        .to_rgba16();
    // TODO: use the input_image to get the default stem. Otherwise, multiple images will overwrite.
    for image_file in &args.input_images {
        let img = open(image_file).expect("Failed to open image");
        extract_images_from_image_grid(
            &img,
            args.aspect_ratio,
            args.max_width,
            Rgba(background_color),
            "/tmp/imm",
            "stem",
        )?;
    }
    Ok(())
}

use argh::FromArgs;
use fpc::*;
use image::{open, Rgba};

/// Find playing cards in a source image.
/// TODO: make this better.
#[derive(FromArgs, Debug)]
struct Args {
    #[argh(positional)]
    input_images: Vec<String>,
}

fn main() {
    let args: Args = argh::from_env();

    for image_file in &args.input_images {
        let img = open(image_file).expect("Failed to open image");
        //        find_bounding_boxes(img).expect("TODO: failed to find bounding box");
        extract_images_from_image_grid(
            &img,
            1.0,
            (0, 0),
            Rgba([65535, 65535, 65535, 65535]),
            "/tmp/imm",
            "stem",
        );
    }
    println!("Hello, world!");
}

mod grid_finder;

use crate::grid_finder::find_grid_cells;
use image::imageops::{overlay, FilterType};
use image::math::Rect;
use image::{DynamicImage, GenericImage, GenericImageView, ImageBuffer, ImageError, Pixel, Rgba};
use std::ffi::OsStr;
use std::ops::Range;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FpcError {
    #[error("A border was not detected along the {0} edge")]
    MissingBorder(&'static str),

    #[error("a blank row was not found")]
    BlankNotFound,

    #[error("an underlying image error")]
    ImageError(#[from] ImageError),

    #[error("An unknown and hopefully unused error.")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, FpcError>;

// extract_images_from_image_grid
// - takes aspect ratio (w/h)
// - takes padding (horiz, vert)
// - background color (CSS color)
// - output directory (default './')
// - output file stem (default 'image'
//
// 1. find the grid locations
// 2. using the grid, find the boxes just inside the grids
// 3. for each grid cell,
//    a. find the image inside
//    b. add padding
//    c. expand to desired AR
//    d. ensure fits in grid box
//    e. create output image
//         i. if background color, fill image with rounded corners
//        ii. copy from original image into output image
//       iii. write to output file

pub fn extract_images_from_image_grid(
    img: &DynamicImage,
    aspect_ratio: f64,
    max_width: u32,
    background_color: Rgba<u16>,
    output_directory: impl AsRef<OsStr>,
    output_file_stem: impl AsRef<OsStr>,
) -> Result<()> {
    let cells = find_grid_cells(img)?;
    output_debug_image(img, &cells)?;

    for (i, rect) in cells.iter().enumerate() {
        let tail = format!("-{}", i);
        let mut filename = output_file_stem.as_ref().to_os_string();
        filename.push(tail);
        let path = Path::new(output_directory.as_ref())
            .join(filename)
            .with_extension("png");
        // Rgba([65535u16, 65535, 65535, 65535]
        let new_image = make_sub_image(img, rect, background_color)?;
        let new_image_bounds = new_image.bounds();
        let scaled_image = scale_to_constraints(
            &DynamicImage::ImageRgba16(new_image),
            new_image_bounds,
            aspect_ratio,
            max_width,
        )?;

        let corner_radius = ((1.0 / 8.0) * 300.0) as u32; // 1/8in with 300dpi
        let rounded_image = round_the_corners(&scaled_image, corner_radius)?;
        let rescaled_image = rescale_to_72dpi_1in(aspect_ratio, rounded_image);

        rescaled_image.save(path)?;
    }

    Ok(())
}

fn rescale_to_72dpi_1in(aspect_ratio: f64, rounded_image: DynamicImage) -> DynamicImage {
    let new_width = (1.5 * 72.0) as u32;
    let rescaled_image = rounded_image.resize(
        new_width,
        (new_width as f64 * aspect_ratio) as u32,
        FilterType::Triangle,
    );
    rescaled_image
}

fn round_the_corners(img: &DynamicImage, corner_radius_px: u32) -> Result<DynamicImage> {
    let mut rounded_image = img.clone();
    let (x, y, width, height) = rounded_image.bounds();

    let top_left_center = (x + corner_radius_px, y + corner_radius_px);
    round_a_corner(
        &mut rounded_image,
        x..top_left_center.0,
        y..top_left_center.1,
        corner_radius_px,
        top_left_center,
    );
    let top_right_center = (x + width - corner_radius_px, y + corner_radius_px);
    round_a_corner(
        &mut rounded_image,
        top_right_center.0..x + width,
        y..top_right_center.1,
        corner_radius_px,
        top_right_center,
    );
    let bottom_left_center = (x + corner_radius_px, y + height - corner_radius_px);
    round_a_corner(
        &mut rounded_image,
        x..bottom_left_center.0,
        bottom_left_center.1..y + height,
        corner_radius_px,
        bottom_left_center,
    );
    let bottom_right_center = (x + width - corner_radius_px, y + height - corner_radius_px);
    round_a_corner(
        &mut rounded_image,
        bottom_right_center.0..x + width,
        bottom_right_center.1..y + height,
        corner_radius_px,
        bottom_right_center,
    );
    Ok(rounded_image)
}

fn round_a_corner(
    img: &mut DynamicImage,
    x_range: Range<u32>,
    y_range: Range<u32>,
    corner_radius_px: u32,
    center_point: (u32, u32),
) {
    let squared = corner_radius_px * corner_radius_px;
    for y in y_range {
        let y_offset = y as i32 - center_point.1 as i32;
        for x in x_range.clone() {
            let x_offset = x as i32 - center_point.0 as i32;
            if (x_offset * x_offset + y_offset * y_offset) as u32 > squared {
                img.put_pixel(x, y, Rgba([0, 0, 0, 0]));
            }
        }
    }
}

fn scale_to_constraints(
    img: &DynamicImage,
    bounds: (u32, u32, u32, u32),
    aspect_ratio: f64,
    max_width: u32,
) -> Result<DynamicImage> {
    let (x, y, width, height) = bounds;
    let mut maybe_width = std::cmp::min(width, max_width);
    let mut maybe_height = (maybe_width as f64 * aspect_ratio) as u32;
    if maybe_height > height {
        let scale = height / maybe_height;
        maybe_width *= scale;
        maybe_height *= scale;
    }

    let new_bounds = Rect {
        x: x + (width - maybe_width) / 2,
        y: y + (height - maybe_height) / 2,
        width: maybe_width,
        height: maybe_height,
    };

    let final_image = img.crop_imm(
        new_bounds.x,
        new_bounds.y,
        new_bounds.width,
        new_bounds.height,
    );
    Ok(final_image)
}

fn make_sub_image(
    img: &DynamicImage,
    rect: &Rect,
    background_color: Rgba<u16>,
) -> Result<ImageBuffer<Rgba<u16>, Vec<u16>>> {
    // TODO: add rounded corners.
    let mut new_image = ImageBuffer::from_pixel(rect.width, rect.height, background_color);
    let sub_image = img.crop_imm(rect.x, rect.y, rect.width, rect.height);
    overlay(&mut new_image, &sub_image.to_rgba16(), 0, 0);
    Ok(new_image)
}

fn output_debug_image(img: &DynamicImage, cells: &Vec<Rect>) -> Result<()> {
    let mut img_copy = img.clone();
    for rect in cells {
        draw_rect(&mut img_copy, rect);
    }
    img_copy.save("DEBUG.png")?;
    Ok(())
}

fn draw_rect(img: &mut DynamicImage, rect: &Rect) {
    let color = Rgba::<u8>([255, 0, 0, 255]);
    draw_line(img, rect.x..rect.x + rect.width, rect.y, |img, i, at| {
        img.put_pixel(i, at, color);
    });
    draw_line(
        img,
        rect.x..rect.x + rect.width,
        rect.y + rect.height - 1,
        |img, i, at| {
            img.put_pixel(i, at, color);
        },
    );

    draw_line(img, rect.y..rect.y + rect.height, rect.x, |img, i, at| {
        img.put_pixel(at, i, color);
    });
    draw_line(
        img,
        rect.y..rect.y + rect.height,
        rect.x + rect.width - 1,
        |img, i, at| {
            img.put_pixel(at, i, color);
        },
    );
}

fn draw_line(
    img: &mut DynamicImage,
    range: impl Iterator<Item = u32>,
    at: u32,
    set_pixel: impl Fn(&mut DynamicImage, u32, u32),
) {
    range.for_each(|i| set_pixel(img, i, at));
}

pub fn find_bounding_boxes(img: DynamicImage) -> Result<Vec<Rect>> {
    let bounds = img.bounds();
    println!("bounds: {:?}", bounds);
    let center = ((bounds.0 + bounds.2) / 2, (bounds.1 + bounds.3) / 2);
    println!("center: {:?}", center);
    let left_edge = scan_horiz(&img, center, -1)?;
    let right_edge = scan_horiz(&img, center, 1)?;
    let top_edge = scan_vert(&img, center, -1)?;
    println!("top edge: {}", top_edge);
    let bottom_edge = scan_vert(&img, center, 1)?;
    println!("bottom edge: {}", bottom_edge);

    Ok(vec![Rect {
        x: left_edge,
        y: top_edge,
        width: right_edge - left_edge + 1,
        height: bottom_edge - top_edge + 1,
    }])
}

fn scan_horiz(img: &DynamicImage, center: (u32, u32), delta: i32) -> Result<u32> {
    let mut edge = center.0 as i32;
    while edge >= 0 && (edge as u32) < img.width() {
        if (0..img.height()).all(|y| {
            let pixel = img.get_pixel(edge as u32, y);

            // channel 3 is the alpha channel
            pixel.channels()[3] == 0
        }) {
            println!("Found horiz: {}", edge);
            return Ok(edge as u32);
        }
        edge += delta;
    }
    Err(FpcError::Unknown)
}

fn scan_vert(img: &DynamicImage, center: (u32, u32), delta: i32) -> Result<u32> {
    let mut edge = center.1 as i32;
    while edge >= 0 && (edge as u32) < img.height() {
        if (0..img.width()).all(|x| {
            let pixel = img.get_pixel(x, edge as u32);

            // channel 3 is the alpha channel
            pixel.channels()[3] == 0
        }) {
            println!("Found vert: {}", edge);
            return Ok(edge as u32);
        }
        edge += delta;
    }
    Err(FpcError::Unknown)
}

#[cfg(test)]
mod test {
    use crate::find_bounding_boxes;
    use image::math::Rect;
    use image::open;

    #[test]
    fn circle_test() {
        let img = open("test_inputs/circle.png").expect("Failed to open image");
        let boxes = find_bounding_boxes(img).expect("Failed to get bounding box");

        assert_eq!(1, boxes.len());
        assert_eq!(
            Rect {
                x: 54,
                y: 90,
                // TODO: this is wrong. It should be 70. Fix it.
                width: 72,
                height: 72,
            },
            boxes[0]
        );
    }

    #[test]
    fn rect_test() {
        let img = open("test_inputs/rect.png").expect("Failed to open image");
        let boxes = find_bounding_boxes(img).expect("Failed to get bounding box");

        assert_eq!(1, boxes.len());
        assert_eq!(
            Rect {
                x: 39,
                y: 59,
                width: 102,
                height: 122,
            },
            boxes[0]
        );
    }

    #[test]
    fn rect_circle_border() {
        let img = open("test_inputs/circle_border.png").expect("Failed to open image");
        let boxes = find_bounding_boxes(img).expect("Failed to get bounding box");

        assert_eq!(1, boxes.len());
        assert_eq!(
            Rect {
                x: 21,
                y: 30,
                width: 30,
                height: 30,
            },
            boxes[0]
        );
    }
}

/// Returns `true` if the pixel in `img` and (`x`,`y`) is transparent (alpha = 0).
pub fn transparent_pixel(img: &DynamicImage, x: u32, y: u32) -> bool {
    img.get_pixel(x, y).channels()[3] == 0
}

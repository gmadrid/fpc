use crate::transparent_pixel;
use crate::{FpcError, Result};
use image::math::Rect;
use image::DynamicImage;
use itertools::Itertools;
use std::ops::Range;

#[derive(Debug)]
struct GridFinder {}

impl GridFinder {}

fn find_pixels<'a>(
    range: impl Iterator<Item = u32> + 'a,
    mut predicate: impl FnMut(u32) -> bool + 'a,
) -> impl Iterator<Item = u32> + 'a {
    range.flat_map(move |i| if predicate(i) { Some(i) } else { None })
}

fn group_sequences(iter: impl Iterator<Item = u32>) -> Vec<Range<u32>> {
    iter.fold(vec![], |mut acc: Vec<Range<u32>>, i| {
        if let Some(range) = acc.last_mut() {
            if range.end == i {
                range.end += 1;
            } else {
                acc.push(Range {
                    start: i,
                    end: i + 1,
                });
            }
        } else {
            acc.push(Range {
                start: i,
                end: i + 1,
            })
        }
        acc
    })
}

pub fn find_grid_cells(img: &DynamicImage) -> Result<Vec<Rect>> {
    let bordered_bounds = find_bordered_bounds(img)?;
    let (left_thickness, top_thickness, right_thickness, bottom_thickness) =
        find_border_widths(img, bordered_bounds)?;
    let inside_border_bounds = Rect {
        x: left_thickness,
        y: top_thickness,
        width: bordered_bounds.width - left_thickness - right_thickness,
        height: bordered_bounds.height - top_thickness - bottom_thickness,
    };
    let vert_grid_line_ranges = find_grid_line_ranges(
        bordered_bounds.x..bordered_bounds.x + bordered_bounds.width,
        move |xx| transparent_pixel(img, xx, inside_border_bounds.y),
    );
    let horiz_grid_line_ranges = find_grid_line_ranges(
        bordered_bounds.y..bordered_bounds.y + bordered_bounds.height,
        move |yy| transparent_pixel(img, inside_border_bounds.x, yy),
    );
    let mut rects: Vec<Rect> = vec![];
    for (top, bottom) in horiz_grid_line_ranges.iter().tuple_windows() {
        for (left, right) in vert_grid_line_ranges.iter().tuple_windows() {
            let x = left.end;
            let y = top.end;
            rects.push(Rect {
                x,
                y,
                width: right.start - x,
                height: bottom.start - y,
            })
        }
    }
    Ok(rects)
}

fn find_grid_line_ranges<'a>(
    range: impl Iterator<Item = u32>,
    is_blank: impl Fn(u32) -> bool + 'a,
) -> Vec<Range<u32>> {
    let pixels_iter = find_pixels(range, |i| !is_blank(i));
    group_sequences(pixels_iter)
}

fn find_bordered_bounds(img: &DynamicImage) -> Result<Rect> {
    // From the center of each edge, search inward until we find the first non-blank pixel.
    // We assume that this is the start of the border around the entire image grid.

    let center_x = img.width() / 2;
    let center_y = img.height() / 2;
    let left = (0..img.width())
        .find(|xx| !transparent_pixel(img, *xx, center_y))
        .ok_or(FpcError::MissingBorder("top"))?;
    let right = (0..img.width())
        .rev()
        .find(|xx| !transparent_pixel(img, *xx, center_y))
        .ok_or(FpcError::MissingBorder("right"))?;
    let top = (0..img.height())
        .find(|yy| !transparent_pixel(img, center_x, *yy))
        .ok_or(FpcError::MissingBorder("top"))?;
    let bottom = (0..img.height())
        .rev()
        .find(|yy| !transparent_pixel(img, center_x, *yy))
        .ok_or(FpcError::MissingBorder("bottom"))?;
    Ok(Rect {
        x: left,
        y: top,
        width: right - left + 1,
        height: bottom - top + 1,
    })
}

fn scan_range(
    mut range: impl Iterator<Item = u32>,
    predicate: impl FnMut(&u32) -> bool,
) -> Result<u32> {
    range.find(predicate).ok_or(FpcError::BlankNotFound)
}

fn find_border_widths(img: &DynamicImage, bounds: Rect) -> Result<(u32, u32, u32, u32)> {
    let (x, y, w, h) = (bounds.x, bounds.y, bounds.width, bounds.height);
    let center_x = w / 2;
    let center_y = h / 2;
    let is_blank = transparent_pixel;

    let top_thickness = scan_range(y..y + h, |yy| is_blank(img, center_x, *yy))?;
    let bottom_thickness = h - scan_range((y..y + h).rev(), |yy| is_blank(img, center_x, *yy))? - 1;
    let left_thickness = scan_range(x..x + w, |xx| is_blank(img, *xx, center_y))?;
    let right_thickness = w - scan_range((x..x + w).rev(), |xx| is_blank(img, *xx, center_y))? - 1;

    Ok((
        left_thickness,
        top_thickness,
        right_thickness,
        bottom_thickness,
    ))
}

#[cfg(test)]
mod test {
    use crate::grid_finder::{find_border_widths, find_grid_cells};
    use image::math::Rect;
    use image::open;

    #[test]
    fn test_border_width() {
        let img = open("test_inputs/3x5 grid.png").expect("Failed to open image");
        let foo = find_border_widths(&img).unwrap();

        assert_eq!(foo, (0, 0, 0, 0));
    }

    #[test]
    fn test_grid() {
        let img = open("test_inputs/3x5 grid.png").expect("Failed to open image");
        let foo = find_grid_cells(&img).unwrap();

        assert_eq!(Vec::<Rect>::default(), foo);
    }
}

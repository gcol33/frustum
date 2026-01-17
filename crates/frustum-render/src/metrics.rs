//! Image-derived metrics computation.
//!
//! Computes summary metrics from rendered pixel data without requiring
//! the AI to evaluate raw images.

use crate::audit::{ColorHistogram, ImageMetrics};

/// Compute image metrics from RGBA pixel data.
pub fn compute_image_metrics(
    pixels: &[u8],
    width: u32,
    height: u32,
    background: [f32; 4],
) -> ImageMetrics {
    let pixel_count = (width * height) as usize;

    // Convert background to u8 for comparison
    let bg_r = (background[0] * 255.0) as u8;
    let bg_g = (background[1] * 255.0) as u8;
    let bg_b = (background[2] * 255.0) as u8;

    // Initialize histogram
    let mut histogram = ColorHistogram {
        red: [0; 16],
        green: [0; 16],
        blue: [0; 16],
        alpha: [0; 16],
    };

    let mut transparent_count = 0u32;
    let mut background_count = 0u32;

    // Compute histogram and counts
    for chunk in pixels.chunks(4) {
        if chunk.len() < 4 {
            continue;
        }

        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];

        // Update histogram (16 bins = 256/16 = 16 values per bin)
        histogram.red[(r / 16) as usize] += 1;
        histogram.green[(g / 16) as usize] += 1;
        histogram.blue[(b / 16) as usize] += 1;
        histogram.alpha[(a / 16) as usize] += 1;

        // Count transparent pixels
        if a == 0 {
            transparent_count += 1;
        }

        // Count background pixels (with tolerance)
        if is_similar_color(r, g, b, bg_r, bg_g, bg_b, 5) {
            background_count += 1;
        }
    }

    // Compute edge density using simple Sobel-like operator
    let edge_density = compute_edge_density(pixels, width, height);

    // Find dominant colors
    let dominant_colors = find_dominant_colors(pixels);

    // Compute connected components (simplified)
    let connected_components = estimate_connected_components(pixels, width, height, bg_r, bg_g, bg_b);

    ImageMetrics {
        histogram,
        edge_density,
        transparent_percentage: (transparent_count as f32 / pixel_count as f32) * 100.0,
        background_percentage: (background_count as f32 / pixel_count as f32) * 100.0,
        connected_components,
        dominant_colors,
    }
}

/// Check if two colors are similar within tolerance.
fn is_similar_color(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8, tolerance: u8) -> bool {
    let dr = (r1 as i16 - r2 as i16).unsigned_abs() as u8;
    let dg = (g1 as i16 - g2 as i16).unsigned_abs() as u8;
    let db = (b1 as i16 - b2 as i16).unsigned_abs() as u8;
    dr <= tolerance && dg <= tolerance && db <= tolerance
}

/// Compute edge density using a simple gradient magnitude approach.
fn compute_edge_density(pixels: &[u8], width: u32, height: u32) -> f32 {
    if width < 3 || height < 3 {
        return 0.0;
    }

    let mut edge_count = 0u32;
    let threshold = 30u16; // Edge threshold

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let idx_left = ((y * width + x - 1) * 4) as usize;
            let idx_right = ((y * width + x + 1) * 4) as usize;
            let idx_up = (((y - 1) * width + x) * 4) as usize;
            let idx_down = (((y + 1) * width + x) * 4) as usize;

            // Compute gradient magnitude for luminance
            let lum = |i: usize| -> i16 {
                if i + 2 < pixels.len() {
                    (pixels[i] as i16 + pixels[i + 1] as i16 + pixels[i + 2] as i16) / 3
                } else {
                    0
                }
            };

            let gx = (lum(idx_right) - lum(idx_left)).abs() as u16;
            let gy = (lum(idx_down) - lum(idx_up)).abs() as u16;

            if gx + gy > threshold {
                edge_count += 1;
            }
        }
    }

    let interior_pixels = ((width - 2) * (height - 2)) as f32;
    edge_count as f32 / interior_pixels
}

/// Find dominant colors using simple clustering.
fn find_dominant_colors(pixels: &[u8]) -> Vec<[u8; 3]> {
    use std::collections::HashMap;

    // Quantize colors to 4-bit per channel (16 levels) and count
    let mut color_counts: HashMap<(u8, u8, u8), u32> = HashMap::new();

    for chunk in pixels.chunks(4) {
        if chunk.len() >= 3 && (chunk.len() < 4 || chunk[3] > 128) {
            // Only count non-transparent pixels
            let r = chunk[0] / 16;
            let g = chunk[1] / 16;
            let b = chunk[2] / 16;
            *color_counts.entry((r, g, b)).or_insert(0) += 1;
        }
    }

    // Sort by count and take top 5
    let mut sorted: Vec<_> = color_counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    sorted
        .into_iter()
        .take(5)
        .map(|((r, g, b), _)| [r * 16 + 8, g * 16 + 8, b * 16 + 8])
        .collect()
}

/// Estimate connected components using flood fill on downsampled image.
fn estimate_connected_components(
    pixels: &[u8],
    width: u32,
    height: u32,
    bg_r: u8,
    bg_g: u8,
    bg_b: u8,
) -> u32 {
    // Downsample for performance (8x8 blocks)
    let block_size = 8;
    let small_w = (width / block_size).max(1);
    let small_h = (height / block_size).max(1);

    // Create binary mask (foreground vs background)
    let mut mask = vec![false; (small_w * small_h) as usize];

    for sy in 0..small_h {
        for sx in 0..small_w {
            // Sample center of block
            let x = (sx * block_size + block_size / 2).min(width - 1);
            let y = (sy * block_size + block_size / 2).min(height - 1);
            let idx = ((y * width + x) * 4) as usize;

            if idx + 2 < pixels.len() {
                let is_bg = is_similar_color(
                    pixels[idx],
                    pixels[idx + 1],
                    pixels[idx + 2],
                    bg_r,
                    bg_g,
                    bg_b,
                    10,
                );
                mask[(sy * small_w + sx) as usize] = !is_bg;
            }
        }
    }

    // Count connected components using flood fill
    let mut visited = vec![false; mask.len()];
    let mut components = 0u32;

    for i in 0..mask.len() {
        if mask[i] && !visited[i] {
            flood_fill(&mask, &mut visited, small_w as usize, small_h as usize, i);
            components += 1;
        }
    }

    components
}

/// Simple flood fill for connected components.
fn flood_fill(mask: &[bool], visited: &mut [bool], width: usize, height: usize, start: usize) {
    let mut stack = vec![start];

    while let Some(idx) = stack.pop() {
        if idx >= mask.len() || visited[idx] || !mask[idx] {
            continue;
        }

        visited[idx] = true;

        let x = idx % width;
        let y = idx / width;

        // Add neighbors
        if x > 0 {
            stack.push(idx - 1);
        }
        if x < width - 1 {
            stack.push(idx + 1);
        }
        if y > 0 {
            stack.push(idx - width);
        }
        if y < height - 1 {
            stack.push(idx + width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_image_metrics_solid_color() {
        // Create 4x4 solid red image
        let mut pixels = vec![0u8; 4 * 4 * 4];
        for chunk in pixels.chunks_mut(4) {
            chunk[0] = 255; // R
            chunk[1] = 0; // G
            chunk[2] = 0; // B
            chunk[3] = 255; // A
        }

        let metrics = compute_image_metrics(&pixels, 4, 4, [0.0, 0.0, 0.0, 1.0]);

        assert_eq!(metrics.transparent_percentage, 0.0);
        assert_eq!(metrics.background_percentage, 0.0);
        assert!(metrics.histogram.red[15] == 16); // All red values in highest bin
    }

    #[test]
    fn test_compute_image_metrics_transparent() {
        // Create 4x4 fully transparent image
        let pixels = vec![0u8; 4 * 4 * 4];

        let metrics = compute_image_metrics(&pixels, 4, 4, [0.0, 0.0, 0.0, 1.0]);

        assert_eq!(metrics.transparent_percentage, 100.0);
    }
}

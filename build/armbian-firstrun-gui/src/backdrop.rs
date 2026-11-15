//! The background: light on black, made as an image at the start.
//!
//! The light has the form of the armbian.com page: a warm key light from the top right in
//! the accent color, a cool grey fill on the left, and a low glow from the bottom right.
//! An image, not gradients: the software renderer copies an opaque image fast, and the
//! grain in the image stops the bands that a smooth gradient shows on 16-bit panels.

/// One light: center and radii as parts of the screen, color, and strength (0 to 1).
struct Light {
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    color: [f32; 3],
    strength: f32,
}

/// Black of the page (Theme.bg).
const BASE: [f32; 3] = [5.0, 5.0, 5.0];
/// The cool grey of the armbian.com page light.
const COOL: [f32; 3] = [100.0, 116.0, 139.0];
/// Grain, in color levels. It hides the steps of a 16-bit panel (8 levels for red and blue).
const GRAIN: f32 = 5.0;

/// A number from 0 to 1 for a pixel. The same pixel gives the same number.
fn noise(x: u32, y: u32, salt: u32) -> f32 {
    let mut n = x.wrapping_mul(0x9E37_79B1) ^ y.wrapping_mul(0x85EB_CA77) ^ salt.wrapping_mul(0xC2B2_AE3D);
    n ^= n >> 15;
    n = n.wrapping_mul(0x2C1B_3C6D);
    n ^= n >> 12;
    n = n.wrapping_mul(0x297A_2D39);
    n ^= n >> 15;
    (n >> 8) as f32 / 16_777_216.0
}

/// The background as RGB bytes, `width` x `height`, for the accent color.
pub fn render(width: u32, height: u32, accent: [u8; 3]) -> Vec<u8> {
    let accent = accent.map(f32::from);
    // The accent with some white: the hot center of the key light.
    let hot = accent.map(|c| c + (255.0 - c) * 0.22);
    let lights = [
        Light { cx: 0.80, cy: 0.04, rx: 0.62, ry: 0.86, color: accent, strength: 0.42 },
        Light { cx: 0.52, cy: -0.16, rx: 0.70, ry: 0.70, color: hot, strength: 0.20 },
        Light { cx: 0.06, cy: 0.38, rx: 0.52, ry: 0.72, color: COOL, strength: 0.26 },
        Light { cx: 0.96, cy: 1.04, rx: 0.46, ry: 0.52, color: accent, strength: 0.20 },
    ];
    let mut out = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        let v = (y as f32 + 0.5) / height as f32;
        for x in 0..width {
            let u = (x as f32 + 0.5) / width as f32;
            let mut c = BASE;
            for l in &lights {
                let (dx, dy) = ((u - l.cx) / l.rx, (v - l.cy) / l.ry);
                let d = dx * dx + dy * dy;
                if d < 1.0 {
                    // Soft at the center and at the edge.
                    let f = 1.0 - d.sqrt();
                    let a = l.strength * f * f * (3.0 - 2.0 * f);
                    for i in 0..3 {
                        c[i] += (l.color[i] - c[i]) * a;
                    }
                }
            }
            // Two numbers give a soft grain (most values near 0).
            let g = (noise(x, y, 1) + noise(x, y, 2) - 1.0) * GRAIN;
            out.extend(c.map(|c| (c + g).round().clamp(0.0, 255.0) as u8));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_and_dark() {
        let (w, h) = (160, 100);
        let img = render(w, h, [242, 101, 34]);
        assert_eq!(img.len(), (w * h * 3) as usize);
        let at = |x: u32, y: u32| img[((y * w + x) * 3) as usize] as i32;
        // The top right is lit, the bottom left is almost black.
        assert!(at(130, 6) > 60);
        assert!(at(40, 96) < 24);
    }
}

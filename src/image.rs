use std::ops;

use crate::object::{Hittable, Point, Ray, ScatteredRay, Vec3, World};
use crate::utils::Interval;

// Maximum value contained in an RGB channel
pub const MAX_COLOR_CHANNEL_VALUE: u8 = 255;
// P3 means the file contains a portable pixmap image written in ASCII
// https://en.wikipedia.org/wiki/Netpbm#Description
const PPM_MAGIC_NUMBER: &str = "P3";
const MINIMUM_DISTANCE_AGAINST_SHADOW_ACNE: f64 = 0.0001;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    fn white() -> Color {
        Color {
            r: MAX_COLOR_CHANNEL_VALUE,
            g: MAX_COLOR_CHANNEL_VALUE,
            b: MAX_COLOR_CHANNEL_VALUE,
        }
    }
    fn black() -> Color {
        Color { r: 0, g: 0, b: 0 }
    }

    fn mean_color(colors: Vec<Color>) -> Color {
        let mut r: u16 = 0;
        let mut g: u16 = 0;
        let mut b: u16 = 0;
        for color in &colors {
            r += color.r as u16;
            g += color.g as u16;
            b += color.b as u16;
        }
        r /= colors.len() as u16;
        g /= colors.len() as u16;
        b /= colors.len() as u16;
        Color {
            r: r as u8,
            g: g as u8,
            b: b as u8,
        }
    }
    fn channel_gamma_correction(color: u8) -> u8 {
        if color > 0 {
            f64::sqrt(color as f64) as u8
        } else {
            color
        }
    }

    /// Translate the color values from linear space to gamma space
    fn gamma_corrected(self) -> Color {
        Color {
            r: Color::channel_gamma_correction(self.r),
            g: Color::channel_gamma_correction(self.g),
            b: Color::channel_gamma_correction(self.b),
        }
    }
}

    }
}

impl ops::Mul<f64> for Color {
    type Output = Color;
    fn mul(self, rhs: f64) -> Self::Output {
        Color {
            r: (self.r as f64 * rhs) as u8,
            g: (self.g as f64 * rhs) as u8,
            b: (self.b as f64 * rhs) as u8,
        }
    }
}

impl ops::Mul<Color> for f64 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: (self * rhs.r as f64) as u8,
            g: (self * rhs.g as f64) as u8,
            b: (self * rhs.b as f64) as u8,
        }
    }
}

impl ops::Add<Color> for Color {
    type Output = Color;
    fn add(self, rhs: Color) -> Self::Output {
        let (r, overflow) = self.r.overflowing_add(rhs.r);
        let r = if overflow { MAX_COLOR_CHANNEL_VALUE } else { r };
        let (g, overflow) = self.g.overflowing_add(rhs.g);
        let g = if overflow { MAX_COLOR_CHANNEL_VALUE } else { g };
        let (b, overflow) = self.b.overflowing_add(rhs.b);
        let b = if overflow { MAX_COLOR_CHANNEL_VALUE } else { b };
        Color { r, g, b }
    }
}

#[derive(Clone)]
struct Pixel {
    color: Color,
}

impl std::fmt::Display for Pixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = &self.color;
        write!(f, "{:3} {:3} {:3}", color.r, color.g, color.b)
    }
}

pub struct Camera {
    image_width: u32,
    image_height: u32,
    pixel_00_loc: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    center: Point,
    sample_per_pixel: u32,
    max_ray_bounces: u16,
}

impl Camera {
    fn ray_color<T: Hittable>(ray: &Ray, world: &World<T>, depth: u16) -> Color {
        if depth <= 0 {
            return Color::black();
        }
        if let Some(hit) = world.hit(
            ray,
            Interval {
                // Because of floating rounding error, the origin of the reflected Ray might be
                // just slightly off from where it's supposed to be. If the error puts the Ray
                // origin inside the object, the reflected ray might detect a new hit from the
                // inside of the object it just bounced off.  This is called shadow acne.
                // To prevent this, discard hits that occur very close to the Ray origin.
                min: MINIMUM_DISTANCE_AGAINST_SHADOW_ACNE,
                max: f64::INFINITY,
            },
        ) {
            // Get scattered ray based on the type of material that was hit
            let scattered_ray = ScatteredRay::scatter(&hit);
            scattered_ray.attenuation * Camera::ray_color(&scattered_ray.ray, world, depth - 1)
        } else {
            Ray::blue_lerp(ray)
        }
    }

    pub fn initialize(
        aspect_ratio: f64,
        image_width: u32,
        sample_per_pixel: u32,
        max_ray_bounces: u16,
    ) -> Camera {
        let image_height = (image_width as f64 / aspect_ratio) as u32;
        let image_height = if image_height < 1 { 1 } else { image_height };

        // Viewport
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (image_width / image_height) as f64;
        let camera_center = Point {
            x: 0.,
            y: 0.,
            z: 0.,
        };

        let viewport_u = Vec3 {
            x: 0.,
            y: 0.,
            z: viewport_width,
        };
        let viewport_v = Vec3 {
            x: 0.,
            y: -viewport_height,
            z: 0.,
        };

        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;
        let viewport_upper_left = Vec3 {
            x: focal_length,
            y: 0.,
            z: 0.,
        } - viewport_u / 2.
            - viewport_v / 2.;
        // Position of the center of the pixel at location (0,0).
        let pixel_00_loc = viewport_upper_left + 0.5 * (pixel_delta_v + pixel_delta_u);

        Camera {
            sample_per_pixel,
            image_width,
            image_height,
            pixel_00_loc,
            pixel_delta_u,
            pixel_delta_v,
            center: camera_center,
            max_ray_bounces,
        }
    }

    pub fn render<T: Hittable>(&self, world: &World<T>, gamma_corrected: bool) -> Image {
        // Image content
        let mut pixels_matrix = Vec::with_capacity(self.image_height as usize);
        // Get the color of each pixel
        // For each pixel, we're going to sample multiple colors
        for row in 0..self.image_height {
            let mut pixel_row: Vec<Pixel> = Vec::with_capacity(self.image_width as usize);
            for col in 0..self.image_width {
                let mut sampled_colors: Vec<Color> =
                    Vec::with_capacity(self.sample_per_pixel as usize);
                for _ in 0..self.sample_per_pixel {
                    let ray = self.get_ray(row as usize, col as usize);
                    sampled_colors.push(Camera::ray_color(&ray, world, self.max_ray_bounces));
                }

                pixel_row.push(if gamma_corrected {
                    Pixel {
                        color: Color::mean_color(sampled_colors).gamma_corrected(),
                    }
                } else {
                    Pixel {
                        color: Color::mean_color(sampled_colors),
                    }
                });
            }
            pixels_matrix.push(pixel_row);
        }

        Image {
            pixels: pixels_matrix,
        }
    }
    /// Construct a camera ray originating from the origin and directed at randomly sampled
    /// point around the pixel location row, column to prevent aliasing.
    /// Sampling around a pixel will prevent the "stair" like on edges of objects.
    fn get_ray(&self, row: usize, column: usize) -> Ray {
        let offset = Camera::sample_square();
        let pixel_sample = self.pixel_00_loc
            + (column as f64 + offset.z) * self.pixel_delta_u
            + (row as f64 + offset.y) * self.pixel_delta_v;
        let origin = self.center;
        let direction = pixel_sample - origin;
        Ray { origin, direction }
    }

    // Returns the vector to a random point in the [-.5,-.5];[+.5,+.5] unit square.
    fn sample_square() -> Vec3 {
        Vec3 {
            x: 0.,
            y: rand::random::<f64>() - 0.5, // rand::random::<f64> output is in [0;1[
            z: rand::random::<f64>() - 0.5,
        }
    }
}

pub struct Image {
    pixels: Vec<Vec<Pixel>>,
}

impl std::fmt::Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rows = self.pixels.len();
        let columns = self.pixels[0].len();
        let mut content =
            format!("{PPM_MAGIC_NUMBER}\n{columns} {rows}\n{MAX_COLOR_CHANNEL_VALUE}\n");
        for row in &self.pixels {
            let mut row_str = String::new();
            for pixel in row {
                row_str.push_str(format!("{pixel} ").as_str())
            }
            content.push_str(format!("{row_str} \n").as_str())
        }
        write!(f, "{}", content)
    }
}

#[allow(dead_code)]
fn example_img() -> Image {
    let row1 = [
        Pixel {
            color: Color { r: 255, g: 0, b: 0 },
        },
        Pixel {
            color: Color { r: 0, g: 255, b: 0 },
        },
        Pixel {
            color: Color { r: 0, g: 0, b: 255 },
        },
    ]
    .to_vec();
    let row2 = [
        Pixel {
            color: Color {
                r: 255,
                g: 255,
                b: 0,
            },
        },
        Pixel {
            color: Color {
                r: 255,
                g: 255,
                b: 255,
            },
        },
        Pixel {
            color: Color { r: 0, g: 0, b: 0 },
        },
    ]
    .to_vec();
    let img_content = [row1, row2].to_vec();
    Image {
        pixels: img_content,
    }
}

#[allow(dead_code)]
fn single_color_img(color: Color) -> Image {
    let image_height = 400;
    let image_width = 600;
    let mut pixels = Vec::with_capacity(image_height as usize);
    for _ in 0..image_height {
        let mut row = Vec::with_capacity(image_width as usize);
        for _ in 0..image_width {
            row.push(Pixel { color });
        }
        pixels.push(row);
    }
    Image { pixels }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_mul_f64() {
        let color = Color {
            r: 100,
            g: 40,
            b: 80,
        };
        assert_eq!(
            color * 0.5,
            Color {
                r: 50,
                g: 20,
                b: 40
            }
        );
        assert_eq!(
            0.5 * color,
            Color {
                r: 50,
                g: 20,
                b: 40
            }
        );
    }

    #[test]
    fn color_add() {
        let color1 = Color {
            r: 100,
            g: 40,
            b: 80,
        };
        let color2 = Color {
            r: 100,
            g: 40,
            b: 80,
        };
        assert_eq!(
            color1 + color2,
            Color {
                r: 200,
                g: 80,
                b: 160,
            }
        );

        let color1 = Color {
            r: 200,
            g: 40,
            b: 80,
        };
        let color2 = Color {
            r: 200,
            g: 40,
            b: 80,
        };
        assert_eq!(
            color1 + color2,
            Color {
                r: 255,
                g: 80,
                b: 160,
            }
        );
    }
}

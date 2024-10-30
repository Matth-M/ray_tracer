use std::ops;

use crate::object::{Hittable, HittableList, Point, Ray, Vec3};
use crate::utils::Interval;

// Maximum value contained in an RGB channel
pub const MAX_COLOR_CHANNEL_VALUE: u8 = 255;
// P3 means the file contains a portable pixmap image written in ASCII
// https://en.wikipedia.org/wiki/Netpbm#Description
const PPM_MAGIC_NUMBER: &str = "P3";

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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

pub struct Image {
    pixels: Vec<Vec<Pixel>>,
}

pub struct Camera {
    image_width: u32,
    image_height: u32,
    pixel_00_loc: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    center: Point,
}

impl Camera {
    fn ray_color<T: Hittable>(ray: &Ray, world: &HittableList<T>) -> Color {
        if let Some(hit) = world.hit(ray, Interval{min:0.,max:f64::INFINITY}) {
            let r = (0.5 * (hit.normal.x + 1.0) * MAX_COLOR_CHANNEL_VALUE as f64) as u8;
            let g = (0.5 * (hit.normal.y + 1.0) * MAX_COLOR_CHANNEL_VALUE as f64) as u8;
            let b = (0.5 * (hit.normal.z + 1.0) * MAX_COLOR_CHANNEL_VALUE as f64) as u8;
            Color { r, g, b }
        } else {
            Ray::blue_lerp(ray)
        }
    }

    pub fn initialize(aspect_ratio: f64, image_width: u32) -> Camera {
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
            image_width,
            image_height,
            pixel_00_loc,
            pixel_delta_u,
            pixel_delta_v,
            center: camera_center,
        }
    }

    pub fn render<T: Hittable>(&self, world: &HittableList<T>) -> Image {
        // Image content
        let mut pixels = Vec::with_capacity(self.image_height as usize);
        for j in 0..self.image_height {
            let mut row = Vec::with_capacity(self.image_width as usize);
            for i in 0..self.image_width {
                let pixel_center =
                    self.pixel_00_loc + i * self.pixel_delta_u + j * self.pixel_delta_v;
                let ray_direction = pixel_center - self.center;
                let r = Ray {
                    origin: self.center,
                    direction: ray_direction,
                };
                let color = Camera::ray_color(&r, world);
                row.push(Pixel { color });
            }
            pixels.push(row);
        }

        Image { pixels }
    }
}

impl std::fmt::Display for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rows = self.pixels.len();
        let columns = self.pixels[0].len();
        let mut content = format!(
            "{}\n{} {}\n{}\n",
            PPM_MAGIC_NUMBER, columns, rows, MAX_COLOR_CHANNEL_VALUE
        );
        for row in &self.pixels {
            let mut row_str = String::new();
            for pixel in row {
                row_str.push_str(format!("{} ", pixel).as_str())
            }
            content.push_str(format!("{} \n", row_str).as_str())
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

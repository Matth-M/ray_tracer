use std::ops;
use std::{f64, fs::File, io::Write, path::Path};

// Maximum value contained in an RGB channel
const MAX_COLOR_CHANNEL_VALUE: u8 = 255;
// P3 means the file contains a portable pixmap image written in ASCII
// https://en.wikipedia.org/wiki/Netpbm#Description
const PPM_MAGIC_NUMBER: &str = "P3";

#[derive(Clone)]
struct Pixel {
    color: Color,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
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

struct Image {
    pixels: Vec<Vec<Pixel>>,
}

impl std::fmt::Display for Pixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = &self.color;
        write!(f, "{:3} {:3} {:3}", color.r, color.g, color.b)
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
                row_str.push_str(format!("{} ", pixel.to_string()).as_str())
            }
            content.push_str(format!("{} \n", row_str).as_str())
        }
        write!(f, "{}", content)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalized(&self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        } / self.len()
    }

    fn dot(&self, v: &Vec3) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }
}

impl ops::Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl ops::Mul<Vec3> for u32 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self as f64 * rhs.x,
            y: self as f64 * rhs.y,
            z: self as f64 * rhs.z,
        }
    }
}

impl ops::Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Div<f64> for Vec3 {
    type Output = Vec3;
    fn div(self, rhs: f64) -> Self::Output {
        Vec3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

type Position = Vec3;

struct Ray {
    origin: Position,
    direction: Vec3,
}

impl Ray {
    fn at(&self, t: f64) -> Position {
        self.origin + self.direction * t
    }

    #[allow(dead_code)]
    fn blue_lerp(&self) -> Color {
        let normalized = self.direction.normalized();
        // a = 1 when y = 1.0, a = 0 when y = -1.0
        let a = 0.5 * (normalized.y + 1.0);
        let start_color = Color {
            r: MAX_COLOR_CHANNEL_VALUE,
            g: MAX_COLOR_CHANNEL_VALUE,
            b: MAX_COLOR_CHANNEL_VALUE,
        };
        let end_color = Color {
            r: (MAX_COLOR_CHANNEL_VALUE as f64 * 0.5) as u8,
            g: (MAX_COLOR_CHANNEL_VALUE as f64 * 0.7) as u8,
            b: (MAX_COLOR_CHANNEL_VALUE as f64 * 1.0) as u8,
        };
        (1.0 - a) * start_color + a * end_color
    }

    fn simple_sphere(&self) -> Color {
        if self.hits_sphere(
            Vec3 {
                x: 1.,
                y: 0.,
                z: 0.,
            },
            0.5,
        ) {
            Color {
                r: MAX_COLOR_CHANNEL_VALUE,
                g: 0,
                b: 0,
            }
        } else {
            let normalized = self.direction.normalized();
            // a = 1 when y = 1.0, a = 0 when y = -1.0
            let a = 0.5 * (normalized.y + 1.0);
            let start_color = Color {
                r: MAX_COLOR_CHANNEL_VALUE,
                g: MAX_COLOR_CHANNEL_VALUE,
                b: MAX_COLOR_CHANNEL_VALUE,
            };
            let end_color = Color {
                r: (MAX_COLOR_CHANNEL_VALUE as f64 * 0.5) as u8,
                g: (MAX_COLOR_CHANNEL_VALUE as f64 * 0.7) as u8,
                b: (MAX_COLOR_CHANNEL_VALUE as f64 * 1.0) as u8,
            };
            (1.0 - a) * start_color + a * end_color
        }
    }

    /// Checks if the ray is intersecting a sphere, and where along the ray. Returns None if no
    /// point of intersection is found
    fn hits_sphere(&self, sphere_center: Vec3, radius: f64) -> Option<f64> {
        let qc = sphere_center - self.origin; // ray origin to sphere center
        let a = self.direction.dot(&self.direction);
        let b = -2.0 * self.direction.dot(&qc);
        let c = qc.dot(&qc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0. {
            None
        } else {
            Some(discriminant)
        }
    }
}

fn main() {
    // Image
    let aspect_ratio = 3.0 / 2.0;
    let image_width = 500;
    let image_height = (image_width as f32 / aspect_ratio) as u32;
    let image_height = if image_height < 1 { 1 } else { image_height };

    // Viewport
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * (image_width / image_height) as f64;
    let camera_center = Position {
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

    // Image content
    let mut pixels = vec![];
    pixels.reserve(image_height as usize);
    for j in 0..image_height {
        let mut row: Vec<Pixel> = vec![];
        row.reserve(image_width as usize);
        for i in 0..image_width {
            let pixel_center = pixel_00_loc + i * pixel_delta_u + j * pixel_delta_v;
            let ray_direction = pixel_center - camera_center;
            let r = Ray {
                origin: camera_center,
                direction: ray_direction,
            };
            let color = r.simple_sphere();
            row.push(Pixel { color });
        }
        pixels.push(row);
    }

    let img = Image { pixels };

    // Create output file
    let path = Path::new("img.ppm");
    let display = path.display();
    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write image to file
    match file.write_all(img.to_string().as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => {}
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
    let mut pixels = vec![];
    pixels.reserve(image_height as usize);
    for _ in 0..image_height {
        let mut row: Vec<Pixel> = vec![];
        row.reserve(image_width as usize);
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

    #[test]
    fn vec3_normalized() {
        let v = Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        assert_eq!(
            v.normalized(),
            Vec3 {
                x: 1.0 / 3.0_f64.sqrt(),
                y: 1.0 / 3.0_f64.sqrt(),
                z: 1.0 / 3.0_f64.sqrt(),
            }
        );
    }

    #[test]
    fn vec3_len() {
        let v = Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };
        assert_eq!(v.len(), 3.0_f64.sqrt())
    }
}

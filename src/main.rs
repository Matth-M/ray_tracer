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

#[derive(Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
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

#[derive(Clone, Copy)]
struct Vec3 {
    x: f64,
    y: f64,
    z: f64,
}

impl Vec3 {
    fn len(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z + self.z).sqrt()
    }

    fn normalized(&self) -> Vec3 {
        let norm = self.len() * self.len();
        Vec3 {
            x: self.x / norm,
            y: self.y / norm,
            z: self.z / norm,
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
        self.origin.clone() + self.direction.clone() * t
    }

    fn color(&self) -> Color {
        let normalized = self.direction.normalized();
        // a = 1 when y = 1.0, a = 0 when y = -1.0
        let a = ((0.5) * (normalized.y + 1.0)) as u8;
        let bledend_value = 0;
        Color { r: 0, g: 0, b: 0 }
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
    } + viewport_u / 2.
        + viewport_v / 2.;
    // Position of the center of the pixel at location (0,0).
    let pixel_00_loc = viewport_upper_left + 0.5 * (pixel_delta_v + pixel_delta_u);

    // Image content
    let mut pixels = vec![];
    for j in 0..image_height {
        let mut row: Vec<Pixel> = vec![];
        for i in 0..image_width {
            let pixel_center = pixel_00_loc + i * pixel_delta_u + j * pixel_delta_v;
            let ray_direction = pixel_center - camera_center;
            let r = Ray {
                origin: camera_center,
                direction: ray_direction,
            };
            let color = r.color();
            row.push(Pixel { color: color });
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

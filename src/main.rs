use std::{f64, fs::File, io::Write, path::Path};

// Maximum value contained in an RGB channel
const MAX_COLOR_CHANNEL_VALUE: u8 = 255;
// P3 means the file contains a portable pixmap image written in ASCII
// https://en.wikipedia.org/wiki/Netpbm#Description
const PPM_MAGIC_NUMBER: &str = "P3";

#[derive(Clone)]
struct Pixel {
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

    }
}

fn main() {
    let row1 = [
        Pixel { r: 255, g: 0, b: 0 },
        Pixel { r: 0, g: 255, b: 0 },
        Pixel { r: 0, g: 0, b: 255 },
    ]
    .to_vec();
    let row2 = [
        Pixel {
            r: 255,
            g: 255,
            b: 0,
        },
        Pixel {
            r: 255,
            g: 255,
            b: 255,
        },
        Pixel { r: 0, g: 0, b: 0 },
    ]
    .to_vec();
    let img_content = [row1, row2].to_vec();
    let img = Image {
        pixels: img_content,
    };

    // Write image to file
    let path = Path::new("img.ppm");
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
    match file.write_all(img.to_string().as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => {}
    }
}

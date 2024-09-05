use std::{f64, fs::File, io::Write, path::Path};

#[derive(Clone)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

struct Image {
    pixels: Vec<Vec<Pixel>>,
}

impl Pixel {
    fn to_string(&self) -> String {
        String::from(format!("{:3} {:3} {:3}", self.r, self.g, self.b))
    }
}

impl Image {
    fn to_string(&self) -> String {
        let rows = self.pixels.len();
        let columns = self.pixels[0].len();
        let mut content = format!("P3\n{} {}\n255\n", columns, rows);
        for row in &self.pixels {
            let mut row_str = String::new();
            for pixel in row {
                row_str.push_str(format!("{} ", pixel.to_string()).as_str())
            }
            content.push_str(format!("{} \n", row_str).as_str())
        }
        content
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

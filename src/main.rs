use std::{fs::File, io::Write, path::Path};

mod image;
use image::Image;

mod object;

fn main() {
    // Image
    let aspect_ratio = 3.0 / 2.0;
    let image_width = 500;
    let img = Image::new(aspect_ratio, image_width);

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

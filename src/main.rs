use std::{fs::File, io::Write, path::Path, rc::Rc};

mod utils;
mod image;
use image::Camera;

mod object;
use object::{HittableList, Point, Sphere};

fn main() {
    let objects = vec![Rc::new(Sphere {
        center: Point {
            x: 1.,
            y: 0.,
            z: 0.,
        },
        radius: 0.5,
    })];

    let mut world = HittableList { objects };
    world.add(Rc::new(Sphere {
        center: Point {
            x: 1.,
            y: -100.5,
            z: 0.,
        },
        radius: 100.,
    }));

    // camera
    let aspect_ratio = 3.0 / 2.0;
    let image_width = 500;
    let camera = Camera::initialize(aspect_ratio, image_width);
    let image = camera.render(&world);

    // Create output file
    let path = Path::new("img.ppm");
    let display = path.display();
    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    // Write image to file
    if let Err(why) = file.write_all(image.to_string().as_bytes()) {
        panic!("couldn't write to {}: {}", display, why)
    }
}

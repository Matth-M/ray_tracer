use std::{fs::File, io::Write, path::Path, rc::Rc};

mod image;
mod utils;
use image::{Camera, Color};

mod object;
use object::{Material, MaterialType, Point, Sphere, World};

fn main() {
    let material_ground = Rc::new(Material {
        material_type: MaterialType::Lambertian,
        albedo: Color::from([0.5, 0.5, 0.5]),
    });

    let objects = vec![
        Rc::new(Sphere {
            center: Point {
                x: 1.,
                y: 0.,
                z: 0.,
            },
            radius: 0.5,
            material: Rc::clone(&material_ground),
        }),
        Rc::new(Sphere {
            center: Point {
                x: 1.,
                y: -100.5,
                z: 0.,
            },
            radius: 100.,
            material: Rc::clone(&material_ground),
        }),
    ];

    let world = World { objects };

    // camera
    let aspect_ratio = 3.0 / 2.0;
    let image_width = 500;
    let sample_per_pixel = 100;
    let max_ray_bounces = 50;
    let gamma_corrected = false;
    let camera = Camera::initialize(aspect_ratio, image_width, sample_per_pixel, max_ray_bounces);
    let image = camera.render(&world, gamma_corrected);

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

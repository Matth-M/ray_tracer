use std::path::Path;

mod image;
mod utils;
use image::Camera;

mod object;
use object::World;

fn main() {
    let objects = World::three_close_spheres();
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
    let path = Path::new("img.png");
    image.save(path).expect("Couldn't save image.");
}

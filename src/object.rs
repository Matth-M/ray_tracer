use crate::image::{Color, MAX_COLOR_CHANNEL_VALUE};
use std::{ops, rc::Rc};

use crate::utils::Interval;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
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

    pub fn dot(&self, v: &Vec3) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn random_unit_vector() -> Vec3 {
        Vec3 {
            x: rand::random::<f64>(),
            y: rand::random::<f64>(),
            z: rand::random::<f64>(),
        }
        .normalized()
    }

    fn near_zero(&self) -> bool {
        let limit = 1e-8;
        self.x < limit && self.y < limit && self.z < limit
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

pub type Point = Vec3;

pub struct Ray {
    pub origin: Point,
    pub direction: Vec3,
}

impl Ray {
    fn at(&self, t: f64) -> Point {
        self.origin + self.direction * t
    }

    /// Background, blue gradient based on y coordinates.
    pub fn blue_lerp(ray: &Ray) -> Color {
        let normalized = ray.direction.normalized();
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

#[derive(Debug, PartialEq)]
pub struct HitRecord {
    pub p: Point,
    pub normal: Vec3,
    t: f64,
    front_face: bool,
    material: Rc<Material>,
}

impl HitRecord {
    fn is_hit_from_front(ray: &Ray, outward_normal: &Vec3) -> bool {
        // If the normal and incoming ray's direction have a positive dot
        // product, they go in the same general "direction" -> the ray is not
        // goind inside the object
        ray.direction.dot(outward_normal) < 0.
    }
}

pub trait Hittable {
    /// Returns a HitRecord if the ray hits an objects, not too far from its origin
    /// -> with it's t (ray = origin + t * direction) inside the interval.
    fn hit(&self, ray: &Ray, interval: Interval) -> Option<HitRecord>;
}

pub struct ScatteredRay {
    pub ray: Ray,
    pub attenuation: Color,
}

impl ScatteredRay {
    pub fn scatter(hit: &HitRecord, incident_ray: &Ray) -> ScatteredRay {
        let mut scatter_direction: Vec3;
        match hit.material.material_type {
            MaterialType::Lambertian => {
                // Diffuse objects reflect light in random directions
                // Adding normal so that scatters are in general closer to the normal
                scatter_direction = Vec3::random_unit_vector() + hit.normal;
                // If the random unit vector is opposite to the normal, the scatter is the null
                // vector. To prevent troubles with this (NaN, Infinity ...) we use the normal
                // as the scatter direction in case the vector is null.
                if scatter_direction.near_zero() {
                    scatter_direction = hit.normal;
                }
            }
            MaterialType::Metal { fuzz } => {
                scatter_direction = (incident_ray.direction
                    - 2.0 * incident_ray.direction.dot(&hit.normal) * hit.normal)
                    .normalized()
                    + fuzz * Vec3::random_unit_vector();
            }
        }
        // Chck if the scatter is in the same direction as the normal
        // Otherwise, the scatter would be pointing inside the object.
        scatter_direction = if scatter_direction.dot(&hit.normal) >= 0. {
            scatter_direction
        } else {
            -1.0 * scatter_direction
        };
        let scattered_ray = Ray {
            origin: hit.p,
            direction: scatter_direction,
        };
        ScatteredRay {
            ray: scattered_ray,
            attenuation: hit.material.albedo,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub material_type: MaterialType,
    pub albedo: Color,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MaterialType {
    Lambertian,
    Metal { fuzz: f64 },
}

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
    pub material: Rc<Material>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, interval: Interval) -> Option<HitRecord> {
        // Finds t for quadratic equation x(t)^2 + y(t)^2 + z(t)^2 - r^2 = 0,
        // with:  ray = origin + t * direction
        // => t^2d.d - 2td.(C-Q) + (C-Q).(C-Q) - r^2 = 0
        // with d: ray direction,
        // C: sphere center
        // r: sphere radius
        // Q: ray origin
        let qc = self.center - ray.origin; // ray origin to sphere center
        let a = ray.direction.dot(&ray.direction);
        // h = b / -2, simplifies the equation of roots
        let h = ray.direction.dot(&qc);
        let c = qc.dot(&qc) - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant < 0. {
            return None;
        }

        let discriminant_sqrt = discriminant.sqrt();

        let mut root = (h - discriminant_sqrt) / a;
        if !interval.contains(root) {
            root = (h + discriminant_sqrt) / a;
            if !interval.contains(root) {
                return None;
            }
        }
        let t = root;
        let p = ray.at(root);
        let outward_normal = (p - self.center) / self.radius;
        let front_face = HitRecord::is_hit_from_front(ray, &outward_normal);
        // Make normal point outward the surface
        let normal = if front_face {
            outward_normal
        } else {
            -1.0 * outward_normal
        };
        Some(HitRecord {
            t,
            p,
            normal,
            front_face,
            material: Rc::clone(&self.material),
        })
    }
}

pub struct World<T: Hittable> {
    pub objects: Vec<Rc<T>>,
}

impl<T: Hittable> World<T> {
    pub fn add(&mut self, object: Rc<T>) {
        self.objects.push(object);
    }

    pub fn hit(&self, ray: &Ray, mut interval: Interval) -> Option<HitRecord> {
        let mut closest_hit: Option<HitRecord> = None;

        for object in &self.objects {
            if let Some(hit) = object.hit(ray, interval) {
                interval.max = hit.t;
                closest_hit = Some(hit);
            }
        }

        closest_hit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn hit_sphere() {
        let material_test = Rc::new(Material {
            material_type: MaterialType::Lambertian,
            albedo: Color::from([0.9, 0.9, 0.9]),
        });
        let sphere = Sphere {
            radius: 1.0,
            center: Point {
                x: 3.,
                y: 0.,
                z: 0.,
            },
            material: Rc::clone(&material_test),
        };
        let ray_should_hit = Ray {
            origin: Point {
                x: 0.,
                y: 0.,
                z: 0.,
            },
            direction: Vec3 {
                x: 1.0,
                y: 0.,
                z: 0.,
            },
        };
        assert_eq!(
            sphere.hit(&ray_should_hit, Interval { min: 0., max: 100. }),
            Some(HitRecord {
                p: Vec3 {
                    x: 2.,
                    y: 0.,
                    z: 0.
                },
                normal: Vec3 {
                    x: -1.,
                    y: 0.,
                    z: 0.
                },
                t: 2.,
                front_face: true,
                material: Rc::clone(&material_test),
            })
        )
    }
}

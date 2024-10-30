use crate::image::{Color, MAX_COLOR_CHANNEL_VALUE};
use std::{ops, rc::Rc};

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
    p: Point,
    pub normal: Vec3,
    t: f64,
    front_face: bool,
}

impl HitRecord {
    fn is_hit_from_front(ray: &Ray, outward_normal: &Vec3) -> bool {
        ray.direction.dot(outward_normal) < 0.
    }
}

pub trait Hittable {
    fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord>;
}

pub struct Sphere {
    pub center: Point,
    pub radius: f64,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord> {
        // Finds t for quadratic equation x(t)^2 + y(t)^2 + z(t)^2 - r^2 = 0
        // => t^2d.d - 2td.(C-Q) + (C-Q).(C-Q) - r^2 = 0
        // with d: direction,
        // C: sphere center
        // r: sphere radius
        // Q: ray origin
        let qc = self.center - ray.origin; // ray origin to sphere center
        let a = ray.direction.dot(&ray.direction);
        // h = b / -2
        let h = ray.direction.dot(&qc);
        let c = qc.dot(&qc) - self.radius * self.radius;
        let discriminant = h * h - a * c;
        if discriminant < 0. {
            return None;
        }

        let discriminant_sqrt = discriminant.sqrt();

        let mut root = (h - discriminant_sqrt) / a;
        if root < tmin || root > tmax {
            root = (h + discriminant_sqrt) / a;
            if root < tmin || root > tmax {
                return None;
            }
        }
        let t = root;
        let p = ray.at(root);
        let outward_normal = (p - self.center) / self.radius;
        let front_face = HitRecord::is_hit_from_front(ray, &outward_normal);
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
        })
    }
}

pub struct HittableList<T: Hittable> {
    pub objects: Vec<Rc<T>>,
}

impl<T: Hittable> HittableList<T> {
    pub fn add(&mut self, object: Rc<T>) {
        self.objects.push(object);
    }

    pub fn hit(&self, ray: &Ray, tmin: f64, tmax: f64) -> Option<HitRecord> {
        let mut closest_t_so_far = tmax;
        let mut closest_hit: Option<HitRecord> = None;

        for object in &self.objects {
            if let Some(hit) = object.hit(ray, tmin, closest_t_so_far) {
                closest_t_so_far = hit.t;
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
        let sphere = Sphere {
            radius: 1.0,
            center: Point {
                x: 3.,
                y: 0.,
                z: 0.,
            },
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
            sphere.hit(&ray_should_hit, 0., 100.),
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
            })
        )
    }
}

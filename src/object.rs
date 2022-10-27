use std::fmt::Formatter;
use core::fmt::Debug;

use nalgebra::Vector3;

#[derive(Clone, Copy)]
pub struct Material {
    pub diffuse_color: Vector3<f64>
}

impl Material {
    pub fn new(diffuse_color: Vector3<f64>) -> Material {
        Material {
            diffuse_color
        }
    }
}

impl Debug for Material {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Material")
            .field("diffuse_color", &self.diffuse_color)
            .finish()
    }
}

pub struct Sphere {
    pub center: Vector3<f64>,
    pub radius: f64,
    pub material: Material
}

impl Sphere {
    pub fn new(center: Vector3<f64>, radius: f64, material: Material) -> Sphere {
        Sphere {
            center,
            radius,
            material
        }
    }
    
    /// Ray-sphere intersection - return whether the ray intersects the sphere or not
    /// 
    /// ### Arguments
    /// 
    /// * `ray_origin` - The origin of the ray (point of origin)
    /// * `ray_direction` - The direction of the ray (normalized)
    /// * `t0` - The distance from the ray origin to the intersection point
    /// 
    /// ### Returns
    /// 
    /// bool - Whether the ray intersects the sphere or not
    /// 
    /// ### Notes
    /// 
    /// The ray is defined by the parametric equation:
    /// 
    /// `P(t) = ray_origin + t * ray_direction`
    /// 
    /// The sphere is defined by the equation:
    /// 
    /// `(P(t) - center) * (P(t) - center) = radius * radius`
    /// 
    /// ### Example
    /// 
    /// ```
    /// use nalgebra::Vector3;
    /// use raytracer::Sphere;
    /// 
    /// let sphere = Sphere::new(Vector3::new(0.0, 0.0, 0.0), 1.0);
    /// let ray_origin = Vector3::new(0.0, 0.0, 0.0);
    /// let ray_direction = Vector3::new(0.0, 0.0, 1.0);
    /// 
    /// let t0 = 0.0;
    /// let t1 = 0.0;
    /// 
    /// let intersects = sphere.intersect(ray_origin, ray_direction, &mut t0, &mut t1);
    /// 
    /// assert_eq!(intersects, true);
    /// ```
    #[warn(unused_assignments)]
    pub fn ray_intersect(&self, ray_origin: Vector3<f64>, dir: Vector3<f64>, mut t0: f64) -> (bool, f64) {
        let l: Vector3<f64> = self.center - ray_origin;
        let tca: f64 = l.dot(&dir);
        let d2: f64 = l.dot(&l) - tca * tca;
        if d2 > self.radius * self.radius { return (false, t0) }
        let thc: f64 = (self.radius * self.radius - d2).sqrt();
        t0 = tca - thc;
        let t1: f64 = tca + thc;
        if t0 < 0.0 { t0 = t1 }
        if t0 < 0.0 { return (false, t0) }
        (true, t0)
    }
}

impl Debug for Sphere {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Sphere {{ center: {:?}, radius: {:?}, material: {:?} }}", self.center, self.radius, self.material)
    }
}


pub struct Light {
    pub position: Vector3<f64>,
    pub intensity: f64
}

impl Light {
    pub fn new(position: Vector3<f64>, intensity: f64) -> Light {
        Light {
            position,
            intensity
        }
    }
}
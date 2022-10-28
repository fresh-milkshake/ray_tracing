mod image;
mod object;

use image::Image;
use object::Light;
use object::Material;
use object::Sphere;

use std::f64::consts::PI;

use futures;
use futures::executor::block_on;
use nalgebra::Vector3;


const BACKGROUND_COLOR: Vector3<f64> = Vector3::new(0.7, 0.8, 1.0);
const MAX_RECURSION_DEPTH: u32 = 6;

/// Returning the reflection of the vector `i` on the normal `n`
///
/// ### Arguments
///
/// * `i` - The incident vector
/// * `n` - The normal vector
///
/// ### Returns
///
/// Vector3<f64> - The reflected vector
///
fn reflect(i: Vector3<f64>, n: Vector3<f64>) -> Vector3<f64> {
    return i - n * 2.0 * (i.dot(&n));
}

/// Generate a ray from the camera to given object and evaluate the intersection
/// if there is an intersection, return the intersection point, normal and material
///
/// ### Arguments
///
/// * `ray_origin` - The origin of the ray (point of origin)
/// * `ray_direction` - The direction of the ray
/// * `spheres` - The sphere to intersect with
///
/// ### Returns
///
/// `(bool, Vector3<f64>, Vector3<f64>, Material)` - Whether the ray intersects the sphere or not,
/// the intersection point, the normal and the material
///
fn scene_intersect(
    ray_origin: Vector3<f64>,
    ray_direction: Vector3<f64>,
    spheres: &Vec<Sphere>,
) -> Option<(bool, Vector3<f64>, Vector3<f64>, Material)> {
    // initialize defaults
    let mut point = Vector3::default();
    let mut n = Vector3::default();
    let mut material = Material::default();

    // initialize minimum distance to max value of f64 (infinity used for comparison)
    let mut spheres_dist = std::f64::MAX;
    // iterate over all spheres in the scene
    // and evaluate the intersection with the ray
    // to get all its properties
    for sphere in spheres {
        let mut dist_i = 0.0; // distance to intersection
        let (is_intersect, dist_i0) = sphere.ray_intersect(ray_origin, ray_direction, dist_i);
        dist_i = dist_i0;
        if is_intersect && dist_i < spheres_dist {
            spheres_dist = dist_i; // update minimum distance with the current distance
            point = ray_origin + ray_direction * dist_i; // get the intersection point
            n = (point - sphere.center).normalize() as Vector3<f64>; // change the normal to point to center of the sphere
            material = sphere.material.clone(); // get material of the sphere
        }
    }
    Some((spheres_dist < 1000.0, point, n, material))
}

/// Compute the color of the ray at the point of intersection
///
/// ### Arguments
///
/// * `origin` - The origin of the ray (point of origin)
/// * `direction` - The direction of the ray (normalized)
/// * `spheres` - The list of spheres in the scene
/// * `lights` - The list of lights in the scene
///
/// ### Returns
///
/// Vector3<f64> - The color of the ray at the point of intersection
///
/// ### Notes
///
/// The ray is defined by the parametric equation:
///
/// `P(t) = origin + t * direction`
///
fn cast_ray(
    origin: Vector3<f64>,
    direction: Vector3<f64>,
    spheres: &Vec<Sphere>,
    lights: &Vec<Light>,
    depth: u32,
) -> Vector3<f64> {
    // check if the ray intersects any object
    // if it does, compute the intersection point, the normal and the color
    // if it doesn't, or if the maximum recursion depth has been reached (to avoid infinite recursion
    // when the ray hits the mirror surface), return the background color
    let (is_intersect, point, n, material) = scene_intersect(origin, direction, spheres).unwrap();
    if !is_intersect || depth > MAX_RECURSION_DEPTH {
        return BACKGROUND_COLOR;
    }

    // compute the reflection direction (not need to normalize because all vectors are already
    // normalized) and the color of the reflected ray (recursive call, cuz the reflected ray can
    // also reflect on other surfaces)
    let reflect_direction = reflect(direction, n);
    let reflect_origin = if reflect_direction.dot(&n) < 0.0 {
        point - n * 1e-3
    } else {
        point + n * 1e-3
    };
    let reflect_color = cast_ray(
        reflect_origin,
        reflect_direction,
        spheres,
        lights,
        depth + 1,
    );

    // compute color diffused by lambertian shading
    // lambertian shading is the simplest and most common shading model:
    // the color of a point is proportional to the cosine of the angle between the normal and the
    // light vector
    let mut diffuse_light_intensity = 0.0;
    let mut specular_light_intensity = 0.0;
    for light in lights {
        let light_direction = (light.position - point).normalize();
        let light_distance = (light.position - point).norm();

        // evaluate shadow origin and direction to check if the point is in shadow
        let shadow_origin = if light_direction.dot(&n) < 0.0 {
            point - n * 1e-3
        } else {
            point + n * 1e-3
        };

        // Check if the point lies in the shadow of the current light
        // If it does, skip this light
        // If it doesn't, add the contribution of the light to the diffuse and specular light
        let (shadow_intersect, shadow_pt, _, _) =
            scene_intersect(shadow_origin, light_direction, spheres).unwrap();
        if shadow_intersect && (shadow_pt - shadow_origin).norm() < light_distance {
            continue;
        }

        // add the contribution of the light to color diffusing
        let max_light = nalgebra::clamp(light_direction.dot(&n), 0.0, 1.0);
        diffuse_light_intensity += light.intensity * max_light;

        // process specular light
        let minus_ref = reflect(light_direction, n).dot(&direction);
        let power = nalgebra::clamp(minus_ref, 0.0, 1.0);
        specular_light_intensity += power.powf(material.specular_exponent) * light.intensity;
    }
    let mut diffuse_color = material.diffuse_color * diffuse_light_intensity * material.albedo[0];
    diffuse_color += Vector3::new(1.0, 1.0, 1.0) * specular_light_intensity * material.albedo[1];
    diffuse_color += reflect_color * material.albedo[2];
    diffuse_color
}

/// Asyncronous version of the `cast_ray` function with same arguments
/// except for the `i` and `j` arguments which are used to write the
/// pixel color to the image buffer
async fn cast_ray_async(
    origin: Vector3<f64>,
    direction: Vector3<f64>,
    spheres: &Vec<Sphere>,
    lights: &Vec<Light>,
    depth: u32,
    i: u32,
    j: u32,
) -> (u32, u32, Vector3<f64>) {
    (i, j, cast_ray(origin, direction, spheres, lights, depth))
}

/// Render a scene with spheres and lights
async fn render(
    width: u32,
    height: u32,
    fov: f64,
    spheres: &Vec<Sphere>,
    lights: &Vec<Light>,
) -> Vec<u8> {
    // `buffer` is a 1D array of pixels (RGB triplets) with the size of the image
    let mut buffer = vec![0; (width * height * 3) as usize];
    let mut tasks = Vec::new();

    for j in 0..height {
        for i in 0..width {
            // X and Y calculated from the camera's perspective by the formula
            // x = (2 * (i + 0.5) / width - 1) * tan(fov / 2) * width / height
            // y = -(2 * (j + 0.5) / height - 1) * tan(fov / 2)
            // z = -1
            let x =
                (2.0 * (i as f64 + 0.5) / width as f64 - 1.0) * (fov / 2.0).tan() * width as f64
                    / height as f64;
            let y = -(2.0 * (j as f64 + 0.5) / height as f64 - 1.0) * (fov / 2.0).tan();
            // The camera is at (0, 0, 0) and looks along the negative Z axis
            // The direction of the ray is the normalized vector from the camera to the pixel
            let direction = Vector3::new(x, y, -1.0).normalize();
            let task = cast_ray_async(Vector3::default(), direction, spheres, lights, 0, i, j);
            tasks.push(task);
        }
    }
    let results = futures::future::join_all(tasks).await;
    for (i, j, color) in results {
        let index = (i + j * width) as usize;
        buffer[index * 3] = (color.x * 255.0) as u8;
        buffer[index * 3 + 1] = (color.y * 255.0) as u8;
        buffer[index * 3 + 2] = (color.z * 255.0) as u8;
    }
    buffer
}

fn main() {
    let ivory = Material::new(
        Vector3::new(0.6, 0.3, 0.1),
        Vector3::new(0.4, 0.4, 0.3),
        50.0,
    );
    let red_rubber = Material::new(
        Vector3::new(0.9, 0.1, 0.0),
        Vector3::new(0.3, 0.1, 0.1),
        10.0,
    );
    let mirror = Material::new(
        Vector3::new(0.0, 10.0, 0.8),
        Vector3::new(1.0, 1.0, 1.0),
        1425.0,
    );

    let spheres = vec![
        Sphere::new(Vector3::new(-3.0, 0.0, -16.0), 2.0, ivory),
        Sphere::new(Vector3::new(-1.0, -1.5, -12.0), 2.0, red_rubber),
        Sphere::new(Vector3::new(1.5, -0.5, -18.0), 3.0, mirror),
        Sphere::new(Vector3::new(7.0, 5.0, -18.0), 4.0, mirror),
    ];

    let lights = vec![
        Light::new(Vector3::new(-20.0, 20.0, 20.0), 1.5),
        Light::new(Vector3::new(30.0, 50.0, -25.0), 1.8),
        Light::new(Vector3::new(30.0, 20.0, 30.0), 1.7),
    ];

    // size of resulting image
    let (width, height) = (1024, 768);
    // field of view in radians (90 degrees)
    let fov = PI / 2.0;
    let framebuffer: Vec<u8> = block_on(render(width, height, fov, &spheres, &lights));

    let mut image = Image::new(width, height);
    for j in 0..height {
        for i in 0..width {
            let pixel_index: usize = (j * width + i) as usize;
            let color: Vec<u8> = vec![
                framebuffer[pixel_index * 3],
                framebuffer[pixel_index * 3 + 1],
                framebuffer[pixel_index * 3 + 2],
            ];
            image.set_pixel(i, j, color);
        }
    }
    image.save("out.png");
}

mod image;
mod object;

use std::f64::consts::PI;

use image::Image;
use nalgebra::Vector3;

use object::Light;
use object::Material;
use object::Sphere;
// use rand::random;

const BACKGROUND_COLOR: Vector3<f64> = Vector3::new(0.7, 0.8, 1.0);
const MAX_RECURSION_DEPTH: u32 = 6;

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

    let mut spheres_dist = std::f64::MAX;
    for sphere in spheres {
        let mut dist_i = 0.0;
        let (is_intersect, dist_i0) = sphere.ray_intersect(ray_origin, ray_direction, dist_i);
        dist_i = dist_i0;
        if is_intersect && dist_i < spheres_dist {
            spheres_dist = dist_i;
            point = ray_origin + ray_direction * dist_i;
            n = (point - sphere.center).normalize() as Vector3<f64>;
            material = sphere.material.clone();
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
    let (is_intersect, point, n, material) = scene_intersect(origin, direction, spheres).unwrap();
    if !is_intersect || depth > MAX_RECURSION_DEPTH {
        return BACKGROUND_COLOR;
    }

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

    let mut diffuse_light_intensity = 0.0;
    let mut specular_light_intensity = 0.0;
    for light in lights {
        let light_direction = (light.position - point).normalize();
        let light_distance = (light.position - point).norm();

        // Check if the point lies in the shadow of the current light
        let shadow_origin = if light_direction.dot(&n) < 0.0 {
            point - n * 1e-3
        } else {
            point + n * 1e-3
        };

        let (shadow_intersect, shadow_pt, _, _) =
            scene_intersect(shadow_origin, light_direction, spheres).unwrap();
        if shadow_intersect && (shadow_pt - shadow_origin).norm() < light_distance {
            continue;
        }

        let max_light = nalgebra::clamp(light_direction.dot(&n), 0.0, 1.0);
        diffuse_light_intensity += light.intensity * max_light;

        let minus_ref = reflect(light_direction, n).dot(&direction);
        let power = nalgebra::clamp(minus_ref, 0.0, 1.0);
        specular_light_intensity += power.powf(material.specular_exponent) * light.intensity;
    }
    let mut diffuse_color = material.diffuse_color * diffuse_light_intensity * material.albedo[0];
    diffuse_color += Vector3::new(1.0, 1.0, 1.0) * specular_light_intensity * material.albedo[1];
    diffuse_color += reflect_color * material.albedo[2];
    diffuse_color
}

/// Render a scene with spheres and lights
fn render(spheres: &Vec<Sphere>, lights: &Vec<Light>) {
    // size of the resulting image in pixels
    let width: u32 = 1024;
    let height: u32 = 768;
    // field of view in radians (90 degrees)
    let fov: f64 = PI / 2.0;

    println!("Generating image...");
    // `framebuffer` is a 1D array of pixels (3D vectors, containing RGB values) that will be rendered
    // to an image. It is indexed as `framebuffer[y * width + x]` (row-major order).
    let mut framebuffer = vec![Vector3::new(0.0, 0.0, 0.0); (width * height) as usize];
    // go through each pixel of the image and cast a ray through it to
    // find the color of the pixel
    for j in 0..height {
        for i in 0..width {
            print!("\rScanlines remaining: {}\r", height - j - 1);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            // X and Y calculated from the camera's perspective by the formula
            // x = (2 * (i + 0.5) / width - 1) * tan(fov / 2) * width / height
            // y = -(2 * (j + 0.5) / height - 1) * tan(fov / 2)
            // z = -1
            let x: f64 =
                (2.0 * (i as f64 + 0.5) / width as f64 - 1.0) * (fov / 2.0).tan() * width as f64
                    / height as f64;
            let y: f64 = -(2.0 * (j as f64 + 0.5) / height as f64 - 1.0) * (fov / 2.0).tan();
            // The camera is at (0, 0, 0) and looks along the negative Z axis
            // The direction of the ray is the normalized vector from the camera to the pixel
            let dir: Vector3<f64> = Vector3::new(x, y, -1.0).normalize();
            framebuffer[(j * width + i) as usize] =
                cast_ray(Vector3::new(0.0, 0.0, 0.0), dir, spheres, lights, 0);
        }
    }

    // save framebuffer to file (.png format) via Image struct
    println!("Saving framebuffer to file");
    let mut image = Image::new(width, height);
    for j in 0..height {
        for i in 0..width {
            let pixel_index: usize = (j * width + i) as usize;
            // convert framebuffer values from floats to 0-255 integers
            // by multiplying by 255 and clamping to [0, 255]
            let color: Vec<u8> = vec![
                (255.0 * framebuffer[pixel_index].x) as u8,
                (255.0 * framebuffer[pixel_index].y) as u8,
                (255.0 * framebuffer[pixel_index].z) as u8,
            ];
            image.set_pixel(i, j, color);
        }
    }
    image.save("image.png");
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

    // const SPHERES_COUNT: i32 = 3;
    // generate spheres_count random spheres in the field of view
    // let mut spheres = Vec::new();
    // for _ in 0..SPHERES_COUNT {
    //     // choose x, y, z coordinates randomly, but make sure that the sphere is
    //     // inside the field of view
    //     let x = random::<f64>() * 2.0 - 1.0;
    //     let y = random::<f64>() * 2.0 - 1.0;
    //     let z = random::<f64>() * 2.0 - 1.0;
    //     let radius: f64 = random::<f64>() * 1.0 + 0.5;
    //     let material: Material;
    //     // generate a random integer between 0 and 2 and choose a material
    //     let choose_mat: i32 = random::<i32>() % 3;
    //     if choose_mat == 0 {
    //         material = ivory;
    //     } else if choose_mat == 1 {
    //         material = red_rubber;
    //     } else {
    //         material = mirror;
    //     }
    //     spheres.push(Sphere::new(Vector3::new(x, y, z), radius, material));
    // }

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

    render(&spheres, &lights);
}

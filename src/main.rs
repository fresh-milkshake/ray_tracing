mod image;
mod object;

use std::f64::consts::PI;

use image::Image;
use nalgebra::Vector3;

use object::Material;
use object::Sphere;
use object::Light;

const BACKGROUND_COLOR: Vector3<f64> = Vector3::new(0.7, 0.8, 1.0);

fn scene_intersect(
    ray_origin: Vector3<f64>,
    ray_direction: Vector3<f64>,
    spheres: &Vec<Sphere>,
    hit: &mut Vector3<f64>,
    n: &mut Vector3<f64>,
    material: &mut Material,
) -> bool {
    let mut spheres_dist = std::f64::MAX;
    for sphere in spheres {
        // println!("sphere: {:?}", sphere);
        let mut dist_i = 0.0;
        let (is_intersect, dist_i0) = sphere.ray_intersect(ray_origin, ray_direction, dist_i);
        dist_i = dist_i0;
        if is_intersect && dist_i < spheres_dist {
            spheres_dist = dist_i;
            *hit = ray_origin + ray_direction * dist_i;
            *n = (*hit - sphere.center).normalize() as Vector3<f64>;
            *material = sphere.material.clone();
        }
    }
    spheres_dist < 1000.0
}

fn cast_ray(origin: Vector3<f64>, direction: Vector3<f64>, spheres: &Vec<Sphere>, lights: &Vec<Light>) -> Vector3<f64> {
    let mut shadow_point = Vector3::new(0.0, 0.0, 0.0);
    let mut shadow_n = Vector3::new(0.0, 0.0, 0.0);
    let mut tmp_material = Material::new(Vector3::new(0.0, 0.0, 0.0));

    if !scene_intersect(origin, direction, spheres, &mut shadow_point, &mut shadow_n, &mut tmp_material) {
        return BACKGROUND_COLOR;
    }

    let mut diffuse_light_intensity = 0.0;
    for light in lights {
        let light_dir  = (light.position - shadow_point).normalize();
        let max_light = nalgebra::clamp(light_dir.dot(&shadow_n), 0.0, 1.0);
        diffuse_light_intensity += light.intensity * max_light;
    }
    let diffuse_color = tmp_material.diffuse_color * diffuse_light_intensity;
    // if diffuse_color == tmp_material.diffuse_color {
    //     println!("diffuse_color: {:?}", diffuse_color);
    //     panic!("diffuse_color == tmp_material.diffuse_color");
    // } else {
    //     println!("diffuse_color: {:?}", diffuse_color);
    //     println!("tmp_material.diffuse_color: {:?}", tmp_material.diffuse_color);
    // }
    diffuse_color
}

fn render(spheres: &Vec<Sphere>, lights: &Vec<Light>) {
    let width: u32 = 1024;
    let height: u32 = 768;
    let fov: f64 = PI / 2.0;
    
    let mut framebuffer = vec![Vector3::new(0.0, 0.0, 0.0); (width * height) as usize];

    let mut pixel_index: usize;
    for j in 0..height {
        for i in 0..width {
            let x: f64 = (2.0 * (i as f64 + 0.5) / width as f64 - 1.0)
                * (fov / 2.0).tan()
                * width as f64
                / height as f64;
            let y: f64 =  -(2.0 * (j as f64 + 0.5) / height as f64 - 1.0)
                * (fov / 2.0).tan();
            let dir: Vector3<f64> = Vector3::new(x, y, -1.0).normalize();
            pixel_index = (j * width + i) as usize;
            // println!("x: {}, y: {}, dir: {:?}, {:?}", i, j, dir, pixel_index);
            framebuffer[pixel_index] = cast_ray(Vector3::new(0.0, 0.0, 0.0), dir, spheres, lights);
        }
    }

    // save framebuffer to file
    let mut image = Image::new(width, height);
    for j in 0..height {
        for i in 0..width {
            let pixel_index: usize = (j * width + i) as usize;
            let color: Vec<u8> = vec![
                (255.0 * framebuffer[pixel_index].x) as u8,
                (255.0 * framebuffer[pixel_index].y) as u8,
                (255.0 * framebuffer[pixel_index].z) as u8,
            ];
            image.set_pixel(i, j, color);
        }
    }
    // find unique colors in image and print em out
    // let mut unique_colors = Vec::new();
    // for j in 0..height {
    //     for i in 0..width {
    //         let pixel_index: usize = (j * width + i) as usize;
    //         let color: Vec<f64> = vec![
    //             framebuffer[pixel_index].x,
    //             framebuffer[pixel_index].y,
    //             framebuffer[pixel_index].z,
    //         ];
    //         if !unique_colors.contains(&color) {
    //             unique_colors.push(color);
    //         }
    //     }
    // }
    // println!("unique colors: {:?}", unique_colors);
    image.save("image.png");
}

fn main() {
    let ivory = Material::new(Vector3::new(0.4, 0.4, 0.3));
    let red_rubber = Material::new(Vector3::new(0.3, 0.1, 0.1));

    let spheres = vec![
        Sphere::new(Vector3::new(-3.0, 0.0, -16.0), 2.0, ivory),
        Sphere::new(Vector3::new(-1.0, -1.5, -12.0), 2.0, red_rubber),
        Sphere::new(Vector3::new(1.5, -0.5, -18.0), 3.0, red_rubber),
        Sphere::new(Vector3::new(7.0, 5.0, -18.0), 4.0, ivory),
    ];

    let lights = vec![
        Light::new(Vector3::new(-20.0, 20.0, 20.0), 1.5)
    ];

    render(&spheres, &lights);
}

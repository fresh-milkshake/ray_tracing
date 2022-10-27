use std::path::Path;


pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Image {
        Image {
            width,
            height,
            data: vec![0; (width * height * 3) as usize],
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Vec<u8>) {
        let offset = (y * self.width * 3 + x * 3) as usize;
        self.data[offset] = color[0];
        self.data[offset + 1] = color[1];
        self.data[offset + 2] = color[2];
    }

    pub fn save(&self, filename: &str) {
        let path = Path::new(filename);
        let file = std::fs::File::create(&path).unwrap();
        let ref mut w = std::io::BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width, self.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&self.data).unwrap();
        writer.finish().unwrap();
    }
}
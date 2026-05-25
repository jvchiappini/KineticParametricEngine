use kpe_schema::material::ProceduralMaterial;

pub struct MaterialGenerator;

impl MaterialGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_texture(&self, material: &ProceduralMaterial, width: u32, height: u32) -> Vec<u8> {
        let base_color = self.parse_hex_color(&material.base.color);

        let mut pixels = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let noise = self.simple_noise(x, y, width, height);
                let r = (base_color[0] as f64 * noise).min(255.0) as u8;
                let g = (base_color[1] as f64 * noise).min(255.0) as u8;
                let b = (base_color[2] as f64 * noise).min(255.0) as u8;

                pixels.push(r);
                pixels.push(g);
                pixels.push(b);
                pixels.push(255);
            }
        }

        pixels
    }

    fn parse_hex_color(&self, hex: &str) -> [f64; 3] {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64;
            [r, g, b]
        } else {
            [128.0, 128.0, 128.0]
        }
    }

    fn simple_noise(&self, x: u32, y: u32, width: u32, height: u32) -> f64 {
        let fx = x as f64 / width as f64;
        let fy = y as f64 / height as f64;
        let n = (fx * 12.9898 + fy * 78.233).sin() * 43758.5453;
        0.85 + 0.15 * (n - n.floor() * 2.0 - 1.0).abs()
    }
}

impl Default for MaterialGenerator {
    fn default() -> Self {
        Self::new()
    }
}

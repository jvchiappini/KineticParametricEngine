use kpe_schema::material::UvMode;

pub struct UvMapper;

impl UvMapper {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_uvs(
        &self,
        vertices: &[[f64; 3]],
        uv_mode: &UvMode,
        uv_scale: [f64; 2],
    ) -> Vec<[f64; 2]> {
        match uv_mode {
            UvMode::WorldScale => self.world_scale_uvs(vertices, uv_scale),
            UvMode::ObjectRelative => self.object_relative_uvs(vertices),
            UvMode::Planar => self.planar_uvs(vertices, uv_scale),
        }
    }

    fn world_scale_uvs(&self, vertices: &[[f64; 3]], uv_scale: [f64; 2]) -> Vec<[f64; 2]> {
        vertices.iter().map(|v| {
            let u = v[0] / uv_scale[0];
            let v_uv = v[1] / uv_scale[1];
            [u, v_uv]
        }).collect()
    }

    fn object_relative_uvs(&self, vertices: &[[f64; 3]]) -> Vec<[f64; 2]> {
        let min_x = vertices.iter().map(|v| v[0]).fold(f64::INFINITY, f64::min);
        let max_x = vertices.iter().map(|v| v[0]).fold(f64::NEG_INFINITY, f64::max);
        let min_y = vertices.iter().map(|v| v[1]).fold(f64::INFINITY, f64::min);
        let max_y = vertices.iter().map(|v| v[1]).fold(f64::NEG_INFINITY, f64::max);
        let range_x = max_x - min_x;
        let range_y = max_y - min_y;

        vertices.iter().map(|v| {
            let u = if range_x > 0.0 { (v[0] - min_x) / range_x } else { 0.0 };
            let v_uv = if range_y > 0.0 { (v[1] - min_y) / range_y } else { 0.0 };
            [u, v_uv]
        }).collect()
    }

    fn planar_uvs(&self, vertices: &[[f64; 3]], uv_scale: [f64; 2]) -> Vec<[f64; 2]> {
        self.world_scale_uvs(vertices, uv_scale)
    }
}

impl Default for UvMapper {
    fn default() -> Self {
        Self::new()
    }
}

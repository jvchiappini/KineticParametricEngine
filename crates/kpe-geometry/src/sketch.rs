use kpe_schema::geometry::Sketch2D;

pub struct SketchEngine;

impl SketchEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn rectangle(&self, width: f64, height: f64) -> Sketch2D {
        let half_w = width / 2.0;
        let half_h = height / 2.0;

        Sketch2D {
            contours: vec![vec![
                [-half_w, -half_h],
                [half_w, -half_h],
                [half_w, half_h],
                [-half_w, half_h],
                [-half_w, -half_h],
            ]],
        }
    }

    pub fn circle(&self, radius: f64, segments: u32) -> Sketch2D {
        let mut contour = Vec::new();
        for i in 0..=segments {
            let angle = (i as f64 / segments as f64) * std::f64::consts::TAU;
            contour.push([radius * angle.cos(), radius * angle.sin()]);
        }
        Sketch2D {
            contours: vec![contour],
        }
    }

    pub fn extrude(&self, sketch: &Sketch2D, _height: f64) -> Sketch2D {
        sketch.clone()
    }

    pub fn revolve(&self, sketch: &Sketch2D, _angle: f64) -> Sketch2D {
        sketch.clone()
    }
}

impl Default for SketchEngine {
    fn default() -> Self {
        Self::new()
    }
}

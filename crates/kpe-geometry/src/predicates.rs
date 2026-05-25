use glam::DVec3;

pub const EPSILON: f64 = 1e-12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Negative = -1,
    Zero = 0,
    Positive = 1,
}

impl Sign {
    pub fn is_positive(self) -> bool {
        self == Sign::Positive
    }
    pub fn is_negative(self) -> bool {
        self == Sign::Negative
    }
    pub fn is_zero(self) -> bool {
        self == Sign::Zero
    }
    pub fn is_non_negative(self) -> bool {
        self != Sign::Negative
    }
    pub fn is_non_positive(self) -> bool {
        self != Sign::Positive
    }
}

pub fn sign(value: f64) -> Sign {
    if value > EPSILON {
        Sign::Positive
    } else if value < -EPSILON {
        Sign::Negative
    } else {
        Sign::Zero
    }
}

pub fn orient2d(pa: DVec3, pb: DVec3, pc: DVec3) -> Sign {
    let det = (pa.x - pc.x) * (pb.y - pc.y) - (pb.x - pc.x) * (pa.y - pc.y);
    sign(det)
}

pub fn orient3d(pa: DVec3, pb: DVec3, pc: DVec3, pd: DVec3) -> Sign {
    let ax = pa.x - pd.x; let ay = pa.y - pd.y; let az = pa.z - pd.z;
    let bx = pb.x - pd.x; let by = pb.y - pd.y; let bz = pb.z - pd.z;
    let cx = pc.x - pd.x; let cy = pc.y - pd.y; let cz = pc.z - pd.z;
    let det = ax * (by * cz - bz * cy)
            - ay * (bx * cz - bz * cx)
            + az * (bx * cy - by * cx);
    sign(det)
}

pub fn points_are_on_same_side(pts: &[DVec3; 3], plane_point: DVec3, plane_normal: DVec3) -> bool {
    let ref_sign = sign(plane_normal.dot(pts[0] - plane_point));
    if ref_sign == Sign::Zero {
        return true;
    }
    for i in 1..3 {
        let s = sign(plane_normal.dot(pts[i] - plane_point));
        if s != Sign::Zero && s != ref_sign {
            return false;
        }
    }
    true
}

pub fn point_in_triangle(p: DVec3, a: DVec3, b: DVec3, c: DVec3) -> bool {
    let v0 = c - a;
    let v1 = b - a;
    let v2 = p - a;
    let dot00 = v0.dot(v0);
    let dot01 = v0.dot(v1);
    let dot02 = v0.dot(v2);
    let dot11 = v1.dot(v1);
    let dot12 = v1.dot(v2);

    let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

    u >= -EPSILON && v >= -EPSILON && u + v <= 1.0 + EPSILON
}

pub fn point_in_triangle_barycentric(p: DVec3, a: DVec3, b: DVec3, c: DVec3) -> Option<[f64; 3]> {
    let v0 = c - a;
    let v1 = b - a;
    let v2 = p - a;
    let dot00 = v0.dot(v0);
    let dot01 = v0.dot(v1);
    let dot02 = v0.dot(v2);
    let dot11 = v1.dot(v1);
    let dot12 = v1.dot(v2);

    let denom = dot00 * dot11 - dot01 * dot01;
    if denom.abs() < EPSILON {
        return None;
    }
    let inv_denom = 1.0 / denom;
    let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
    let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
    let w = 1.0 - u - v;

    if u >= -EPSILON && v >= -EPSILON && w >= -EPSILON {
        Some([u, v, w])
    } else {
        None
    }
}

pub fn triangle_normal(a: DVec3, b: DVec3, c: DVec3) -> DVec3 {
    (b - a).cross(c - a).normalize()
}

pub fn triangle_area(a: DVec3, b: DVec3, c: DVec3) -> f64 {
    (b - a).cross(c - a).length() * 0.5
}

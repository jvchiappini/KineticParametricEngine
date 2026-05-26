use glam::DVec2;

#[derive(Debug, Clone)]
pub enum BooleanOp {
    Union,
    Subtract,
    Intersect,
}

pub fn boolean_contours(
    contours_a: &[Vec<DVec2>],
    contours_b: &[Vec<DVec2>],
    op: BooleanOp,
) -> Vec<Vec<DVec2>> {
    match op {
        BooleanOp::Union => union_contours(contours_a, contours_b),
        BooleanOp::Subtract => subtract_contours(contours_a, contours_b),
        BooleanOp::Intersect => intersect_contours(contours_a, contours_b),
    }
}

fn union_contours(contours_a: &[Vec<DVec2>], contours_b: &[Vec<DVec2>]) -> Vec<Vec<DVec2>> {
    let mut result = Vec::new();

    let mut all_outside_a = true;
    for contour in contours_b {
        if !contour.is_empty() && point_in_contour(contour[0], contours_a) {
            all_outside_a = false;
            break;
        }
    }

    let mut all_outside_b = true;
    for contour in contours_a {
        if !contour.is_empty() && point_in_contour(contour[0], contours_b) {
            all_outside_b = false;
            break;
        }
    }

    if all_outside_a && all_outside_b {
        result.extend_from_slice(contours_a);
        result.extend_from_slice(contours_b);
        return result;
    }

    for c in contours_a {
        if !contour_inside_any(c, contours_b) {
            result.push(c.clone());
        }
    }
    for c in contours_b {
        if !contour_inside_any(c, contours_a) {
            result.push(c.clone());
        }
    }

    result
}

fn subtract_contours(contours_a: &[Vec<DVec2>], contours_b: &[Vec<DVec2>]) -> Vec<Vec<DVec2>> {
    let mut result: Vec<Vec<DVec2>> = Vec::new();

    for c in contours_a {
        if !contour_inside_any(c, contours_b) {
            result.push(c.clone());
        }
    }

    result
}

fn intersect_contours(contours_a: &[Vec<DVec2>], contours_b: &[Vec<DVec2>]) -> Vec<Vec<DVec2>> {
    let mut result: Vec<Vec<DVec2>> = Vec::new();

    for c in contours_a {
        if contour_inside_any(c, contours_b) {
            result.push(c.clone());
        }
    }

    result
}

fn point_in_contour(p: DVec2, contours: &[Vec<DVec2>]) -> bool {
    for contour in contours {
        if contour.len() < 3 {
            continue;
        }
        if point_in_polygon(p, contour) {
            return true;
        }
    }
    false
}

fn point_in_polygon(p: DVec2, polygon: &[DVec2]) -> bool {
    let n = polygon.len();
    if n < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let yi = polygon[i].y;
        let yj = polygon[j].y;
        let xi = polygon[i].x;
        let xj = polygon[j].x;

        if ((yi > p.y) != (yj > p.y))
            && (p.x < (xj - xi) * (p.y - yi) / (yj - yi) + xi)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn contour_inside_any(contour: &[DVec2], others: &[Vec<DVec2>]) -> bool {
    if contour.is_empty() {
        return false;
    }
    let centroid: DVec2 = contour.iter().sum::<DVec2>() / contour.len() as f64;
    point_in_contour(centroid, others)
}

pub fn simplify_contour(contour: &[DVec2], min_dist: f64) -> Vec<DVec2> {
    if contour.len() <= 2 {
        return contour.to_vec();
    }

    let mut result = Vec::new();
    result.push(contour[0]);

    for i in 1..contour.len() {
        let last = result[result.len() - 1];
        if contour[i].distance(last) > min_dist {
            result.push(contour[i]);
        }
    }

    let first = result[0];
    let last = result[result.len() - 1];
    if result.len() > 1 && first.distance(last) < min_dist {
        result.pop();
    }

    result
}

pub fn offset_contour(contour: &[DVec2], _offset: f64) -> Vec<DVec2> {
    contour.to_vec()
}

pub fn extrude_contour_to_3d(
    contour: &[DVec2],
    distance: f64,
    cap_bottom: bool,
    cap_top: bool,
) -> (Vec<[f64; 3]>, Vec<[u32; 3]>) {
    let n = contour.len();
    if n < 3 {
        return (vec![], vec![]);
    }

    let mut verts: Vec<[f64; 3]> = Vec::new();
    let mut tris: Vec<[u32; 3]> = Vec::new();

    for p in contour {
        verts.push([p.x, p.y, 0.0]);
    }

    let top_start = n as u32;
    for p in contour {
        verts.push([p.x, p.y, distance]);
    }

    if cap_bottom && n >= 3 {
        for i in 1..n - 1 {
            tris.push([0, i as u32 + 1, i as u32]);
        }
    }

    if cap_top && n >= 3 {
        for i in 1..n - 1 {
            tris.push([top_start, top_start + i as u32, top_start + i as u32 + 1]);
        }
    }

    for i in 0..n {
        let next = (i + 1) % n;
        let b0 = i as u32;
        let b1 = next as u32;
        let t0 = top_start + i as u32;
        let t1 = top_start + next as u32;
        tris.push([b0, b1, t1]);
        tris.push([b0, t1, t0]);
    }

    (verts, tris)
}

use glam::DVec2;
use std::collections::{HashMap, HashSet};

const EPSILON: f64 = 1e-6;

#[derive(Clone, Copy, Debug)]
pub struct LineSegment {
    pub start: DVec2,
    pub end: DVec2,
}

/// Computes the planar graph from intersecting segments and extracts all independent minimal faces.
pub fn extract_regions(segments: &[LineSegment]) -> Vec<Vec<DVec2>> {
    let mut split_segments = split_at_intersections(segments);
    
    // Remove zero-length segments
    split_segments.retain(|s| s.start.distance_squared(s.end) > EPSILON * EPSILON);

    let mut nodes: Vec<DVec2> = Vec::new();
    let mut get_node_id = |p: DVec2| -> usize {
        for (i, node) in nodes.iter().enumerate() {
            if node.distance_squared(p) < EPSILON * EPSILON {
                return i;
            }
        }
        nodes.push(p);
        nodes.len() - 1
    };

    let mut edges: Vec<(usize, usize)> = Vec::new();
    for seg in &split_segments {
        let i = get_node_id(seg.start);
        let j = get_node_id(seg.end);
        if i != j {
            if !edges.contains(&(i, j)) && !edges.contains(&(j, i)) {
                edges.push((i, j));
            }
        }
    }

    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(u, v) in &edges {
        adj.entry(u).or_default().push(v);
        adj.entry(v).or_default().push(u);
    }

    let mut faces = Vec::new();
    let mut visited_half_edges: HashSet<(usize, usize)> = HashSet::new();

    for &u in adj.keys() {
        let neighbors = adj.get(&u).unwrap();
        let mut sorted_nbs = neighbors.clone();
        
        // Sort neighbors polarly around u (for left-most turn logic)
        sorted_nbs.sort_by(|&a, &b| {
            let angle_a = (nodes[a].y - nodes[u].y).atan2(nodes[a].x - nodes[u].x);
            let angle_b = (nodes[b].y - nodes[u].y).atan2(nodes[b].x - nodes[u].x);
            angle_a.partial_cmp(&angle_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        for &v in &sorted_nbs {
            if visited_half_edges.contains(&(u, v)) {
                continue;
            }

            let mut face = Vec::new();
            let mut curr = u;
            let mut next = v;
            let mut valid = true;
            let mut attempts = 0;
            
            while !visited_half_edges.contains(&(curr, next)) && attempts < edges.len() * 2 {
                visited_half_edges.insert((curr, next));
                face.push(curr);

                // Calculate incoming angle exactly backwards from current progression
                let incoming_angle = (nodes[curr].y - nodes[next].y).atan2(nodes[curr].x - nodes[next].x);
                let nbs = adj.get(&next).unwrap();
                
                let mut best_angle_diff = std::f64::consts::TAU * 2.0;
                let mut best_nb = None;

                for &nb in nbs {
                    if nb == curr && nbs.len() > 1 { continue; } // Exclude backtracking unless it's a dead end
                    let out_angle = (nodes[nb].y - nodes[next].y).atan2(nodes[nb].x - nodes[next].x);
                    
                    let mut diff = out_angle - incoming_angle;
                    while diff < 0.0 { diff += std::f64::consts::TAU; }
                    while diff >= std::f64::consts::TAU { diff -= std::f64::consts::TAU; }
                    
                    if diff < best_angle_diff {
                        best_angle_diff = diff;
                        best_nb = Some(nb);
                    }
                }

                if let Some(nb) = best_nb {
                    curr = next;
                    next = nb;
                } else {
                    valid = false;
                    break;
                }
                attempts += 1;

                if curr == u && next == v {
                    break; // Closed the loop successfully
                }
            }

            if valid && face.len() >= 3 {
                let polygon: Vec<DVec2> = face.iter().map(|&idx| nodes[idx]).collect();
                // Filter out the external infinite bounding region by enforcing positive CCW area.
                let area = polygon_area(&polygon);
                if area > EPSILON {
                    faces.push(polygon);
                }
            }
        }
    }

    faces
}

pub fn polygon_area(pts: &[DVec2]) -> f64 {
    let mut area = 0.0;
    let n = pts.len();
    for i in 0..n {
        let j = (i + 1) % n;
        area += pts[i].x * pts[j].y - pts[j].x * pts[i].y;
    }
    area / 2.0
}

fn split_at_intersections(segments: &[LineSegment]) -> Vec<LineSegment> {
    let mut result = Vec::new();
    for seg in segments {
        let mut split_points = vec![seg.start, seg.end];
        for other in segments {
            if let Some(pt) = segment_intersection(seg, other) {
                split_points.push(pt);
            }
        }
        
        split_points.sort_by(|a, b| {
            let dist_a = a.distance_squared(seg.start);
            let dist_b = b.distance_squared(seg.start);
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut i = 0;
        while i < split_points.len() - 1 {
            if split_points[i].distance_squared(split_points[i+1]) < EPSILON * EPSILON {
                split_points.remove(i+1);
            } else {
                i += 1;
            }
        }

        for j in 0..split_points.len() - 1 {
            result.push(LineSegment { start: split_points[j], end: split_points[j+1] });
        }
    }
    result
}

fn segment_intersection(a: &LineSegment, b: &LineSegment) -> Option<DVec2> {
    let p = a.start;
    let r = a.end - a.start;
    let q = b.start;
    let s = b.end - b.start;

    let rxs = cross(r, s);
    let qminusp_xr = cross(q - p, r);

    if rxs.abs() < EPSILON {
        return None;
    }

    let t = cross(q - p, s) / rxs;
    let u = qminusp_xr / rxs;

    if t >= -EPSILON && t <= 1.0 + EPSILON && u >= -EPSILON && u <= 1.0 + EPSILON {
        Some(p + r * t)
    } else {
        None
    }
}

fn cross(a: DVec2, b: DVec2) -> f64 {
    a.x * b.y - a.y * b.x
}

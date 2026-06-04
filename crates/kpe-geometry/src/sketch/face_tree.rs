use glam::DVec2;
use kpe_schema::geometry::{FaceNode, SketchFillState};
use crate::sketch::dcel::polygon_area;
use crate::sketch::boolean::point_in_polygon;

/// Builds a hierarchical face tree from a flat list of closed boundary loops
/// discovered by the DCEL planar graph walker.
///
/// # Algorithm
///
/// 1. Compute the signed area and centroid of each face.
/// 2. Sort faces by area descending (larger outer boundaries first).
/// 3. For each face, find the **smallest** face whose polygon contains its
///    centroid — that face becomes its parent.
/// 4. Assign a default `SketchFillState` based on the Even-Odd depth rule
///    (odd depth → `Hole`, even depth → `Solid`).
///
/// The resulting tree can be stored in `SketchDef.face_hierarchy` and
/// overridden by the user on a per-node basis.
pub struct FaceTreeBuilder;

impl FaceTreeBuilder {
    /// Build the face hierarchy from a flat list of closed loops.
    ///
    /// Each face in `faces` must be a closed polygon (first vertex ≠ last
    /// vertex is allowed; the closure is implicit).  Loops with fewer than
    /// 3 vertices are skipped.
    pub fn build(faces: &[Vec<DVec2>]) -> Vec<FaceNode> {
        if faces.is_empty() {
            return Vec::new();
        }

        // Compute metadata for each face.
        let mut entries: Vec<FaceEntry> = faces
            .iter()
            .enumerate()
            .filter(|(_, f)| f.len() >= 3)
            .map(|(i, f)| {
                let centroid: DVec2 = f.iter().sum::<DVec2>() / f.len() as f64;
                FaceEntry {
                    id: format!("face_{}", i),
                    contour: f.iter().map(|p| [p.x, p.y]).collect(),
                    centroid,
                    area: polygon_area(f).abs(),
                    depth: 0,
                    parent_idx: None,
                }
            })
            .collect();

        if entries.is_empty() {
            return Vec::new();
        }

        // Sort by area descending so we process outer boundaries first.
        entries.sort_by(|a, b| b.area.partial_cmp(&a.area).unwrap_or(std::cmp::Ordering::Equal));

        // Build containment tree.
        for i in 0..entries.len() {
            let (centroid, area_i) = {
                let e = &entries[i];
                (e.centroid, e.area)
            };

            // Find the smallest face that contains this face's centroid.
            let mut best_parent: Option<usize> = None;
            let mut best_area = f64::INFINITY;

            for j in 0..entries.len() {
                if i == j {
                    continue;
                }
                let other = &entries[j];
                // A face cannot contain itself, and we only consider faces
                // with larger area (outer boundaries enclose smaller loops).
                if other.area <= area_i {
                    continue;
                }
                let poly: Vec<DVec2> = other.contour.iter().map(|p| DVec2::new(p[0], p[1])).collect();
                if point_in_polygon(centroid, &poly) {
                    if other.area < best_area {
                        best_area = other.area;
                        best_parent = Some(j);
                    }
                }
            }

            if let Some(parent) = best_parent {
                entries[i].parent_idx = Some(parent);
                entries[i].depth = entries[parent].depth + 1;
            }
        }

        // Group children under their parents.
        let roots = Self::build_tree_nodes(&entries);
        Self::assign_default_fill(&roots, 0)
    }

    /// Build the tree structure from the flat parent-pointer representation.
    ///
    /// First collects parent→children relationships, then recursively
    /// assembles `FaceNode`s so children are attached before any node
    /// is cloned.
    fn build_tree_nodes(entries: &[FaceEntry]) -> Vec<FaceNode> {
        // Build parent → children index.
        use std::collections::HashMap;
        let mut children_of: HashMap<usize, Vec<usize>> = HashMap::new();
        for (i, e) in entries.iter().enumerate() {
            if let Some(parent) = e.parent_idx {
                children_of.entry(parent).or_default().push(i);
            }
        }

        // Recursively build a node and its descendants.
        fn build_node(idx: usize, entries: &[FaceEntry], children_of: &HashMap<usize, Vec<usize>>) -> FaceNode {
            let e = &entries[idx];
            FaceNode {
                id: e.id.clone(),
                contour: e.contour.clone(),
                fill_state: None,
                children: children_of
                    .get(&idx)
                    .map(|child_indices| {
                        child_indices
                            .iter()
                            .map(|&ci| build_node(ci, entries, children_of))
                            .collect()
                    })
                    .unwrap_or_default(),
            }
        }

        // Roots are entries with no parent.
        entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.parent_idx.is_none())
            .map(|(i, _)| build_node(i, entries, &children_of))
            .collect()
    }

    /// Recursively assign default fill states based on Even-Odd depth rule.
    ///
    /// Depth 0 (outer boundaries) → `Solid`
    /// Depth 1 (first nesting)   → `Hole`
    /// Depth 2 (second nesting)  → `Solid`
    /// ... alternating.
    ///
    /// Only nodes with `fill_state: None` receive the default.
    fn assign_default_fill(nodes: &[FaceNode], depth: u32) -> Vec<FaceNode> {
        nodes
            .iter()
            .map(|n| {
                let fill = if n.fill_state.is_some() {
                    n.fill_state
                } else if depth % 2 == 0 {
                    Some(SketchFillState::Solid)
                } else {
                    Some(SketchFillState::Hole)
                };
                FaceNode {
                    id: n.id.clone(),
                    contour: n.contour.clone(),
                    fill_state: fill,
                    children: Self::assign_default_fill(&n.children, depth + 1),
                }
            })
            .collect()
    }
}

/// Internal metadata for a single face during tree building.
struct FaceEntry {
    id: String,
    contour: Vec<[f64; 2]>,
    centroid: DVec2,
    area: f64,
    depth: u32,
    parent_idx: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f64, y: f64, w: f64, h: f64) -> Vec<DVec2> {
        vec![DVec2::new(x, y), DVec2::new(x + w, y),
             DVec2::new(x + w, y + h), DVec2::new(x, y + h)]
    }

    fn circle(cx: f64, cy: f64, r: f64, segs: u32) -> Vec<DVec2> {
        (0..segs).map(|i| {
            let a = (i as f64 / segs as f64) * std::f64::consts::TAU;
            DVec2::new(cx + r * a.cos(), cy + r * a.sin())
        }).collect()
    }

    #[test]
    fn test_single_face_is_solid() {
        let tree = FaceTreeBuilder::build(&[rect(0.0, 0.0, 10.0, 10.0)]);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].fill_state, Some(SketchFillState::Solid));
        assert!(tree[0].children.is_empty());
    }

    #[test]
    fn test_square_with_circle_hole() {
        let tree = FaceTreeBuilder::build(&[rect(-5.0, -5.0, 10.0, 10.0),
                                            circle(0.0, 0.0, 2.0, 16)]);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].fill_state, Some(SketchFillState::Solid));
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].fill_state, Some(SketchFillState::Hole));
    }

    #[test]
    fn test_three_levels_deep() {
        let tree = FaceTreeBuilder::build(&[rect(-10.0, -10.0, 20.0, 20.0),
                                            rect(-3.0, -3.0, 6.0, 6.0),
                                            circle(0.0, 0.0, 8.0, 16)]);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].fill_state, Some(SketchFillState::Hole));
        assert_eq!(tree[0].children[0].children.len(), 1);
        assert_eq!(tree[0].children[0].children[0].fill_state,
                   Some(SketchFillState::Solid));
    }

    #[test]
    fn test_two_separate_squares() {
        let tree = FaceTreeBuilder::build(&[rect(0.0, 0.0, 5.0, 5.0),
                                            rect(10.0, 10.0, 5.0, 5.0)]);
        assert_eq!(tree.len(), 2);
        for root in &tree {
            assert_eq!(root.fill_state, Some(SketchFillState::Solid));
            assert!(root.children.is_empty());
        }
    }

    #[test]
    fn test_square_with_two_holes() {
        let tree = FaceTreeBuilder::build(&[rect(-10.0, -10.0, 20.0, 20.0),
                                            circle(-5.0, -5.0, 2.0, 12),
                                            circle(5.0, 5.0, 2.0, 12)]);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 2);
        for child in &tree[0].children {
            assert_eq!(child.fill_state, Some(SketchFillState::Hole));
        }
    }

    #[test]
    fn test_centroid_on_boundary_is_safe() {
        let tiny = vec![DVec2::new(5.0, 5.0), DVec2::new(0.0, 0.0),
                        DVec2::new(5.0, 0.0)];
        let tree = FaceTreeBuilder::build(&[rect(0.0, 0.0, 10.0, 10.0), tiny]);
        assert!(!tree.is_empty());
    }
}

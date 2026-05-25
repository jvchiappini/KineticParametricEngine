use glam::DVec3;

#[derive(Debug, Clone)]
pub struct AABB {
    pub min: DVec3,
    pub max: DVec3,
}

impl AABB {
    pub fn new(min: DVec3, max: DVec3) -> Self {
        Self { min, max }
    }

    pub fn from_vertices(vertices: &[DVec3]) -> Self {
        let mut min = DVec3::splat(f64::INFINITY);
        let mut max = DVec3::splat(f64::NEG_INFINITY);
        for v in vertices {
            min = min.min(*v);
            max = max.max(*v);
        }
        Self { min, max }
    }

    pub fn from_triangle(a: DVec3, b: DVec3, c: DVec3) -> Self {
        Self {
            min: a.min(b).min(c),
            max: a.max(b).max(c),
        }
    }

    pub fn centroid(&self) -> DVec3 {
        (self.min + self.max) * 0.5
    }

    pub fn surface_area(&self) -> f64 {
        let d = self.max - self.min;
        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    pub fn intersect(&self, other: &AABB) -> bool {
        (self.min.x <= other.max.x && self.max.x >= other.min.x)
            && (self.min.y <= other.max.y && self.max.y >= other.min.y)
            && (self.min.z <= other.max.z && self.max.z >= other.min.z)
    }

    pub fn union(&self, other: &AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn diagonal(&self) -> f64 {
        (self.max - self.min).length()
    }

    pub fn contains_point(&self, p: DVec3) -> bool {
        p.x >= self.min.x - 1e-9 && p.x <= self.max.x + 1e-9
            && p.y >= self.min.y - 1e-9 && p.y <= self.max.y + 1e-9
            && p.z >= self.min.z - 1e-9 && p.z <= self.max.z + 1e-9
    }
}

#[derive(Debug, Clone)]
pub struct BVHTriangle {
    pub vertices: [DVec3; 3],
    pub aabb: AABB,
    pub centroid: DVec3,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub enum BVHNode {
    Leaf {
        aabb: AABB,
        triangles: Vec<BVHTriangle>,
    },
    Internal {
        aabb: AABB,
        left: Box<BVHNode>,
        right: Box<BVHNode>,
        split_axis: usize,
    },
}

impl BVHNode {
    pub fn aabb(&self) -> &AABB {
        match self {
            BVHNode::Leaf { aabb, .. } => aabb,
            BVHNode::Internal { aabb, .. } => aabb,
        }
    }
}

pub struct BVH {
    pub root: BVHNode,
    pub triangles: Vec<BVHTriangle>,
}

impl BVH {
    pub fn build(vertices: &[DVec3], triangles: &[[u32; 3]]) -> Self {
        let bvh_tris: Vec<BVHTriangle> = triangles.iter().enumerate().map(|(idx, tri)| {
            let v = [
                vertices[tri[0] as usize],
                vertices[tri[1] as usize],
                vertices[tri[2] as usize],
            ];
            let aabb = AABB::from_triangle(v[0], v[1], v[2]);
            let centroid = (v[0] + v[1] + v[2]) / 3.0;
            BVHTriangle { vertices: v, aabb, centroid, index: idx }
        }).collect();

        let root = Self::build_recursive(&bvh_tris, 0);
        Self { root, triangles: bvh_tris }
    }

    fn build_recursive(tris: &[BVHTriangle], depth: usize) -> BVHNode {
        if tris.len() <= 4 || depth > 32 {
            let mut aabb = AABB::from_triangle(tris[0].vertices[0], tris[0].vertices[1], tris[0].vertices[2]);
            for t in tris {
                aabb = aabb.union(&t.aabb);
            }
            return BVHNode::Leaf {
                aabb,
                triangles: tris.to_vec(),
            };
        }

        let mut aabb = AABB::from_triangle(tris[0].vertices[0], tris[0].vertices[1], tris[0].vertices[2]);
        for t in tris {
            aabb = aabb.union(&t.aabb);
        }

        let extent = aabb.max - aabb.min;
        let split_axis = if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        };

        let mid = aabb.min.to_array()[split_axis] + extent.to_array()[split_axis] * 0.5;

        let mut left = Vec::new();
        let mut right = Vec::new();

        for t in tris {
            if t.centroid.to_array()[split_axis] <= mid {
                left.push(t.clone());
            } else {
                right.push(t.clone());
            }
        }

        if left.is_empty() || right.is_empty() {
            return BVHNode::Leaf {
                aabb,
                triangles: tris.to_vec(),
            };
        }

        BVHNode::Internal {
            aabb,
            left: Box::new(Self::build_recursive(&left, depth + 1)),
            right: Box::new(Self::build_recursive(&right, depth + 1)),
            split_axis,
        }
    }

    pub fn query_intersections(&self, other: &BVH) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        Self::query_recursive(&self.root, &other.root, &mut pairs);
        pairs
    }

    fn query_recursive<'a>(
        node_a: &BVHNode,
        node_b: &BVHNode,
        pairs: &mut Vec<(usize, usize)>,
    ) {
        if !node_a.aabb().intersect(node_b.aabb()) {
            return;
        }

        match (node_a, node_b) {
            (BVHNode::Leaf { triangles: tris_a, .. },
             BVHNode::Leaf { triangles: tris_b, .. }) => {
                for ta in tris_a {
                    for tb in tris_b {
                        pairs.push((ta.index, tb.index));
                    }
                }
            }
            (BVHNode::Leaf { triangles: _tris_a, .. }, BVHNode::Internal { left, right, .. }) => {
                Self::query_recursive(node_a, left, pairs);
                Self::query_recursive(node_a, right, pairs);
            }
            (BVHNode::Internal { left, right, .. }, BVHNode::Leaf { .. }) => {
                Self::query_recursive(left, node_b, pairs);
                Self::query_recursive(right, node_b, pairs);
            }
            (BVHNode::Internal { left: l1, right: r1, .. },
             BVHNode::Internal { left: l2, right: r2, .. }) => {
                Self::query_recursive(l1, l2, pairs);
                Self::query_recursive(l1, r2, pairs);
                Self::query_recursive(r1, l2, pairs);
                Self::query_recursive(r1, r2, pairs);
            }
        }
    }
}

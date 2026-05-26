use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kpe_schema::geometry::{
    BoxDef, CylinderDef, GeometryNode, GeometryNodeType, TriangleMesh,
};
use kpe_geometry::mesh::build_mesh_from_node;
use kpe_geometry::CsgKernel;

fn make_box() -> TriangleMesh {
    let node = GeometryNode {
        id: "box".into(),
        node_type: GeometryNodeType::Box(BoxDef { width: 2.0, height: 2.0, depth: 2.0 }),
        transform: None,
        children: vec![],
        operations: vec![],
    };
    build_mesh_from_node(&node)
}

fn make_cylinder(segments: u32) -> TriangleMesh {
    let node = GeometryNode {
        id: "cyl".into(),
        node_type: GeometryNodeType::Cylinder(CylinderDef { radius: 1.0, height: 3.0 }),
        transform: None,
        children: vec![],
        operations: vec![],
    };
    build_mesh_from_node(&node)
}

fn bench_csg_union(c: &mut Criterion) {
    let kernel = CsgKernel;
    let box_mesh = make_box();
    let cyl_mesh = make_cylinder(64);

    c.bench_function("csg/union_box_cylinder", |b| {
        b.iter(|| {
            let result = kernel.union(black_box(&box_mesh), black_box(&cyl_mesh));
            black_box(result)
        })
    });
}

fn bench_csg_subtract(c: &mut Criterion) {
    let kernel = CsgKernel;
    let box_mesh = make_box();
    let cyl_mesh = make_cylinder(64);

    c.bench_function("csg/subtract_cylinder_from_box", |b| {
        b.iter(|| {
            let result = kernel.subtract(black_box(&box_mesh), black_box(&cyl_mesh));
            black_box(result)
        })
    });
}

fn bench_csg_intersect(c: &mut Criterion) {
    let kernel = CsgKernel;
    let box_mesh = make_box();
    let cyl_mesh = make_cylinder(64);

    c.bench_function("csg/intersect_box_cylinder", |b| {
        b.iter(|| {
            let result = kernel.intersect(black_box(&box_mesh), black_box(&cyl_mesh));
            black_box(result)
        })
    });
}

fn bench_bvh_build(c: &mut Criterion) {
    use kpe_geometry::BVH;
    use glam::DVec3;

    let mut triangles = Vec::new();
    for i in 0..200 {
        let x = i as f64 * 0.5;
        triangles.push((
            DVec3::new(x, 0.0, 0.0),
            DVec3::new(x + 0.4, 1.0, 0.0),
            DVec3::new(x, 0.0, 0.5),
        ));
    }

    c.bench_function("bvh/build_200_triangles", |b| {
        b.iter(|| {
            let bvh = BVH::build(black_box(&triangles));
            black_box(bvh)
        })
    });
}

criterion_group! {
    name = csg;
    config = Criterion::default().sample_size(30);
    targets = bench_csg_union, bench_csg_subtract, bench_csg_intersect, bench_bvh_build
}
criterion_main!(csg);

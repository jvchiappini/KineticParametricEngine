use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kpe_schema::geometry::{
    ExtrudeDef, RevolveDef, RevolveAxis, SketchDef, SketchPlane, SketchPrimitive, SweepDef, SweepPath, TriangleMesh,
};
use kpe_geometry::extrude::{extrude_sketch, revolve_sketch, sweep_sketch};

fn make_rect_sketch() -> SketchDef {
    SketchDef {
        plane: SketchPlane::XY,
        primitives: vec![
            SketchPrimitive::Rectangle { x: -2.0, y: -1.0, width: 4.0, height: 2.0 },
        ],
    }
}

fn make_complex_sketch() -> SketchDef {
    let n = 100;
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let angle = (i as f64 / n as f64) * std::f64::consts::TAU;
        let r = 2.0 + 0.5 * (angle * 5.0).sin();
        pts.push([r * angle.cos(), r * angle.sin()]);
    }
    SketchDef {
        plane: SketchPlane::XY,
        primitives: vec![SketchPrimitive::Polygon { points: pts }],
    }
}

fn bench_extrude_rect(c: &mut Criterion) {
    let sketch = make_rect_sketch();
    let ext = ExtrudeDef {
        sketch_id: "".into(),
        distance: 5.0,
        direction: None,
        cap: true,
    };

    c.bench_function("extrude/rectangle", |b| {
        b.iter(|| {
            let result = extrude_sketch(black_box(&sketch), black_box(&ext));
            black_box(result)
        })
    });
}

fn bench_extrude_complex(c: &mut Criterion) {
    let sketch = make_complex_sketch();
    let ext = ExtrudeDef {
        sketch_id: "".into(),
        distance: 3.0,
        direction: None,
        cap: true,
    };

    c.bench_function("extrude/complex_100_verts", |b| {
        b.iter(|| {
            let result = extrude_sketch(black_box(&sketch), black_box(&ext));
            black_box(result)
        })
    });
}

fn bench_revolve(c: &mut Criterion) {
    let sketch = make_rect_sketch();
    let rev = RevolveDef {
        sketch_id: "".into(),
        angle: std::f64::consts::TAU,
        segments: Some(48),
        axis: RevolveAxis::Y,
        cap: false,
    };

    c.bench_function("revolve/rectangle_full", |b| {
        b.iter(|| {
            let result = revolve_sketch(black_box(&sketch), black_box(&rev));
            black_box(result)
        })
    });
}

fn bench_sweep_helix(c: &mut Criterion) {
    let sketch = SketchDef {
        plane: SketchPlane::YZ,
        primitives: vec![
            SketchPrimitive::Circle { cx: 0.0, cy: 0.0, radius: 0.2, segments: Some(12) },
        ],
    };
    let swp = SweepDef {
        sketch_id: "".into(),
        path: SweepPath::Helix { radius: 1.5, pitch: 0.8, turns: 5.0 },
        segments: Some(120),
        cap: false,
    };

    c.bench_function("sweep/helix_spring", |b| {
        b.iter(|| {
            let result = sweep_sketch(black_box(&sketch), black_box(&swp));
            black_box(result)
        })
    });
}

criterion_group! {
    name = sketch;
    config = Criterion::default().sample_size(30);
    targets = bench_extrude_rect, bench_extrude_complex, bench_revolve, bench_sweep_helix
}
criterion_main!(sketch);

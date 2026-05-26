mod export;

use std::env;
use kpe_schema::recipe::KPERecipe;
use kpe_parametric::Solver;
use kpe_geometry::mesh::MeshBuilder;

fn die(msg: &str) -> ! {
    eprintln!("Error: {msg}");
    std::process::exit(1);
}

fn usage() {
    eprintln!(
        "Usage:
  kpe resolve <recipe.json>               — resolves parametric rules, prints result
  kpe export <recipe.json> <output.step>  — export to STEP (AP242 tessellation)
  kpe export <recipe.json> <output.dxf>   — export to DXF (3DFACE entities)
  kpe help                                 — this help"
    );
    std::process::exit(1);
}

fn load_recipe(path: &str) -> KPERecipe {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| die(&format!("read {path}: {e}")));
    serde_json::from_str(&text).unwrap_or_else(|e| die(&format!("parse {path}: {e}")))
}

fn build_mesh(recipe: &KPERecipe) -> kpe_schema::geometry::TriangleMesh {
    let solver = Solver::new();
    let resolved = solver.resolve(recipe).unwrap_or_else(|e| die(&format!("solver: {e:?}")));
    let builder = MeshBuilder::new();
    builder.build_from_node(&resolved.recipe.scene)
}

fn cmd_export(args: &[String]) {
    if args.len() < 4 {
        usage();
    }
    let recipe_path = &args[2];
    let output_path = &args[3];
    let recipe = load_recipe(recipe_path);
    let mesh = build_mesh(&recipe);

    let lower = output_path.to_lowercase();
    if lower.ends_with(".step") || lower.ends_with(".stp") {
        export::step::export_step(&mesh, output_path).unwrap_or_else(|e| die(&e));
    } else if lower.ends_with(".dxf") {
        export::dxf::export_dxf(&mesh, output_path).unwrap_or_else(|e| die(&e));
    } else {
        die(&format!("unknown format: {output_path} (use .step, .stp, or .dxf)"));
    }
    println!("Wrote {}", output_path);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }
    match args[1].as_str() {
        "resolve" => {
            if args.len() < 3 { usage(); }
            let recipe = load_recipe(&args[2]);
            let solver = Solver::new();
            let resolved = solver.resolve(&recipe).unwrap_or_else(|e| die(&format!("solver: {e:?}")));
            let json = serde_json::to_string_pretty(&resolved).unwrap_or_else(|e| die(&e.to_string()));
            println!("{json}");
        }
        "export" => cmd_export(&args),
        "help" | "--help" | "-h" => usage(),
        _ => usage(),
    }
}

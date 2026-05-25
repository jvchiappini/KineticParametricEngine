use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutPiece {
    pub id: String,
    pub label: String,
    pub width: f64,
    pub height: f64,
    pub thickness: f64,
    pub quantity: u32,
    pub material_id: String,
    pub grain_direction: GrainDirection,
    pub surface_area: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CutList {
    pub pieces: Vec<CutPiece>,
    pub total_pieces: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrainDirection {
    Lengthwise,
    Widthwise,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestingConfig {
    pub sheet_width: f64,
    pub sheet_height: f64,
    pub blade_width: f64,
    pub respect_grain: bool,
    pub margin: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestingSheet {
    pub pieces: Vec<NestedPiece>,
    pub waste_area: f64,
    pub utilization_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NestedPiece {
    pub piece_id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricationOutput {
    pub cut_list: CutList,
    pub nesting: Vec<NestingSheet>,
    pub dxf_output: Option<Vec<u8>>,
    pub svg_output: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricationError {
    pub message: String,
    pub code: FabricationErrorCode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FabricationErrorCode {
    InvalidDimensions,
    NoPieces,
    NestingFailed,
    ExportFailed,
}

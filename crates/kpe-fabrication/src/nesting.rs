use kpe_schema::fabrication::{CutList, NestingConfig, NestingSheet, NestedPiece};

pub struct NestingEngine;

impl NestingEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn optimize(&self, cutlist: &CutList, config: &NestingConfig) -> Vec<NestingSheet> {
        let mut sheets = Vec::new();
        let mut remaining: Vec<_> = cutlist.pieces.iter().collect();

        while !remaining.is_empty() {
            let mut placed = Vec::new();
            let mut cursor_x = config.margin;
            let mut cursor_y = config.margin;
            let mut row_height = 0.0;

            remaining.retain(|piece| {
                let w = piece.width + config.margin;
                let h = piece.height + config.margin;

                if cursor_x + w > config.sheet_width - config.margin {
                    cursor_x = config.margin;
                    cursor_y += row_height + config.margin;
                    row_height = 0.0;
                }

                if cursor_y + h > config.sheet_height - config.margin {
                    return true;
                }

                placed.push(NestedPiece {
                    piece_id: piece.id.clone(),
                    x: cursor_x,
                    y: cursor_y,
                    width: piece.width,
                    height: piece.height,
                    rotated: false,
                });

                cursor_x += w;
                row_height = row_height.max(h);
                false
            });

            let sheet_area = config.sheet_width * config.sheet_height;
            let used_area: f64 = placed.iter()
                .map(|p| p.width * p.height)
                .sum();

            sheets.push(NestingSheet {
                pieces: placed,
                waste_area: sheet_area - used_area,
                utilization_pct: (used_area / sheet_area) * 100.0,
            });
        }

        sheets
    }
}

impl Default for NestingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kpe_schema::fabrication::{CutPiece, GrainDirection, CutList};

    fn make_cutlist() -> CutList {
        CutList {
            pieces: vec![
                CutPiece {
                    id: "a".to_string(), label: "A".to_string(),
                    width: 100.0, height: 50.0, thickness: 18.0,
                    quantity: 1, material_id: "mdf".to_string(),
                    grain_direction: GrainDirection::Lengthwise,
                    surface_area: 10000.0,
                },
                CutPiece {
                    id: "b".to_string(), label: "B".to_string(),
                    width: 80.0, height: 40.0, thickness: 18.0,
                    quantity: 2, material_id: "mdf".to_string(),
                    grain_direction: GrainDirection::Widthwise,
                    surface_area: 6400.0,
                },
            ],
            total_pieces: 3,
        }
    }

    #[test]
    fn test_nesting_places_all_pieces() {
        let engine = NestingEngine::new();
        let config = NestingConfig {
            sheet_width: 600.0, sheet_height: 300.0,
            blade_width: 3.0, respect_grain: false, margin: 5.0,
        };
        let cutlist = make_cutlist();
        let sheets = engine.optimize(&cutlist, &config);
        assert!(!sheets.is_empty());
        let total_placed: usize = sheets.iter().map(|s| s.pieces.len()).sum();
        assert_eq!(total_placed, 2);
    }

    #[test]
    fn test_nesting_utilization() {
        let engine = NestingEngine::new();
        let config = NestingConfig {
            sheet_width: 600.0, sheet_height: 300.0,
            blade_width: 3.0, respect_grain: false, margin: 5.0,
        };
        let cutlist = make_cutlist();
        let sheets = engine.optimize(&cutlist, &config);
        assert!(sheets[0].utilization_pct > 0.0);
        assert!(sheets[0].utilization_pct <= 100.0);
    }

    #[test]
    fn test_nesting_empty_cutlist() {
        let engine = NestingEngine::new();
        let config = NestingConfig {
            sheet_width: 600.0, sheet_height: 300.0,
            blade_width: 3.0, respect_grain: false, margin: 5.0,
        };
        let empty = CutList { pieces: vec![], total_pieces: 0 };
        let sheets = engine.optimize(&empty, &config);
        assert!(sheets.is_empty());
    }
}

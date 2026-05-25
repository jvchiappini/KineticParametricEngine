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

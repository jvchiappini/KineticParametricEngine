use kpe_schema::fabrication::NestingSheet;

pub struct DxfExporter;

impl DxfExporter {
    pub fn new() -> Self {
        Self
    }

    pub fn export(&self, sheets: &[NestingSheet]) -> Vec<u8> {
        let mut output = String::new();

        output.push_str("0\nSECTION\n2\nHEADER\n0\nENDSEC\n");
        output.push_str("0\nSECTION\n2\nENTITIES\n");

        for (sheet_idx, sheet) in sheets.iter().enumerate() {
            for piece in &sheet.pieces {
                let x1 = piece.x;
                let y1 = piece.y;
                let x2 = piece.x + piece.width;
                let y2 = piece.y + piece.height;

                output.push_str(&format!(
                    "0\nLWPOLYLINE\n8\n{}\n90\n4\n10\n{}\n20\n{}\n10\n{}\n20\n{}\n10\n{}\n20\n{}\n10\n{}\n20\n{}\n0\n",
                    sheet_idx + 1,
                    x1, y1, x2, y1, x2, y2, x1, y2
                ));
            }
        }

        output.push_str("0\nENDSEC\n0\nEOF\n");

        output.into_bytes()
    }
}

impl Default for DxfExporter {
    fn default() -> Self {
        Self::new()
    }
}
